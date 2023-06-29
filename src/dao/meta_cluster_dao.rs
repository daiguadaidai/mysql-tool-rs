use crate::models::MetaCluster;
use sqlx::{Error, MySql, Pool};

pub struct MetaClusterDao;

impl MetaClusterDao {
    // 通过 vip_port 获取集群信息
    pub async fn get_by_vip_port(pool: &Pool<MySql>, vip_port: &str) -> Result<MetaCluster, Error> {
        let query = r#"
SELECT *
FROM meta_cluster
WHERE is_deleted = 0
    AND vip_port = ?
    "#;

        sqlx::query_as::<_, MetaCluster>(query)
            .bind(vip_port)
            .fetch_one(pool)
            .await
    }

    // 通过 vip_port 获取所有的 vip_prot 集群
    pub async fn find_by_vip_port(
        pool: &Pool<MySql>,
        vip_port: &str,
    ) -> Result<Vec<MetaCluster>, Error> {
        let query = format!(
            "SELECT * FROM meta_cluster WHERE is_deleted = 0 AND vip_port = {vip_port:?}",
            vip_port = vip_port
        );

        sqlx::query_as::<_, MetaCluster>(&query)
            .fetch_all(pool)
            .await
    }
}
