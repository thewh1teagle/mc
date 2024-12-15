use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logger() {
    let default_log_directive = format!("{}=INFO", env!("CARGO_PKG_NAME"));
    let rust_log = std::env::var("RUST_LOG").unwrap_or(default_log_directive);
    let indicatif_layer: IndicatifLayer<tracing_subscriber::registry::Registry> =
        IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(EnvFilter::new(rust_log))
        .init();
}
