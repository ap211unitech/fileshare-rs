use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::doc;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::{config::AppState, utils::cloudinary};

pub async fn auto_delete_file_from_server(app_state: AppState) -> Result<(), JobSchedulerError> {
    let sched = JobScheduler::new().await?;

    // Add basic cron job
    sched
        .add(Job::new(
            "0 */1 * * * *", // Run every 1 min
            move |_uuid, _l| {
                tracing::info!("Running cron job at {}", Utc::now());
                let app_state = app_state.clone();
                tokio::spawn(async move {
                    if let Err(e) = delete_file_from_cloud(app_state).await {
                        tracing::error!("Error in delete_file_from_cloud: {:?}", e);
                    }
                });
            },
        )?)
        .await?;

    // Start the scheduler
    sched.start().await?;

    Ok(())
}

async fn delete_file_from_cloud(app_state: AppState) -> Result<(), JobSchedulerError> {
    let now = Utc::now();
    let doc = doc! {
        "$or": [
            {"expires_at": { "$lt": now.to_string() }},
            {"$expr": { "$gte": ["$download_count", "$max_downloads"] }}
        ]
    };

    let mut files = app_state
        .file_collection
        .find(doc)
        .await
        .map_err(|_| JobSchedulerError::FetchJob)?;

    while let Some(file) = files
        .try_next()
        .await
        .map_err(|_| JobSchedulerError::GetJobData)?
    {
        cloudinary::delete_file_from_cloud(file.cid)
            .await
            .map_err(|_| JobSchedulerError::CantRemove)?;
    }

    Ok(())
}
