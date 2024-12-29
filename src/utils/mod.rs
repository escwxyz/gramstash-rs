pub mod error;
pub mod http;
pub mod redis;

// pub async fn cleanup_old_files(storage_path: PathBuf, max_age_hours: u64) -> Result<(), BotError> {
//     info!("Cleaning up old files...");
//     let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours as i64);

//     let mut entries = fs::read_dir(storage_path)
//         .await
//         .map_err(|e| BotError::NetworkError(format!("Failed to read directory: {}", e)))?;

//     while let Ok(Some(entry)) = entries.next_entry().await {
//         if let Ok(metadata) = entry.metadata().await {
//             if let Ok(modified) = metadata.modified() {
//                 let modified = chrono::DateTime::<chrono::Utc>::from(modified);
//                 if modified < cutoff {
//                     if let Err(e) = fs::remove_file(entry.path()).await {
//                         log::warn!("Failed to remove old file {}: {}", entry.path().display(), e);
//                     }
//                 }
//             }
//         }
//     }

//     Ok(())
// }
