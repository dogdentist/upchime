pub mod config;
pub mod constants;
pub mod db;
pub mod dispatch;
pub mod log;
pub mod pinger;

enum ExitErrorCode {
    Config = 1,
    App = 2,
}

impl ExitErrorCode {
    fn exit(self) {
        std::process::exit(self as i32);
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = config::load() {
        errorln!("failed to initialize the config, error: {}", e.to_string());
        ExitErrorCode::Config.exit();
    }

    log::initialize();

    outputln!("starting");

    if let Err(e) = dispatch::start().await {
        errorln!("runtime error on the app, error: {}", e.to_string());
        ExitErrorCode::App.exit();
    }
}
