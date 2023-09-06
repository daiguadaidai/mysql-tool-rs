use crate::config::show_processlist_conf::ShowProcesslistConf;
use crate::core::show_processlist::common;
use crate::dao::{InstanceDao, MetaClusterDao, NormalDao};
use crate::error::CustomError;
use crate::models::{Instance, ShowProcesslistInfo};
use crate::{rdbc, utils};
use sqlx::{MySql, Pool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub async fn run(cfg: &ShowProcesslistConf) -> Result<(), CustomError> {
    // 检测参数
    cfg.check_all()?;

    // 如果文件路径不存在则创建
    utils::file::create_dir(&cfg.output_dir).map_err(|e| {
        CustomError::new(format!(
            "创建保存processlist信息目录出错, 目录: {dir}. {e}",
            dir = &cfg.output_dir,
            e = e.to_string()
        ))
    })?;

    // 创建一个全局共享map
    let old_instance_set = Arc::new(RwLock::new(HashSet::<String>::new()));

    let (tx, mut rx) = mpsc::channel::<Instance>(100);
    // 获取所有的实例
    let tx_product = tx.clone();
    let cfg_product = cfg.clone();
    let old_instance_set_product = old_instance_set.clone();
    tokio::spawn(async move {
        // 产生所有实例
        if let Err(e) = product_instance(&cfg_product, &tx_product, &old_instance_set_product).await
        {
            log::error!("异步产生实例出错. {e}", e = e.to_string())
        }
    });

    while let Some(instance) = rx.recv().await {
        let tmp_old_instance_set = old_instance_set.clone();
        let tmp_cfg = cfg.clone();
        tokio::spawn(async move {
            // 添加实例
            add_instance(&tmp_old_instance_set, &instance);
            log::info!(
                "接收到实例: {host}:{port}, 并且添加到 set 成功",
                host = &instance.machine_host.as_ref().unwrap(),
                port = &instance.port.unwrap(),
            );

            // 开始执行 processlist
            if let Err(e) =
                show_processlist_by_instance(&tmp_old_instance_set, &tmp_cfg, &instance).await
            {
                log::error!(
                    "{host}:{port}. 执行 processlist 失败. {e}",
                    host = &instance.machine_host.as_ref().unwrap(),
                    port = &instance.port.unwrap(),
                    e = e.to_string(),
                )
            }

            // 结束processlist 任务
            log::info!(
                "实例 {host}:{port} processlist 结束",
                host = &instance.machine_host.as_ref().unwrap(),
                port = &instance.port.unwrap(),
            );

            // 删除结束的实例
            delete_instance(&old_instance_set, &instance);
        });
    }

    Ok(())
}

// 产生实例
async fn product_instance(
    cfg: &ShowProcesslistConf,
    tx: &Sender<Instance>,
    old_instance_set: &Arc<RwLock<HashSet<String>>>,
) -> Result<(), CustomError> {
    // 获取所有实例
    loop {
        let instances = match find_all_instances(cfg).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("获取所有实例失败: {e}", e = e.to_string());
                continue;
            }
        };

        // 获取 cluster#host:port 的实例map
        let mut instance_map = HashMap::<String, Instance>::new();
        instances.iter().for_each(|instance| {
            let key = format!(
                "{host}:{port}",
                host = instance.machine_host.as_ref().unwrap(),
                port = instance.port.unwrap(),
            );
            instance_map.entry(key).or_insert(instance.clone());
        });

        // 获取需要新增的实例 现在 - 老的
        let need_add_instances = get_need_add_instance(old_instance_set, &instance_map);

        // 获取需要删除的实例 老的 - 现在
        let need_remove_instances = get_need_remove_instance(old_instance_set, &instance_map);

        // 对实例进行删除
        delete_instance_by_vec(old_instance_set, &need_remove_instances);

        // 将需要新增的实例进行发送
        for instance in need_add_instances.into_iter() {
            let key = format!(
                "{host}:{port}",
                host = &instance.machine_host.as_ref().unwrap(),
                port = &instance.port.unwrap(),
            );

            if let Err(e) = tx.send(instance).await {
                log::error!("发送实例出错. {}", utils::string::to_json_str(&e.0))
            }
            log::info!("{key} 发送成功.", key = &key)
        }

        // 休眠多少秒
        let _ = tokio::time::sleep(std::time::Duration::from_secs(
            cfg.product_instance_duration,
        ))
        .await;
    }
}

