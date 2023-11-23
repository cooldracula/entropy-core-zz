use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

/// The log output format that the application should use.
#[derive(Clone, Default, Debug, clap::ValueEnum)]
pub enum Logger {
    #[default]
    Full,
    Pretty,
    Json,
}

impl Logger {
    /// Configures and initializes the global `tracing` Subscriber.
    pub fn setup(&self) {
        // We set up the logger to only print out logs of `INFO` or higher by default, otherwise we
        // fall back to the user's `RUST_LOG` settings.
        let stdout = tracing_subscriber::fmt::layer();
        let env_filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy();
        let registry = tracing_subscriber::registry().with(env_filter);

        match self {
            Logger::Full => registry.with(stdout).init(),
            Logger::Pretty => registry.with(stdout.pretty()).init(),
            Logger::Json => registry.with(stdout.json()).init(),
        }
    }
}

impl std::fmt::Display for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let logger = match self {
            Logger::Full => "full",
            Logger::Pretty => "pretty",
            Logger::Json => "json",
        };
        write!(f, "{}", logger)
    }
}
