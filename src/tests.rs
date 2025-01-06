// TODO: Add tests for other commands and scenarios

use teloxide_tests::{MockBot, MockMe, MockMessageText};

use crate::bot::handler_tree;

#[tokio::test]
async fn test_start_command() {
    // Create a mock message with /start command
    let mock_message = MockMessageText::new().text("/start");

    let bot = MockBot::new(mock_message, handler_tree());

    // let me = MockMe::new().first_name("GramStash");

    // Add debug print to verify the mock setup
    // println!("Mock first name: {}", me.first_name);

    // bot.me(me);

    bot.dispatch().await;

    let responses = bot.get_responses();

    let message = responses.sent_messages.last().expect("No sent messages were detected!");

    println!("Response text: {:?}", message.text());

    assert_eq!(message.text(), Some("👋 Hi First\\!\n\nWelcome to GramStash\\! I can help you download content from Instagram\\.\n\nPlease select an option below\\:"));
}

#[tokio::test]
async fn test_help_command() {
    let mock_message = MockMessageText::new().text("/help");
    let bot = MockBot::new(mock_message, handler_tree());
    bot.dispatch().await;
    let responses = bot.get_responses();
    let message = responses.sent_messages.last().expect("No sent messages were detected!");
    assert_eq!(message.text(), Some("This is a help message"));
}

#[tokio::test]
async fn test_keyboard_help_menu() {
    let mock_message = MockMessageText::new().text("ℹ️ Help");
    let bot = MockBot::new(mock_message, handler_tree());
    bot.dispatch().await;
    let responses = bot.get_responses();
    let message = responses.sent_messages.last().expect("No sent messages were detected!");
    assert_eq!(
        message.text(),
        Some("ℹ️ Help and Information\n\n/start - Start the bot\n/help - Show this help message")
    );
}
