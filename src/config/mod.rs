use mongodb::{bson::doc, options::IndexOptions, Collection, Database, IndexModel};
use std::env;

use crate::models::{token::TokenCollection, user::UserCollection};

pub struct AppConfig {
    pub server_url: String,
    pub mongodb_url: String,
    pub sendgrid_api_key: String,
    pub sendgrid_sender_name: String,
    pub sendgrid_sender_email: String,
    pub jwt_secret_key: String,
    pub cloudinary_cloud_name: String,
    pub cloudinary_api_key: String,
    pub cloudinary_api_secret: String,
}

#[derive(Clone)]
pub struct AppState {
    pub user_collection: Collection<UserCollection>,
    pub token_collection: Collection<TokenCollection>,
}

impl AppConfig {
    pub fn load_config() -> Self {
        dotenvy::dotenv().expect("Unable to access .env file!");

        let app_config = AppConfig {
            server_url: env::var("SERVER_URL").unwrap_or("127.0.0.1:8000".to_string()),
            mongodb_url: env::var("MONGODB_URL").expect("MONGODB_URL not found in .env"),
            sendgrid_api_key: env::var("SENDGRID_API_KEY")
                .expect("SENDGRID_API_KEY not found in .env"),
            sendgrid_sender_name: env::var("SENDGRID_SENDER_NAME")
                .expect("SENDGRID_SENDER_NAME not found in .env"),
            sendgrid_sender_email: env::var("SENDGRID_SENDER_EMAIL")
                .expect("SENDGRID_SENDER_EMAIL not found in .env"),
            jwt_secret_key: env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY not found in .env"),
            cloudinary_cloud_name: env::var("CLOUDINARY_CLOUD_NAME")
                .expect("CLOUDINARY_CLOUD_NAME not found in .env"),
            cloudinary_api_key: env::var("CLOUDINARY_API_KEY")
                .expect("CLOUDINARY_API_KEY not found in .env"),
            cloudinary_api_secret: env::var("CLOUDINARY_API_SECRET")
                .expect("CLOUDINARY_API_SECRET not found in .env"),
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

        // Set up indexes BEFORE using collections
        Self::get_user_collection(&db).await.unwrap();
        Self::get_token_collection(&db).await.unwrap();

        let user_collection = db.collection::<UserCollection>("users");
        let token_collection = db.collection::<TokenCollection>("tokens");

        AppState {
            user_collection,
            token_collection,
        }
    }

    async fn get_token_collection(db: &Database) -> mongodb::error::Result<()> {
        let token_collection = db.collection::<TokenCollection>("tokens");

        let index_model = IndexModel::builder()
            .keys(doc! { "hashed_token": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .background(false) // Make sure we wait until it's done
                    .build(),
            )
            .build();

        token_collection.create_index(index_model).await?;

        Ok(())
    }

    async fn get_user_collection(db: &Database) -> mongodb::error::Result<()> {
        let user_collection = db.collection::<UserCollection>("users");

        let index_model = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .background(false) // Make sure we wait until it's done
                    .build(),
            )
            .build();

        user_collection.create_index(index_model).await?;

        Ok(())
    }
}
