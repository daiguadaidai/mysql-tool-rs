use crate::config::show_processlist_conf::ShowProcesslistConf;
use crate::core::show_processlist::{all_cluster_handler, common};
use crate::dao::{InstanceDao, MetaClusterDao, NormalDao};
use crate::error::CustomError;
use crate::models::{Instance, ShowProcesslistInfo};
use crate::{rdbc, utils};
use sqlx::{MySql, Pool};
use tokio::sync::mpsc;

pub async fn run(cfg: &ShowProcesslistConf) -> Result<(), CustomError> {
    log::info!("配置文件: {}", utils::string::to_json_str_pretty(cfg));
    // 检测配置文件相关参数
    cfg.check()?;

    // 指定 host port
    if cfg.have_host_port() {
        log::info!("通过 host port 获取 processlist 信息");
        start_host_port(cfg).await?;
    } else if cfg.have_vip_port() {
        log::info!("通过 vip port 获取 processlist 信息");
        start_vip_port(cfg).await?;
    } else if cfg.all {
        log::info!("指定了 --all 参数, 获取所有集群实例 processlist 信息");
        all_cluster_handler::run(cfg).await?;
    }

    Ok(())
}

async fn start_vip_port(cfg: &ShowProcesslistConf) -> Result<(), CustomError> {
    // 通过 vip_port 获取所有实例
    let instances = find_instances(cfg).await?;
    log::info!(
        "实例信息为: {}",
        utils::string::to_json_str_pretty(&instances)
    );

    let (tx, mut rx) = mpsc::channel::<String>(instances.len());
    for instance in instances {
        let tmp_tx = tx.clone();
        let tmp_cfg = cfg.clone();
        tokio::spawn(async move {
            // 开始执行 processlist
            if let Err(e) = start_processlist_by_instance(&tmp_cfg, &instance).await {
                log::error!(
                    "{host}:{port}, {e}",
                    host = &instance.machine_host.as_ref().unwrap(),
                    port = &instance.port.unwrap(),
                    e = e.to_string()
                );
            }

            // 发送任务完成消息
            let message = format!("{vip_port} 任务完成", vip_port = &tmp_cfg.vip_port);
            let _ = tmp_tx.send(message).await;
        });
    }
    // 手动释放多出来到变量
    drop(tx);

    while let Some(message) = rx.recv().await {
        log::info!("接收到一个完成到任务: {message}", message = &message)
    }

    Ok(())
}

// 获取集群实例
async fn find_instances(cfg: &ShowProcesslistConf) -> Result<Vec<Instance>, CustomError> {
    // 获取数据库链接
    // 获取 easydb 数据库链接
    let easydb = rdbc::get_db(cfg.get_easydb_dsn().as_str(), cfg.is_sql_log)
        .await
        .map_err(|e| {
            CustomError::new(format!("获取easydb数据库链接出错. {e}", e = e.to_string()))
        })?;

    // 获取所有实例
    let instances = match find_instances_op(cfg, &easydb).await {
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

async fn find_instances_op(
    cfg: &ShowProcesslistConf,
    easydb: &Pool<MySql>,
) -> Result<Vec<Instance>, CustomError> {
    // 通过vip_port获取集群
    let cluster = MetaClusterDao::get_by_vip_port(easydb, &cfg.vip_port)
        .await
        .map_err(|e| {
            CustomError::new(format!(
                "通过vip_port 获取集群信息失败. vip_port: {vip_port}, {e}",
                vip_port = &cfg.vip_port,
                e = e.to_string()
            ))
        })?;
    log::info!(
        "通过vip_port获取集群信息成功: {}",
        utils::string::to_json_str_pretty(&cluster)
    );

    // 通过集群 id 获取所有实例
    InstanceDao::find_by_meta_cluster_id(easydb, cluster.id.unwrap())
        .await
        .map_err(|e| {
            CustomError::new(format!(
                "通过集群id获取实例信息失败. meta_cluster_id: {meta_cluster_id}, vip_port: {vip_port}. {e}",
                meta_cluster_id=cluster.id.unwrap(),
                vip_port=&cfg.vip_port,
                e=e.to_string()
            ))
        })
}

// 开始执行实例级别processlist
async fn start_processlist_by_instance(
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

    // 循环执行 processlist
    loop {
        let infos = match NormalDao::show_processlist(&db).await {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "{host}:{port}, 获取processlist信息失败. {e}",
                    host = instance.machine_host.as_ref().unwrap(),
                    port = instance.port.unwrap(),
                    e = e.to_string()
                );
                // 休眠多少毫秒
                let _ = tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep)).await;
                continue;
            }
        };

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
            let infos_table = common::get_infos_table(&infos);

            // 记录日志
            log::info!(
                "\n---- {host}:{port} Time: {time}, Total: {total}, Filter Sleep: {filter_sleep} ----\n{infos_table}",
                host = instance.machine_host.as_ref().unwrap(),
                port = instance.port.unwrap(),
                time = &utils::time::now_str(utils::time::NORMAL_FMT),
                total=infos.len(),
                filter_sleep = filter_infos_sleep.len(),
                infos_table = infos_table,
            );
        }

        // 休眠多少毫秒
        let _ = tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep)).await;
    }
}

async fn start_host_port(cfg: &ShowProcesslistConf) -> Result<(), CustomError> {
    let password = cfg.get_password();
    // 创建数据库链接
    let db = rdbc::get_db_by_default(
        &cfg.host,
        cfg.port as i16,
        &cfg.username,
        &password,
        "",
        cfg.is_sql_log,
    )
    .await
    .map_err(|e| {
        CustomError::new(format!(
            "创建需要执行 processlist 数据库链接失败. host:port:{host}:{port}. {e}",
            host = &cfg.host,
            port = cfg.port,
            e = e.to_string()
        ))
    })?;

    // 循环执行 processlist
    loop {
        let infos = match NormalDao::show_processlist(&db).await {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "{host}:{port}, 获取processlist信息失败. {e}",
                    host = &cfg.host,
                    port = cfg.port,
                    e = e.to_string()
                );
                // 休眠多少毫秒
                let _ = tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep)).await;
                continue;
            }
        };

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

            // 记录日志
            log::info!(
                "\n---- {host}:{port} Time: {time}, Total: {total}, Filter Sleep: {filter_sleep} ----\n{infos_table}",
                host = &cfg.host,
                port = cfg.port,
                time = &utils::time::now_str(utils::time::NORMAL_FMT),
                total=infos.len(),
                filter_sleep = filter_infos_sleep.len(),
                infos_table = infos_table,
            );
        }

        // 休眠多少毫秒
        let _ = tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep)).await;
    }
}