// 获取所有实例
async fn find_all_instances(cfg: &ShowProcesslistConf) -> Result<Vec<Instance>, CustomError> {
    // 获取数据库链接
    // 获取 easydb 数据库链接
    let easydb = rdbc::get_db(cfg.get_easydb_dsn().as_str(), cfg.is_sql_log)
        .await
        .map_err(|e| {
            CustomError::new(format!("获取easydb数据库链接出错. {e}", e = e.to_string()))
        })?;

    // 获取所有实例
    let mut instances = match find_all_instances_op(&easydb).await {
        Ok(v) => v,
        Err(e) => {
            // 关闭链接
            let _ = easydb.close().await;
            return Err(e);
        }
    };

    let ignore_instances = cfg.ignore_instances_to_set();
    instances = instances
        .into_iter()
        .filter(|instance| {
            let key = format!(
                "{host}:{port}",
                host = instance.machine_host.as_ref().unwrap(),
                port = instance.port.unwrap(),
            );

            if ignore_instances.get(&key).is_some() {
                log::info!(
                    "该实例将不进行获取processlist信息, 手动指定了忽略该实例 {host}:{port}",
                    host = instance.machine_host.as_ref().unwrap(),
                    port = instance.port.unwrap(),
                );
                return false;
            } else {
                return true;
            }
        })
        .collect::<Vec<Instance>>();

    // 关闭 数据库链接
    let _ = easydb.close().await;

    Ok(instances)
}

async fn find_all_instances_op(easydb: &Pool<MySql>) -> Result<Vec<Instance>, CustomError> {
    // 获取所有集群
    let clusters = MetaClusterDao::all(easydb)
        .await
        .map_err(|e| CustomError::new(format!("获取所有集群失败: {e}", e = e.to_string())))?;

    // 遍历集群 获取所有实例
    let mut instances = Vec::<Instance>::new();
    for cluster in clusters {
        let mut tmp_instances =
            match InstanceDao::find_by_meta_cluster_id(easydb, cluster.id.unwrap())
                .await
                .map_err(|e| {
                    CustomError::new(format!(
                        "通过集群获取实例失败. 集群: {cluster}. {e}",
                        cluster = utils::string::to_json_str(&cluster),
                        e = e.to_string()
                    ))
                }) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{}", e.to_string());
                    continue;
                }
            };
        log::info!(
            "通过集群获取实例成功: 集群: {cluster_name}, 实例数: {instance_count}",
            cluster_name = &cluster.name.as_ref().unwrap(),
            instance_count = tmp_instances.len()
        );

        instances.append(&mut tmp_instances);
    }

    Ok(instances)
}

// 获取需要新添加的实例
fn get_need_add_instance(
    old_set: &Arc<RwLock<HashSet<String>>>,
    new_map: &HashMap<String, Instance>,
) -> Vec<Instance> {
    let mut need_add_instances = Vec::<Instance>::new();

    // 新的存在, 老的不存在
    new_map.iter().for_each(|(key, instance)| {
        let old_set = old_set.read().unwrap();
        if old_set.get(key).is_none() {
            log::info!("{key}, 是需要 [新增] 的实例");
            need_add_instances.push(instance.clone());
        }
    });

    need_add_instances
}

// 获取需要新移除的实例
fn get_need_remove_instance(
    old_set: &Arc<RwLock<HashSet<String>>>,
    new_map: &HashMap<String, Instance>,
) -> Vec<String> {
    let mut need_remove_instances = Vec::<String>::new();

    // 老的存在, 新的不存在
    let old_set = old_set.read().unwrap();
    old_set.iter().for_each(|key| {
        if new_map.get(key).is_none() {
            log::info!("{key}, 是需要 [移除] 的实例");
            need_remove_instances.push(key.to_string());
        }
    });

    need_remove_instances
}

fn delete_instance_by_vec(
    old_set: &Arc<RwLock<HashSet<String>>>,
    need_delete_instances: &Vec<String>,
) {
    let mut old_set = old_set.write().unwrap();
    for key in need_delete_instances.iter() {
        old_set.remove(key);
        log::info!("{}, 已经从 set 中移除", key)
    }
}

fn delete_instance(old_set: &Arc<RwLock<HashSet<String>>>, instance: &Instance) {
    let key = format!(
        "{host}:{port}",
        host = instance.machine_host.as_ref().unwrap(),
        port = instance.port.unwrap(),
    );
    let mut old_set = old_set.write().unwrap();
    old_set.remove(&key);
    log::info!("{}, 已经从 set 中移除", key)
}

fn add_instance(old_set: &Arc<RwLock<HashSet<String>>>, instance: &Instance) {
    let key = format!(
        "{host}:{port}",
        host = instance.machine_host.as_ref().unwrap(),
        port = instance.port.unwrap(),
    );

    let mut old_set = old_set.write().unwrap();
    old_set.insert(key);
}

fn exists_instance(old_set: &Arc<RwLock<HashSet<String>>>, instance: &Instance) -> bool {
    let key = format!(
        "{host}:{port}",
        host = instance.machine_host.as_ref().unwrap(),
        port = instance.port.unwrap(),
    );

    old_set.read().unwrap().get(&key).is_some()
}

