use crate::config::{Commands, Config};
use crate::error::CustomError;
use clap::Parser;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

mod config;
mod core;
mod dao;
mod error;
mod models;
mod rdbc;
mod utils;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let cfg: Config = Config::parse();

    match &cfg.command {
        Commands::ShowProcesslist(cfg) => {
            init_log(&cfg.log_file, &cfg.log_level)?;
            core::show_processlist::run(cfg).await
        }
        Commands::ShowIndex(cfg) => {
            // 只打印到控制台
            init_log("", &cfg.log_level)?;
            core::show_index::run(cfg).await
        }
    }
}

fn init_log(log_file: &str, log_level: &str) -> Result<(), CustomError> {
    let pattern = "{d(%Y-%m-%d %H:%M:%S)} - {l} - {f}::{L} - {m}{n}";
    // 获取日志写入文件配置
    let file_appender = get_file_appender(log_file, log_level, pattern)?;

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    // 一定会打印控制台
    let mut root_builder = Root::builder().appender("stdout");
    if file_appender.is_some() {
        root_builder = root_builder.appender("file")
    }

    let root = match log_level {
        "info" => root_builder.build(LevelFilter::Info),
        "debug" => root_builder.build(LevelFilter::Debug),
        &_ => root_builder.build(LevelFilter::Info),
    };

    let mut config_builder = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(console_appender)));
    if file_appender.is_some() {
        config_builder = config_builder
            .appender(Appender::builder().build("file", Box::new(file_appender.unwrap())));
    }
    let config = config_builder.build(root).map_err(|e| {
        CustomError::new(format!(
            "生成日志 config 出错. log_file: {log_file}, log_level: {log_level}. err: {e}",
            log_file = log_file,
            log_level = log_level,
            e = e.to_string()
        ))
    })?;

    let _ = log4rs::init_config(config)?;

    Ok(())
}

// 获取写文件 appender
fn get_file_appender(
    log_file: &str,
    log_level: &str,
    pattern: &str,
) -> Result<Option<FileAppender>, CustomError> {
    if log_file.is_empty() {
        return Ok(None);
    }

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(log_file)
        .map_err(|e| {
            CustomError::new(format!(
                "生成日志 file_appender 出错. log_file: {log_file}, log_level: {log_level}, err: {e}",
                log_file = log_file,
                log_level = log_level,
                e = e.to_string(),
            ))
        })?;

    Ok(Some(file_appender))
}
