use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, Clone)]
pub struct Instance {
    #[sqlx(default)]
    pub id: Option<i64>,
    #[sqlx(default)]
    pub is_deleted: Option<i8>,
    #[sqlx(default)]
    pub created_at: Option<NaiveDateTime>,
    #[sqlx(default)]
    pub updated_at: Option<NaiveDateTime>,
    #[sqlx(default)]
    pub instance_id: Option<String>, // 实例ID
    #[sqlx(default)]
    pub instance_name: Option<String>, // 实例名称
    #[sqlx(default)]
    pub meta_cluster_id: Option<i64>, // 集群id
    #[sqlx(default)]
    pub machine_host: Option<String>, // 机器host
    #[sqlx(default)]
    pub vpcgw_rip: Option<String>, // 机器host
    #[sqlx(default)]
    pub port: Option<i32>, // 端口
    #[sqlx(default)]
    pub role: Option<String>, // 实例角色: master, slave, backup
    #[sqlx(default)]
    pub cpu: Option<i32>, // 核数
    #[sqlx(default)]
    pub mem: Option<i32>, // 内存大小, 单位G
    #[sqlx(default)]
    pub disk: Option<i32>, // 磁盘容量, 单位G
    #[sqlx(default)]
    pub master_host: Option<String>, // 主库host
    #[sqlx(default)]
    pub master_port: Option<i32>, // 主库端口
    #[sqlx(default)]
    pub version: Option<String>, // 实例版本
    #[sqlx(default)]
    pub is_maintenance: Option<i8>, // 是否在维护中: 0:否, 1:是
    #[sqlx(default)]
    pub descript: Option<String>, // 描述
    #[sqlx(default)]
    pub vip_port: Option<String>, // VIP 和端口
    #[sqlx(default)]
    pub vpcgw_vip_port: Option<String>, // VPC GateWay VIP和端口
    #[sqlx(default)]
    pub set_name: Option<String>, // set 名称
}
