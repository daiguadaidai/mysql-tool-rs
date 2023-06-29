use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, Clone)]
pub struct MetaCluster {
    #[sqlx(default)]
    pub id: Option<i64>,
    #[sqlx(default)]
    pub name: Option<String>,
    #[sqlx(default)]
    pub cluster_id: Option<String>,
    #[sqlx(default)]
    pub is_deleted: Option<i8>,
    #[sqlx(default)]
    pub created_at: Option<NaiveDateTime>,
    #[sqlx(default)]
    pub updated_at: Option<NaiveDateTime>,
    #[sqlx(default)]
    pub business_line: Option<String>,
    #[sqlx(default)]
    pub owner: Option<String>,
    #[sqlx(default)]
    pub domain_name: Option<String>,
    #[sqlx(default)]
    pub vip_port: Option<String>,
    #[sqlx(default)]
    pub vpcgw_vip_port: Option<String>,
    #[sqlx(default)]
    pub is_shard: Option<i8>,
    #[sqlx(default)]
    pub category: Option<i8>,
    #[sqlx(default)]
    pub set_name: Option<String>,
    #[sqlx(default)]
    pub shard_type: Option<String>,
    #[sqlx(default)]
    pub read_host_port: Option<String>,
    #[sqlx(default)]
    pub status: Option<i8>,
}
