use anyhow::Context;
use sea_orm::{Database, DatabaseConnection};

use crate::objstore::{Config, ObjStore};


pub mod api;
pub mod objstore;
// pub mod db;
pub mod entities;
#[derive(Clone, Debug)]
pub struct State {
    pub db_conn: DatabaseConnection,
    pub obj_store: ObjStore 
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let db_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(db_url).await?;

    let s3_secret = std::env::var("S3_SECRET_KEY")?;
    let s3_access_id = std::env::var("S3_ACCESS_ID")?;
    let s3_bucket = std::env::var("S3_BUCKET")?;
    let s3_region = std::env::var("S3_REGION")?;
    let s3_endpoint = std::env::var("S3_ENDPOINT")?;

    let o_store = ObjStore::new(
        Config{
            region: s3_region,
            endpoint: s3_endpoint,
            bucket_name: s3_bucket, 
            secret_key: s3_secret, 
            access_id: s3_access_id, 
        }
    )?;

    db.get_schema_registry(module_path!().split("::").next().unwrap())
        .sync(&db)
        .await?;

    let state = State { db_conn: db, obj_store: o_store };
    let app = api::create_app(state).await?;
    let address = String::from("127.0.0.1:8080");
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .context("Failed to listen")?;

    tracing::info!("Listening on http://{address}");

    axum::serve(listener, app)
        .await
        .context("Failed to serve app")?;

    // TODO: close connection on graceful shutdown

    Ok(())
}
