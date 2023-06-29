use crate::models::Instance;
use sqlx::{Error, MySql, Pool};

pub struct InstanceDao;

impl InstanceDao {
    // 通过集群id, 获取 instance 信息
    pub async fn find_by_meta_cluster_id(
        pool: &Pool<MySql>,
        meta_cluster_id: i64,
    ) -> Result<Vec<Instance>, Error> {
        let query = r#"
SELECT *
FROM instance
WHERE is_deleted = 0
    AND meta_cluster_id = ?
    "#;

        sqlx::query_as::<_, Instance>(query)
            .bind(meta_cluster_id)
            .fetch_all(pool)
            .await
    }

    // 通过集群获取master
    pub async fn find_master_by_meta_cluster_id(
        pool: &Pool<MySql>,
        meta_cluster_id: i64,
    ) -> Result<Vec<Instance>, Error> {
        let query = format!(
            "SELECT * FROM instance WHERE is_deleted = 0 AND meta_cluster_id = {meta_cluster_id} AND role='master'",
            meta_cluster_id = meta_cluster_id
        );

        sqlx::query_as::<_, Instance>(&query).fetch_all(pool).await
    }
}
