use crate::models::ShowProcesslistInfo;
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
}
