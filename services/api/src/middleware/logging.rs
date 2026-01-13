use tracing_actix_web::{TracingLogger, DefaultRootSpanBuilder};

pub type Logger = TracingLogger<DefaultRootSpanBuilder>;

pub fn create_logger() -> Logger {
    TracingLogger::default()
}
