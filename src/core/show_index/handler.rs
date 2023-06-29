use crate::config::show_index_conf::ShowIndexConf;
use crate::dao::{InstanceDao, MetaClusterDao, NormalDao};
use crate::error::CustomError;
use crate::models::show_index_info::{ShowIndexInfo8, ShowIndexInfosEnum};
use crate::models::{Instance, ShowIndexInfo};
use crate::{rdbc, utils};
use prettytable::{format, Cell, Row, Table};
use sqlx::{MySql, Pool};

pub async fn run(cfg: &ShowIndexConf) -> Result<(), CustomError> {
    log::info!("{}", utils::string::to_json_str_pretty(&cfg));
    // 检测配置文件相关参数
    cfg.check()?;

    // 链接数据库
    let easydb_db = rdbc::get_db(&cfg.get_easydb_dsn(), cfg.is_sql_log)
        .await
        .map_err(|e| {
            CustomError::new(format!("创建 EasyDB 实例链接出错. {e}", e = e.to_string()))
        })?;

    // 执行逻辑
    if let Err(e) = start(cfg, &easydb_db).await {
        let _ = easydb_db.close();
        return Err(e);
    }

    // 关闭数据库链接
    let _ = easydb_db.close();

    Ok(())
}

async fn start(cfg: &ShowIndexConf, easydb_db: &Pool<MySql>) -> Result<(), CustomError> {
    // 获取集群
    let clusters = MetaClusterDao::find_by_vip_port(easydb_db, &cfg.vip_port)
        .await
        .map_err(|err| {
            CustomError::new(format!(
                "通过vip_port{vip_port}获取所有集群失败 {err}",
                vip_port = &cfg.vip_port,
                err = err.to_string()
            ))
        })?;
    log::info!("获取集群数: {}", clusters.len());

    // 循环每个集群获取master实例信息
    for cluster in clusters.iter() {
        log::info!("集群: {}", utils::string::to_json_str(cluster));

        let masters = InstanceDao::find_master_by_meta_cluster_id(easydb_db, cluster.id.unwrap())
            .await
            .map_err(|err| {
                CustomError::new(format!(
                    "通过集群id获取master失败. {}. {}",
                    utils::string::to_json_str(cluster),
                    err.to_string()
                ))
            })?;
        log::info!("获取到master数: {}", masters.len());

        // 循环master获取 show index 信息
        for master in masters.iter() {
            log::info!("master信息: {}", utils::string::to_json_str(master));
            let show_index_infos_enum = match get_master_show_index_infos(cfg, master).await {
                Ok(v) => v,
                Err(err) => {
                    log::error!("获取表 show index 信息出错. {}", err.to_string());
                    continue;
                }
            };

            let table = match show_index_infos_enum {
                ShowIndexInfosEnum::Normal(infos) => get_print_table_normal(&infos),
                ShowIndexInfosEnum::Info8(infos) => get_print_table_mysql8(&infos),
            };

            table.printstd();
        }
    }

    Ok(())
}

async fn get_master_show_index_infos(
    cfg: &ShowIndexConf,
    master: &Instance,
) -> Result<ShowIndexInfosEnum, CustomError> {
    // 链接数据库
    let password = cfg.get_password();
    let db = rdbc::get_db_by_default(
        master.machine_host.as_ref().unwrap(),
        master.port.unwrap() as i16,
        &cfg.username,
        &password,
        "",
        cfg.is_sql_log,
    )
    .await
    .map_err(|e| CustomError::new(format!("创建 master 实例链接出错. {e}", e = e.to_string())))?;

    let infos = if cfg.is_mysql8 {
        ShowIndexInfosEnum::Info8(
            NormalDao::show_index_8(&db, &cfg.show_index_db, &cfg.show_index_table)
                .await
                .map_err(|err| {
                    let _ = db.close();
                    CustomError::new(format!(
                        "执行SHOW INDEX `{}`.`{}` 出错. {}",
                        &cfg.show_index_db,
                        &cfg.show_index_table,
                        err.to_string()
                    ))
                })?,
        )
    } else {
        ShowIndexInfosEnum::Normal(
            NormalDao::show_index(&db, &cfg.show_index_db, &cfg.show_index_table)
                .await
                .map_err(|err| {
                    let _ = db.close();
                    CustomError::new(format!(
                        "执行SHOW INDEX `{}`.`{}` 出错. {}",
                        &cfg.show_index_db,
                        &cfg.show_index_table,
                        err.to_string()
                    ))
                })?,
        )
    };

    Ok(infos)
}

fn get_print_table_normal(infos: &Vec<ShowIndexInfo>) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    // A more complicated way to add a row:
    table.set_titles(Row::new(vec![
        Cell::new("Table"),
        Cell::new("Non_unique"),
        Cell::new("Key_name"),
        Cell::new("Seq_in_index"),
        Cell::new("Column_name"),
        Cell::new("Collation"),
        Cell::new("Cardinality"),
        Cell::new("Sub_part"),
        Cell::new("Packed"),
        Cell::new("Null"),
        Cell::new("Index_type"),
        Cell::new("Comment"),
        Cell::new("Index_comment"),
    ]));

    for info in infos.iter() {
        let mut info_vec = Vec::new();
        info_vec.push(Cell::new(
            info.table
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.non_unique.unwrap().to_string()));
        info_vec.push(Cell::new(
            info.key_name
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.seq_in_index.unwrap().to_string()));
        info_vec.push(Cell::new(
            info.column_name
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.collation
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.cardinality.unwrap().to_string()));
        info_vec.push(Cell::new(
            info.sub_part
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.packed
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.null
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.index_type
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.comment
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.index_comment
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        table.add_row(Row::new(info_vec));
    }

    table
}

fn get_print_table_mysql8(infos: &Vec<ShowIndexInfo8>) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    // A more complicated way to add a row:
    table.set_titles(Row::new(vec![
        Cell::new("Table"),
        Cell::new("Non_unique"),
        Cell::new("Key_name"),
        Cell::new("Seq_in_index"),
        Cell::new("Column_name"),
        Cell::new("Collation"),
        Cell::new("Cardinality"),
        Cell::new("Sub_part"),
        Cell::new("Packed"),
        Cell::new("Null"),
        Cell::new("Index_type"),
        Cell::new("Comment"),
        Cell::new("Index_comment"),
    ]));

    for info in infos.iter() {
        let mut info_vec = Vec::new();
        info_vec.push(Cell::new(
            info.table
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.non_unique.unwrap().to_string()));
        info_vec.push(Cell::new(
            info.key_name
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.seq_in_index.unwrap().to_string()));
        info_vec.push(Cell::new(
            info.column_name
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.collation
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.cardinality.unwrap().to_string()));
        info_vec.push(Cell::new(
            info.sub_part
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.packed
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.null
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.index_type
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.comment
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.index_comment
                .clone()
                .or(Some(String::from("NULL")))
                .as_ref()
                .unwrap(),
        ));
        table.add_row(Row::new(info_vec));
    }

    table
}
