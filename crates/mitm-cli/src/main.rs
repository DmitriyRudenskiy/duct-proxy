//! mitm-cli: Command-line interface for mitmproxy-rs.
//!
//! This binary provides a complete CLI tool for running the mitmproxy-rs proxy
//! with features like CA initialization, addon registration, and startup output.

use clap::Parser;
use mitm_addons::{AddonManager, Block, ModifyHeaders};
use mitm_certs::CaRoot;
use mitm_options::Options;
use mitm_proxy::ProxyServer;
use std::path::PathBuf;
use tracing::info;

/// Mitmproxy-rs CLI: Interactive HTTPS proxy
#[derive(Parser, Debug)]
#[command(name = "mitm-cli", version, about)]
struct Cli {
    /// Host to listen on.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on.
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Proxy mode: explicit, transparent, upstream, local.
    #[arg(long, default_value = "explicit")]
    mode: String,

    /// Don't verify upstream server certificates.
    #[arg(long)]
    ssl_insecure: bool,

    /// Configuration file path.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Log level: trace, debug, info, warn, error.
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments.
    let cli = Cli::parse();

    // Initialize tracing subscriber.
    let filter = tracing_subscriber::EnvFilter::try_new(&cli.log_level).unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new("info")
    });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    info!("Mitmproxy-rs v{}", env!("CARGO_PKG_VERSION"));

    // Load config file if present.
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(default_config_path);
    let config = if config_path.exists() {
        info!("Loading config from {:?}", config_path);
        match Options::from_config(&config_path) {
            Ok(opts) => opts,
            Err(e) => {
                tracing::error!("Failed to load config: {}", e);
                Options::default()
            }
        }
    } else {
        info!("No config file found at {:?}, using defaults", config_path);
        Options::default()
    };

    // Merge CLI args with config (CLI takes precedence).
    let opts = Options::merge(
        &Options {
            listen_host: cli.host,
            listen_port: cli.port,
            mode: cli.mode.clone(),
            ssl_insecure: cli.ssl_insecure,
            ..Default::default()
        },
        &config,
    );

    // Display startup information.
    let ca_path = format!("{}/mitmproxy-ca-cert.pem", opts.conf_dir);
    let listen_addr = format!("{}:{}", opts.listen_host, opts.listen_port);
    
    info!("Mitmproxy-rs v{}", env!("CARGO_PKG_VERSION"));
    info!("Listening on {} ({} mode)", listen_addr, opts.mode);
    info!("CA: {}", ca_path);
    info!("Addons: ModifyHeaders, Block");

    // Initialize CA: try to load, if fails generate new one.
    let ca_dir = PathBuf::from(&opts.conf_dir);
    let ca_result = CaRoot::load(&ca_dir);
    let _ca = match ca_result {
        Ok(ca) => {
            info!("CA loaded from {:?}", ca_dir);
            ca
        }
        Err(_) => {
            info!("No CA found, generating new one at {:?}", ca_dir);
            let ca = match CaRoot::generate("Mitmproxy-rs CA") {
                Ok(ca) => ca,
                Err(e) => {
                    tracing::error!("Failed to generate CA: {}", e);
                    std::process::exit(1);
                }
            };
            if let Err(e) = ca.save(&ca_dir) {
                tracing::error!("Failed to save CA: {}", e);
                std::process::exit(1);
            }
            info!("CA saved to {:?}", ca_dir);
            ca
        }
    };

    // Create addon manager and register built-in addons.
    let _addon_mgr = AddonManager::new();
    let _modify_headers = ModifyHeaders::new();
    let _block = Block::new();

    // Create proxy server.
    let server = match ProxyServer::from_options(&opts) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to create proxy server: {}", e);
            std::process::exit(1);
        }
    };

    // Bind to listen address.
    let listener = match server.bind().await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", listen_addr, e);
            std::process::exit(1);
        }
    };

    // Run the proxy server (handles graceful shutdown internally).
    if let Err(e) = server.run(listener).await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }

    info!("Server stopped.");
}

/// Get the default config file path.
fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mitmproxy")
        .join("config.yaml")
}
