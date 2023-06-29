use crate::models::{ShowIndexInfo, ShowProcesslistInfo};
use sqlx::{Error, MySql, Pool};

pub struct NormalDao;

impl NormalDao {
    // 获取 processlist 信息
    pub async fn show_processlist(pool: &Pool<MySql>) -> Result<Vec<ShowProcesslistInfo>, Error> {
        let query = r#"
SELECT
 ID
 , USER
 , HOST
 , DB
 , COMMAND
 , TIME
 , STATE
 , INFO
FROM information_schema.PROCESSLIST;
    "#;

        sqlx::query_as::<_, ShowProcesslistInfo>(query)
            .fetch_all(pool)
            .await
    }

    // 执行 show index 语句
    pub async fn show_index(
        pool: &Pool<MySql>,
        db_name: &str,
        table_name: &str,
    ) -> Result<Vec<ShowIndexInfo>, Error> {
        let query = format!(
            "SHOW INDEX FROM `{db_name}`.`{table_name}`;",
            db_name = db_name,
            table_name = table_name
        );

        sqlx::query_as::<_, ShowIndexInfo>(&query)
            .fetch_all(pool)
            .await
    }
}