async fn show_processlist_by_instance(
    old_set: &Arc<RwLock<HashSet<String>>>,
    cfg: &ShowProcesslistConf,
    instance: &Instance,
) -> Result<(), CustomError> {
    let password = cfg.get_password();
    // 创建数据库链接
    let db = rdbc::get_db_by_default(
        instance.machine_host.as_ref().unwrap(),
        instance.port.unwrap() as i16,
        &cfg.username,
        &password,
        "",
        cfg.is_sql_log,
    )
    .await
    .map_err(|e| {
        CustomError::new(format!(
            "创建需要执行 processlist 数据库链接失败. host:port:{host}:{port}. {e}",
            host = instance.machine_host.as_ref().unwrap(),
            port = instance.port.unwrap(),
            e = e.to_string()
        ))
    })?;

    let mut clean_timestamp = utils::time::now_datetime().timestamp();

    loop {
        if let Err(e) = start_processlist(cfg, instance, &db, &mut clean_timestamp).await {
            log::error!(
                "{host}:{port}, 执行 show processlist 出错. {e}",
                host = instance.machine_host.as_ref().unwrap(),
                port = instance.port.unwrap(),
                e = e.to_string()
            );
            // 休眠多少毫秒, 出错后需要睡眠60秒后再重新
            let _ = tokio::time::sleep(std::time::Duration::from_millis(60 * 1000)).await;
        }

        // 实例已经不存在, 被移除了就不跑了
        if !exists_instance(old_set, instance) {
            break;
        }

        // 休眠多少毫秒
        let _ = tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep)).await;
    }

    let _ = db.close().await;

    Ok(())
}

async fn start_processlist(
    cfg: &ShowProcesslistConf,
    instance: &Instance,
    db: &Pool<MySql>,
    clean_timestamp: &mut i64,
) -> Result<(), CustomError> {
    let infos = NormalDao::show_processlist(&db).await.map_err(|e| {
        CustomError::new(format!(
            "{host}:{port}, 获取processlist信息失败. {e}",
            host = instance.machine_host.as_ref().unwrap(),
            port = instance.port.unwrap(),
            e = e.to_string()
        ))
    })?;

    // 过滤 processlist 信息, 过滤掉 command=Sleep
    let filter_infos_sleep = infos
        .iter()
        .map(|info| info.clone())
        .filter(|info| info.command.as_ref().unwrap() != "Sleep")
        .collect::<Vec<ShowProcesslistInfo>>();

    // 在 过滤掉 command=Sleep 基础上过滤掉 user=system user
    let fitler_infos_system_user = filter_infos_sleep
        .iter()
        .map(|info| info.clone())
        .filter(|info| info.user.as_ref().unwrap() != "system user")
        .collect::<Vec<ShowProcesslistInfo>>();

    // 除了 Sleep 和 system user 外的processlist 超过了指定数需要进行记录
    if fitler_infos_system_user.len() >= cfg.print_cnt_threshold as usize {
        let infos_table = common::get_infos_table(&filter_infos_sleep);

        let log_data = format!(
                "\n---- {host}:{port} Time: {time}, Total: {total}, Filter Sleep: {filter_sleep} ----\n{infos_table}",
                host = instance.machine_host.as_ref().unwrap(),
                port = instance.port.unwrap(),
                time = &utils::time::now_str(utils::time::NORMAL_FMT),
                total=infos.len(),
                filter_sleep = filter_infos_sleep.len(),
                infos_table = infos_table,
            );

        let now_timestamp = utils::time::now_datetime().timestamp();

        // 打开文件, 并且追加内容
        let file_path = format!(
            "{dir}/{host}_{port}.txt",
            dir = &cfg.output_dir,
            host = instance.machine_host.as_ref().unwrap(),
            port = instance.port.unwrap()
        );

        let mut open_ops = fs::OpenOptions::new();
        if !Path::new(&file_path).exists() {
            open_ops.create_new(true).append(true);
        } else {
            // 达到需要清理文件的时间
            if now_timestamp - *clean_timestamp >= cfg.clear_file_duration {
                open_ops.write(true).truncate(true);

                // 清理后清理时间变成当前时间
                *clean_timestamp = now_timestamp;
            } else {
                // append打开
                open_ops.append(true);
            }
        }

        // 打开文件
        let mut file = open_ops.open(&file_path).map_err(|e| {
            CustomError::new(format!(
                "打开文件出错. 路径: {file_path}. {e}",
                file_path = &file_path,
                e = e.to_string()
            ))
        })?;
        // processlist写入文件
        file.write_all(log_data.as_bytes()).map_err(|e| {
            CustomError::new(format!(
                "写入 processlist 信息失败, 文件: {file_path}, {e}",
                file_path = &file_path,
                e = e.to_string()
            ))
        })?;
    }

    Ok(())
}
