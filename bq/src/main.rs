use clap::{Parser, Subcommand};
use engine::config::{Config, Strategy};
use tokio::{
    fs,
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

/// Initializes the tracing system for the application.
///
/// This function sets up the tracing_subscriber to output logs in JSON format, set the max log level to INFO, and disable
/// including the current span information. The tracing system is then initialized with these settings.
///
/// Usage procedures are initialized by calling this function at the start of the main function in the application.
/// After the call, tracing system will be configured and ready to use within the scope of the application.
fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_current_span(false)
        .init();
}

#[derive(Parser)]
#[command(name = "BQ")]
#[command(author, version, about, long_about = None)]
struct Client {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start a trade engine.")]
    Run {
        #[arg(short, long, default_value = "./config.toml", value_name = "FILE")]
        config: String,
    },
    #[command(about = "Inject id for config file.")]
    Inject {
        #[arg(short, long, default_value = "./config.toml", value_name = "FILE")]
        config: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Client::parse();
    match cli.command {
        Commands::Run { config } => {
            run_engine(config).await;
        }
        Commands::Inject { config } => {
            inject_id_with_config(config).await;
        }
    }
}

async fn run_engine(config: String) {
    init_tracing();

    let mut file = File::open(config).await.expect("config.toml not exist");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .await
        .expect("read config.toml failed");

    let conf: Config = toml::from_str(&str).expect("msg");

    let mut e = engine::Engine::new_with_env(conf);
    tracing::info!("Engine started");
    e.run().await;
}

async fn inject_id_with_config(config: String) {
    let mut file = File::open(&config).await.expect("config.toml not exist");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .await
        .expect("read config.toml failed");
    let mut conf: Config = toml::from_str(&str).expect("msg");

    for instance in conf.instances.iter_mut() {
        if instance.id.is_empty() {
            instance.id = uuid::Uuid::new_v4().to_string();
        }
        for strategy in instance.strategies.iter_mut() {
            match strategy {
                Strategy::Rsi { ref mut id, .. } => {
                    if id.is_empty() {
                        *id = uuid::Uuid::new_v4().to_string();
                    }
                }
                Strategy::Atr { ref mut id, .. } => {
                    if id.is_empty() {
                        *id = uuid::Uuid::new_v4().to_string();
                    }
                }
            }
        }
    }
    let mut file = File::create(&config).await.expect("config.toml not exist");

    let cf = toml::to_string(&conf).expect("");
    file.write_all(cf.as_bytes()).await.expect("");
    file.flush().await.expect("");
}
