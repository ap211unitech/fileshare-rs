use mongodb::Collection;
use std::env;

use crate::models::user::UserCollection;

pub struct AppConfig {
    pub server_url: String,
    pub mongodb_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub user_collection: Collection<UserCollection>,
}

impl AppConfig {
    pub fn load_config() -> Self {
        dotenvy::dotenv().expect("Unable to access .env file!");

        let app_config = AppConfig {
            server_url: env::var("SERVER_URL").unwrap_or("127.0.0.1:8000".to_string()),
            mongodb_url: env::var("MONGODB_URL").expect("MONGODB_URL not found in .env"),
        };

        app_config
    }
}

impl AppState {
    pub async fn load_state() -> Self {
        let app_config = AppConfig::load_config();

        let db = mongodb::Client::with_uri_str(app_config.mongodb_url)
            .await
            .unwrap()
            .database("fileshare-rs");

        let user_collection = db.collection::<UserCollection>("user");

        AppState { user_collection }
    }
}
