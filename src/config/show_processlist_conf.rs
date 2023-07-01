use crate::error::CustomError;
use crate::utils::peep;
use clap::Args;
use serde::{Deserialize, Serialize};

const DEFAULT_USERNAME: &str = "root";
const DEFAULT_PASSWORD: &str = "NHJtbG91cVdmVjIxTWpLTLr7hJX88U1EC1maABXZJoI=";
const DEFAULT_HOST: &str = "";
const DEFAULT_PORT: u16 = 0;
const DEFAULT_EASYDB_USERNAME: &str = "yh_easydb";
const DEFAULT_EASYDB_PASSWORD: &str =
    "WmlPc3JSY295bTduTUFVZElpx3Z5jRDQHK4vz9T65kQ6Zkz4j/08nnapTpEqATmc";
const DEFAULT_EASYDB_HOST: &str = "127.0.0.1";
const DEFAULT_EASYDB_PORT: u16 = 3306;
const DEFAULT_EASYDB_DATABASE: &str = "easydb";
const DEFAULT_IS_SQL_LOG: bool = false;
const DEFAULT_LOG_FILE_SHOW_PROCESSLIT: &str = "logs/show_processlist.log";
const DEFAULT_LOG_LEVEL_SHOW_PROCESSLIT: &str = "info";
const DEFAULT_VIP_PORT_SHOW_PROCESSLIT: &str = "";
const DEFAULT_ALL: bool = false;
const DEFAULT_OUTPUT_DIR: &str = "./processlist_files";
const DEFAULT_PRODUCT_INSTANCE_DURATION: u64 = 6 * 60 * 60;
const DEFAULT_SLEEP_SHOW_PROCESSLIT: u64 = 1000; // 单位毫秒
const DEFAULT_PRINT_CNT_THRESHOLD: u64 = 50;

#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct ShowProcesslistConf {
    #[arg(long, default_value_t = String::from(DEFAULT_USERNAME), help = "数据库用户名")]
    pub username: String,
    #[arg(long, default_value_t = String::from(DEFAULT_PASSWORD), help = "数据库密码")]
    pub password: String,
    #[arg(long, default_value_t = String::from(DEFAULT_HOST), help = "需要执行 show processlist 数据库host")]
    pub host: String,
    #[arg(long, default_value_t = DEFAULT_PORT, help = "需要执行 show processlist 数据库端口")]
    pub port: u16,
    #[arg(long, default_value_t = String::from(DEFAULT_VIP_PORT_SHOW_PROCESSLIT), help = "需要执行 show processlist 数据库地址, 如果指定了 --host --port 参数则忽略该参数")]
    pub vip_port: String,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_USERNAME), help = "easydb 数据库用户名")]
    pub easydb_username: String,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_PASSWORD), help = "easydb 数据库密码")]
    pub easydb_password: String,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_HOST), help = "easydb 数据库地址")]
    pub easydb_host: String,
    #[arg(long, default_value_t = DEFAULT_EASYDB_PORT, help = "easydb 数据库端口")]
    pub easydb_port: u16,
    #[arg(long, default_value_t = String::from(DEFAULT_EASYDB_DATABASE), help = "easydb链接的数据库名")]
    pub easydb_database: String,
    #[arg(long, default_value_t = DEFAULT_SLEEP_SHOW_PROCESSLIT, help = "循环执行 SHOW PROCESSLIST 中间需要睡眠多久(单位:ms)")]
    pub sleep: u64,
    #[arg(long, default_value_t = DEFAULT_PRINT_CNT_THRESHOLD, help = "SHOW PROCESSLIST返回多少数据需要打印到日志文件")]
    pub print_cnt_threshold: u64,
    #[arg(long, default_value_t = DEFAULT_ALL, help = "所有集群对实例进行 SHOW PROCESSLIST")]
    pub all: bool,
    #[arg(long, default_value_t = String::from(DEFAULT_OUTPUT_DIR), help = "在使用 --all 参数时每个实例的信息输出到哪个目录")]
    pub output_dir: String,
    #[arg(long, default_value_t = DEFAULT_PRODUCT_INSTANCE_DURATION, help = "在指定 --all 参数时多久进行重新生成一次实例(单位:s)")]
    pub product_instance_duration: u64,
    #[arg(long, default_value_t = DEFAULT_IS_SQL_LOG, help = "执行sql是否打印日志")]
    pub is_sql_log: bool,
    #[arg(long, default_value_t = String::from(DEFAULT_LOG_FILE_SHOW_PROCESSLIT), help = "日志文件")]
    pub log_file: String,
    #[arg(long, default_value_t = String::from(DEFAULT_LOG_LEVEL_SHOW_PROCESSLIT), help = "日志级别")]
    pub log_level: String,
}

impl ShowProcesslistConf {
    pub fn check(&self) -> Result<(), CustomError> {
        if self.vip_port.is_empty() && (self.host.is_empty() || self.port <= 0) && !self.all {
            return Err(CustomError::new(format!(
                "没有获取到需要查询的实例信息, 请指定 --vip-port 或 --host --port 参数 或 --all"
            )));
        }

        Ok(())
    }

    // 在使用 --all 参数到时候使用这个检测
    pub fn check_all(&self) -> Result<(), CustomError> {
        if self.all && self.output_dir.is_empty() {
            return Err(CustomError::new(format!(
                "没有指定 output_dir 参数, 在使用 --all 到时候, 需要使用 --output-dir 参数指定信息输出到目录"
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

    pub fn have_host_port(&self) -> bool {
        return !self.host.is_empty() && self.port > 0;
    }

    pub fn have_vip_port(&self) -> bool {
        return !self.vip_port.is_empty();
    }
}
