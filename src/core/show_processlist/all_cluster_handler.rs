use crate::config::show_processlist_conf::ShowProcesslistConf;
use crate::dao::{InstanceDao, MetaClusterDao};
use crate::error::CustomError;
use crate::models::Instance;
use crate::{rdbc, utils};
use sqlx::{MySql, Pool};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub async fn run(cfg: &ShowProcesslistConf) -> Result<(), CustomError> {
    // 检测参数
    cfg.check_all()?;

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
        let _tmp_cfg = cfg.clone();
        tokio::spawn(async move {
            // 添加实例
            add_instance(&tmp_old_instance_set, &instance);
            log::info!(
                "接收到实例: {host}:{port}, 并且添加到 set 成功",
                host = &instance.machine_host.as_ref().unwrap(),
                port = &instance.port.unwrap(),
            );

            // 开始执行 processlist

            // 结束processlist 任务
            log::info!(
                "实例 {host}:{port} 结束processlist完成",
                host = &instance.machine_host.as_ref().unwrap(),
                port = &instance.port.unwrap(),
            );
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
        delete_instance(old_instance_set, &need_remove_instances);

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
    let instances = match find_all_instances_op(&easydb).await {
        Ok(v) => v,
        Err(e) => {
            // 关闭链接
            let _ = easydb.close().await;
            return Err(e);
        }
    };

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

fn delete_instance(old_set: &Arc<RwLock<HashSet<String>>>, need_delete_instances: &Vec<String>) {
    let mut old_set = old_set.write().unwrap();
    for key in need_delete_instances.iter() {
        old_set.remove(key);
        log::info!("{}, 已经从 set 中移除", key)
    }
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
