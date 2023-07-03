use crate::config::show_index_conf::ShowIndexConf;
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
    --all \
    --product-instance-duration=21600 \
    --output-dir="./processlist_files" \
    --clear-file-duration=172800 \
    --ignore-instances="localhost:3306" \
    --ignore-instances="localhost:3307" \
    --is-sql-log \
    --log-file="logs/show_processlist.log" \
    --log-level="info"
    "#
    )]
    ShowProcesslist(ShowProcesslistConf),

    #[command(
        about = "执行 SHOW INDEX 语句",
        long_about = r#"
示例:
./target/release/mysql-tool-rs show-index \
    --username="root" \
    --password="NHJtbG91cVdmVjIxTWpLTLr7hJX88U1EC1maABXZJoI=" \
    --easydb-host="127.0.0.1" \
    --easydb-port=3306 \
    --easydb-username="yh_easydb" \
    --easydb-password="WmlPc3JSY295bTduTUFVZElpx3Z5jRDQHK4vz9T65kQ6Zkz4j/08nnapTpEqATmc" \
    --easydb-database="easydb" \
    --vip-port="127.0.0.1:3306" \
    --show-index-db="db1" \
    --show-index-table="table1" \
    --is-sql-log \
    --log-level="info"
"#
    )]
    ShowIndex(ShowIndexConf),
}
