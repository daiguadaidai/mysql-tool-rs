use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, Clone)]
pub struct ShowProcesslistInfo {
    #[sqlx(default, rename = "ID")]
    pub id: Option<u64>,
    #[sqlx(default, rename = "USER")]
    pub user: Option<String>,
    #[sqlx(default, rename = "HOST")]
    pub host: Option<String>,
    #[sqlx(default, rename = "DB")]
    pub db: Option<String>,
    #[sqlx(default, rename = "COMMAND")]
    pub command: Option<String>,
    #[sqlx(default, rename = "TIME")]
    pub time: Option<i32>,
    #[sqlx(default, rename = "STATE")]
    pub state: Option<String>,
    #[sqlx(default, rename = "INFO")]
    pub info: Option<String>,
}
