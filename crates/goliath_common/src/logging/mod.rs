use log::LevelFilter;
use std::io::Write;

pub fn setup_logger() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("rustls", LevelFilter::Off)
        .filter_module("tungstenite", LevelFilter::Off)
        .filter_module("tokio_tungstenite", LevelFilter::Off)
        .parse_default_env()
        .format(|fmt, record| {
            let current_time = time::OffsetDateTime::now_utc();
            writeln!(
                fmt,
                "[{}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}Z {}:{}] ({}) {}",
                current_time.year(),
                current_time.month() as u8,
                current_time.day(),
                current_time.hour(),
                current_time.minute(),
                current_time.second(),
                record.module_path_static().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .init();
}

#[cfg(test)]
mod tests {
    use crate::logging::setup_logger;

    #[test]
    fn test_setup_loggers() {
        setup_logger(); // Does not panic, great test
    }
}
