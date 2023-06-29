use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct ShowIndexInfo {
    #[sqlx(default, rename = "Table")]
    pub table: Option<String>,
    #[sqlx(default, rename = "Non_unique")]
    pub non_unique: Option<i32>,
    #[sqlx(default, rename = "Key_name")]
    pub key_name: Option<String>,
    #[sqlx(default, rename = "Seq_in_index")]
    pub seq_in_index: Option<i64>,
    #[sqlx(default, rename = "Column_name")]
    pub column_name: Option<String>,
    #[sqlx(default, rename = "Collation")]
    pub collation: Option<String>,
    #[sqlx(default, rename = "Cardinality")]
    pub cardinality: Option<i64>,
    #[sqlx(default, rename = "Sub_part")]
    pub sub_part: Option<String>,
    #[sqlx(default, rename = "Packed")]
    pub packed: Option<String>,
    #[sqlx(default, rename = "Null")]
    pub null: Option<String>,
    #[sqlx(default, rename = "Index_type")]
    pub index_type: Option<String>,
    #[sqlx(default, rename = "Comment")]
    pub comment: Option<String>,
    #[sqlx(default, rename = "Index_comment")]
    pub index_comment: Option<String>,
}
