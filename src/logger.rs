use error_chain::error_chain;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SetLogger(log::SetLoggerError);
    }
}

pub fn init() -> () {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("output.log");

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile.unwrap())))
        .build(Root::builder()
                   .appender("logfile")
                   .build(LevelFilter::Info));

    log4rs::init_config(config.unwrap());

    log::info!("Log initialized");

    ()
}
