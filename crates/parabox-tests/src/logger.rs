use tracing::dispatcher::DefaultGuard;
use tracing_subscriber::fmt;

pub fn init_logger() -> DefaultGuard {
    let subscriber = fmt::fmt()
        .without_time()
        .with_max_level(tracing::Level::TRACE)
        .with_target(false)
        .with_test_writer()
        .pretty()
        .finish();
    tracing::subscriber::set_default(subscriber)
}
