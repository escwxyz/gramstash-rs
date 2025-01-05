# GramStash (An Instagram Media Downloader) 🤖

A Telegram bot that allows users to download media content from Instagram, including posts, reels, and stories. Built with Rust for performance.

## Features 🌟

- Download Instagram posts and reels
- Rate limiting & Cache, powered by Upstash
- Access and download Instagram stories (Coming soon)
- Private chat only (no group chat support)
- Secure authentication handling (Coming soon)
- Redis support for session storage, powered by Upstash
- Shuttle for easy deployment

## Getting Started 🚀

### Prerequisites

- Rust 1.70 or higher
- A Telegram Bot Token (get it from [@BotFather](https://t.me/botfather))
- [Upstash](https://upstash.com/) account (for Redis and Rate Limiting)
- [Shuttle](https://shuttle.dev/) account (for deployment)

### Environment Variables

Create a `Secret.toml` file in the root directory:

```toml
UPSTASH_REDIS_HOST = "your-redis-host"
UPSTASH_REDIS_PASSWORD = "your-redis-password"
UPSTASH_REDIS_PORT = "your-redis-port"
TELEGRAM_BOT_TOKEN = "your-telegram-bot-token"
INSTAGRAM_API_ENDPOINT = "https://www.instagram.com/graphql/query/"
INSTAGRAM_DOC_ID = "your-instagram-doc-id"
RATE_LIMIT_DAILY_LIMIT = "your-rate-limit-daily-limit"
RATE_LIMIT_WINDOW_SECS = "your-rate-limit-window-secs"
CACHE_EXPIRY_SECS = "your-cache-expiry-secs"
DIALOGUE_USE_REDIS = "true to use redis, false to use in-memory"
DIALOGUE_CLEAR_INTERVAL_SECS = "your-dialogue-clear-interval-secs"
```

## Architecture 🏗️

- `src/bot.rs` - Core bot logic and dialogue state management
- `src/handlers/` - Message and callback handlers
- `src/services/` - Instagram API integration
- `src/utils/` - Helper functions and error handling

## Roadmap 🛣️

- [x] Basic post/reel downloading
- [x] Redis integration
- [x] Rate limiting
- [ ] Unit tests
- [ ] Instagram authentication
- [ ] Story downloading
- [ ] Highlight downloading
- [ ] Internationalization

...

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust coding conventions
- Add tests for new features
- Update documentation as needed
- Keep commits clean and well-documented

## Security 🔒

- User credentials are never stored permanently
- Messages containing sensitive information are automatically deleted
- Protected content mode for password input
- Session tokens are stored securely
- Group chat access is blocked

## License 📝

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments 👏

- [Teloxide](https://github.com/teloxide/teloxide) - Telegram Bot Framework

## Support 💬

For support, please open an issue.

## Disclaimer ⚠️

This bot is not affiliated with Instagram or Meta. Use it responsibly and in accordance with Instagram's terms of service.
