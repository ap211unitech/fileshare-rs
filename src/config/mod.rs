use std::env;

pub struct Env {
    pub server_url: String,
}

pub struct AppConfig {
    pub env: Env,
}

impl AppConfig {
    pub fn load_config() -> Self {
        dotenvy::dotenv().expect("Unable to access .env file!");

        let env = Env {
            server_url: env::var("SERVER_URL").unwrap_or("127.0.0.1:8000".to_string()),
        };

        AppConfig { env }
    }
}
