use crate::error::CustomError;
use crate::utils::peep;
use clap::Args;
use serde::{Deserialize, Serialize};

const DEFAULT_USERNAME: &str = "root";
const DEFAULT_PASSWORD: &str = "NHJtbG91cVdmVjIxTWpLTLr7hJX88U1EC1maABXZJoI=";
const DEFAULT_EASYDB_USERNAME: &str = "yh_easydb";
const DEFAULT_EASYDB_PASSWORD: &str =
    "WmlPc3JSY295bTduTUFVZElpx3Z5jRDQHK4vz9T65kQ6Zkz4j/08nnapTpEqATmc";
const DEFAULT_EASYDB_HOST: &str = "127.0.0.1";
const DEFAULT_EASYDB_PORT: i16 = 3306;
const DEFAULT_EASYDB_DATABASE: &str = "easydb";
const DEFAULT_VIP_PORT: &str = "";
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_IS_SQL_LOG: bool = false;

#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct ShowIndexConf {
    #[arg(long, default_value_t = String::from(DEFAULT_USERNAME), help = "默认链接所有实例数据库用户名")]
    pub username: String,
    #[arg(long, default_value_t = String::from(DEFAULT_PASSWORD), help = "默认链接所有实例数据库密码")]
    pub password: String,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_USERNAME), help = "easydb 数据库用户名")]
    pub easydb_username: String,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_PASSWORD), help = "easydb 数据库密码")]
    pub easydb_password: String,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_HOST), help = "easydb 数据库地址")]
    pub easydb_host: String,
    #[arg(long, default_value_t = DEFAULT_EASYDB_PORT, help = "easydb 数据库端口")]
    pub easydb_port: i16,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_DATABASE), help = "easydb链接的数据库名")]
    pub easydb_database: String,
    #[arg(long, default_value_t = String::from(DEFAULT_VIP_PORT), help = "需要查询集群的vip_port")]
    pub vip_port: String,
    #[arg(long, help = "执行SHOW INDEX 的数据库名")]
    pub show_index_db: String,
    #[arg(long, help = "执行SHOW INDEX 的表名")]
    pub show_index_table: String,
    #[arg(long, default_value_t = String::from(DEFAULT_LOG_LEVEL), help = "日志级别")]
    pub log_level: String,
    #[arg(long, default_value_t = DEFAULT_IS_SQL_LOG, help = "执行sql是否打印日志")]
    pub is_sql_log: bool,
}

impl ShowIndexConf {
    pub fn check(&self) -> Result<(), CustomError> {
        if self.vip_port.is_empty() {
            return Err(CustomError::new(format!(
                "没有获取到需要查询的实例信息, 请指定 --vip-port"
            )));
        }

        Ok(())
    }

    pub fn get_easydb_dsn(&self) -> String {
        let password = peep::decrypt_default(&self.easydb_password);

        return format!(
            "mysql://{username}:{password}@{host}:{port}/{database}",
            username = self.easydb_username,
            password = password,
            host = self.easydb_host,
            port = self.easydb_port,
            database = self.easydb_database,
        );
    }

    pub fn get_password(&self) -> String {
        peep::decrypt_default(&self.password)
    }
}
