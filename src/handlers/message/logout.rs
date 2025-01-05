use teloxide::{prelude::*, ApiError, RequestError};

use crate::services::instagram::InstagramService;

pub async fn handle(bot: Bot, msg: Message, instagram_service: &InstagramService) -> ResponseResult<()> {
    match instagram_service.clone().logout().await {
        Ok(_) => {
            bot.send_message(msg.chat.id, "✅ Logged out successfully").await?;
            Ok(())
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Logout failed: {}", e))
                .await?;
            Err(RequestError::Api(ApiError::Unknown(e.to_string())))
        }
    }
}
