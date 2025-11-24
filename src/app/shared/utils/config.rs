use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    // pub worker_enabled: bool,
    pub database_url: String,
    // pub rabbitmq_url: String,
    // pub rabbitmq_queue: String,
    // pub redis_url: String,
    // pub workspace_dir: String,
    // pub max_concurrent_builds: usize,
    // pub build_timeout_seconds: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // let worker_enabled = env::var("WORKER_ENABLED").ok().unwrap_or_else(|| "true".into()) == "true";
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://user:password@localhost/dbname".into());
        // let rabbitmq_url = env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());
        // let rabbitmq_queue = env::var("RABBITMQ_QUEUE").unwrap_or_else(|_| "builds".into());
        // let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into());
        // let workspace_dir = env::var("WORKSPACE_DIR").unwrap_or_else(|_| "/tmp/bybu-builds".into());
        // let max_concurrent_builds: usize = env::var("MAX_CONCURRENT_BUILDS").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
        // let build_timeout_seconds: u64 = env::var("BUILD_TIMEOUT_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(3600);

        Self {
            // worker_enabled,
            database_url,
            // rabbitmq_url,
            // rabbitmq_queue,
            // redis_url,
            // workspace_dir,
            // max_concurrent_builds,
            // build_timeout_seconds,
        }
    }
}


