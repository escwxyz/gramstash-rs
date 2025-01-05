use crate::services::instagram::InstagramService;
use teloxide::prelude::*;

pub async fn handle(
    bot: Bot,
    msg: Message,
    username: String,
    password: String,
    instagram_service: &InstagramService,
) -> ResponseResult<()> {
    info!("Login with username: {} and password: {}", username, password);

    match instagram_service.clone().login(&username, &password).await {
        Ok(_) => {
            bot.send_message(
                msg.chat.id,
                "✅ Login successful! You can now download stories.\n\n\
                 Note: This session will expire in 30 days.",
            )
            .await?
        }
        Err(e) => bot.send_message(msg.chat.id, format!("❌ Login failed: {}", e)).await?,
    };

    Ok(())
}

// Helper function to show login instructions
pub fn get_login_instructions() -> String {
    "To access Instagram stories, you need to login first. Follow these steps:\n\n\
     1. Open Instagram in your desktop browser\n\
     2. Login to your account\n\
     3. Press F12 to open Developer Tools\n\
     4. Go to Application > Cookies > instagram.com\n\
     5. Find 'sessionid' and copy its value\n\
     6. Send it here with: /login <sessionid>\n\n\
     Note: Your session is stored securely and will expire in 30 days."
        .to_string()
}
