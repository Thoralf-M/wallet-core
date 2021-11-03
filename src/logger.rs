use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
pub use log::LevelFilter;

pub fn init_logger(filename: &str, levelfilter: LevelFilter) -> crate::Result<()> {
    let output_config = LoggerOutputConfigBuilder::new()
        .name(filename)
        .level_filter(levelfilter);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config)?;
    Ok(())
}
