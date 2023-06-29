use crate::config::show_processlist_conf::ShowProcesslistConf;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "mysql 相关定时任务", long_about = None)]
pub struct Config {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(
        about = "获取 processlist 信息",
        long_about = r#"
示例:
./target/release/mysql-tool-rs show-processlist \
    --username="root" \
    --password="NHJtbG91cVdmVjIxTWpLTLr7hJX88U1EC1maABXZJoI=" \
    --host="" \
    --port=0 \
    --vip-port="127.0.0.1:3306" \
    --easydb-host="127.0.0.1" \
    --easydb-port=3306 \
    --easydb-username="yh_easydb" \
    --easydb-password="WmlPc3JSY295bTduTUFVZElpx3Z5jRDQHK4vz9T65kQ6Zkz4j/08nnapTpEqATmc" \
    --easydb-database="easydb" \
    --sleep=1000 \
    --print-cnt-threshold=50 \
    --is-sql-log \
    --log-file="logs/show_processlist.log" \
    --log-level="info"
    "#
    )]
    ShowProcesslist(ShowProcesslistConf),
}
