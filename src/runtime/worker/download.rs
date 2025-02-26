use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use async_trait::async_trait;
use teloxide::{
    adaptors::Throttle,
    payloads::{EditMessageTextSetters, SendMessageSetters, SendPhotoSetters},
    prelude::Requester,
    types::{ChatId, InputFile, MessageId},
    Bot,
};
use tokio::sync::broadcast;

use crate::{
    handler::{get_confirm_download_keyboard, get_download_ask_for_link_keyboard, get_main_menu_keyboard},
    platform::{traits::PlatformCapability, DownloadState, Platform, PlatformInstagram, PostDownloadState},
    runtime::{
        queue::TaskQueueManager,
        task::{DownloadTask, PostDownloadTask},
        RuntimeError,
    },
    state::AppState,
};

use super::Worker;

#[derive(Clone)]
pub struct DownloadWorker {
    name: String,
    concurrency: usize,
    queue_manager: TaskQueueManager,
    bot: Throttle<Bot>,
    shutdown: broadcast::Sender<()>,
    running: Arc<AtomicBool>,
}

impl DownloadWorker {
    pub fn new(name: &str, concurrency: usize, queue_manager: TaskQueueManager, bot: Throttle<Bot>) -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Self {
            name: name.to_string(),
            concurrency,
            queue_manager,
            bot,
            shutdown,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn process_task(&self, task: DownloadTask) -> Result<DownloadState, RuntimeError> {
        let platform_registry = AppState::get()?.platform_registry;
        let telegram_user_id = task.context.user_id.to_string();

        let result = match task.context.platform {
            crate::platform::Platform::Instagram => {
                info!("Processing Instagram download task");
                platform_registry
                    .handle_download::<PlatformInstagram>(&task.context.platform, &task.url, &telegram_user_id)
                    .await
                    .map_err(|e| RuntimeError::TaskError(e.to_string()))?
            }
            _ => {
                error!("Not implemented yet");
                DownloadState::Error
            }
        };

        match result {
            crate::platform::DownloadState::RateLimited => {
                self.bot
                    .edit_message_text(
                        ChatId(task.context.chat_id),
                        MessageId(task.context.message_id),
                        t!("messages.download.download_limit_reached"),
                    )
                    .reply_markup(get_main_menu_keyboard())
                    .await
                    .unwrap();

                Ok(DownloadState::RateLimited)
            }
            crate::platform::DownloadState::Success(media_info) => {
                let queue_manager = &AppState::get().unwrap().runtime.queue_manager;

                queue_manager.add_pending_confirmation(media_info.clone(), task.context.clone());

                let preview_text = media_info.get_preview_text();

                if let Some(thumbnail_url) = &media_info.thumbnail {
                    self.bot
                        .delete_message(ChatId(task.context.chat_id), MessageId(task.context.message_id))
                        .await
                        .map_err(|e| RuntimeError::TaskError(format!("Failed to delete message: {}", e)))?;

                    let new_message = self
                        .bot
                        .send_photo(ChatId(task.context.chat_id), InputFile::url(thumbnail_url.clone()))
                        .caption(preview_text)
                        .reply_markup(get_confirm_download_keyboard())
                        .await
                        .map_err(|e| RuntimeError::TaskError(format!("Failed to send preview message: {}", e)))?;

                    let mut updated_context = task.context.clone();
                    updated_context.message_id = new_message.id.0;
                    queue_manager.update_pending_confirmation_context(media_info.id.clone(), updated_context);
                } else {
                    self.bot
                        .edit_message_text(
                            ChatId(task.context.chat_id),
                            MessageId(task.context.message_id),
                            preview_text,
                        )
                        .reply_markup(get_confirm_download_keyboard())
                        .await
                        .map_err(|e| RuntimeError::TaskError(format!("Failed to edit message: {}", e)))?;
                }

                Ok(DownloadState::Success(media_info))
            }
            crate::platform::DownloadState::Error => {
                self.bot
                    .edit_message_text(
                        ChatId(task.context.chat_id),
                        MessageId(task.context.message_id),
                        t!("messages.download.error"),
                    )
                    .reply_markup(get_main_menu_keyboard())
                    .await
                    .map_err(|e| RuntimeError::TaskError(format!("Something went wrong: {}", e)))?;

                Ok(DownloadState::Error)
            }
        }
    }
}

#[async_trait]
impl Worker for DownloadWorker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&self) -> Result<(), RuntimeError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let queue_manager = self.queue_manager.clone();
        let worker_ref = Arc::new(self.clone());

        for i in 0..self.concurrency {
            let worker_name = format!("{}_{}", self.name, i);
            let running = running.clone();
            let queue_manager = queue_manager.clone();
            let worker = worker_ref.clone();
            let mut rx = self.shutdown.subscribe();

            tokio::spawn(async move {
                while running.load(Ordering::SeqCst) {
                    tokio::select! {
                        task = queue_manager.pop_download_task() => {
                            if let Some(task_with_result) = task {
                                let result = match worker.process_task(task_with_result.task).await {
                                    Ok(result) => result,
                                    Err(e) => {
                                        error!("Worker {} failed to process task: {}", worker_name, e);
                                        DownloadState::Error
                                    }
                                };

                                if let Err(_e) = task_with_result.result_tx.send(result) {
                                   error!("Failed to send task result");
                                }
                            }
                        }
                        _ = rx.recv() => {
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    async fn stop(&self) -> Result<(), RuntimeError> {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.shutdown.send(());
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct PostDownloadWorker {
    name: String,
    concurrency: usize,
    queue_manager: TaskQueueManager,
    bot: Throttle<Bot>,
    shutdown: broadcast::Sender<()>,
    running: Arc<AtomicBool>,
}

impl PostDownloadWorker {
    pub fn new(name: &str, concurrency: usize, queue_manager: TaskQueueManager, bot: Throttle<Bot>) -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Self {
            name: name.to_string(),
            concurrency,
            queue_manager,
            bot,
            shutdown,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn process_task(&self, task: PostDownloadTask) -> Result<PostDownloadState, RuntimeError> {
        let platform_registry = AppState::get().unwrap().platform_registry;

        self.bot
            .delete_message(ChatId(task.context.chat_id), MessageId(task.context.message_id))
            .await
            .unwrap();

        let downloading_msg = self
            .bot
            .send_message(ChatId(task.context.chat_id), t!("callbacks.download.downloading"))
            .await
            .unwrap();

        self.bot
            .delete_message(ChatId(task.context.chat_id), downloading_msg.id)
            .await
            .unwrap();

        match task.context.platform {
            Platform::Instagram => {
                let platform = platform_registry
                    .get_platform::<PlatformInstagram>(&task.context.platform)
                    .unwrap();
                platform
                    .send_to_telegram(&self.bot, ChatId(task.context.chat_id), &task.media_file)
                    .await
                    .unwrap();
            }
            Platform::Youtube => {
                info!("Processing Youtube download task");
            }
            Platform::Bilibili => {
                info!("Processing Bilibili download task");
            }
        }

        self.bot
            .send_message(
                ChatId(task.context.chat_id),
                t!("callbacks.download.download_completed"),
            )
            .reply_markup(get_download_ask_for_link_keyboard(task.context.platform))
            .await
            .unwrap();

        Ok(PostDownloadState::Success)
    }
}

#[async_trait]
impl Worker for PostDownloadWorker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&self) -> Result<(), RuntimeError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let queue_manager = self.queue_manager.clone();
        let worker_ref = Arc::new(self.clone());

        for i in 0..self.concurrency {
            let worker_name = format!("{}_{}", self.name, i);
            let running = running.clone();
            let queue_manager = queue_manager.clone();
            let worker = worker_ref.clone();
            let mut rx = self.shutdown.subscribe();

            tokio::spawn(async move {
                while running.load(Ordering::SeqCst) {
                    tokio::select! {
                        task = queue_manager.pop_post_download_task() => {
                            if let Some(task_with_result) = task {
                                let result = match worker.process_task(task_with_result.task).await {
                                    Ok(result) => result,
                                    Err(e) => {
                                        error!("Worker {} failed to process task: {}", worker_name, e);
                                        PostDownloadState::Error
                                    }
                                };

                                if let Err(_e) = task_with_result.result_tx.send(result) {
                                   error!("Failed to send task result");
                                }
                            }
                        }
                        _ = rx.recv() => {
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    async fn stop(&self) -> Result<(), RuntimeError> {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.shutdown.send(());
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
