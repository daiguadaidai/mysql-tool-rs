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
    }
}

fn init_log(log_file: &str, log_level: &str) -> Result<(), CustomError> {
    let pattern = "{d(%Y-%m-%d %H:%M:%S)} - {l} - {f}::{L} - {m}{n}";
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

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let root = match log_level {
        "info" => Root::builder()
            .appender("stdout")
            .appender("file")
            .build(LevelFilter::Info),
        "debug" => Root::builder()
            .appender("stdout")
            .appender("file")
            .build(LevelFilter::Debug),
        &_ => Root::builder()
            .appender("stdout")
            .appender("file")
            .build(LevelFilter::Info),
    };

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(console_appender)))
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(root)
        .map_err(|e| {
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
