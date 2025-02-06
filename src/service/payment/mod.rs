mod polar;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    PastDue,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub api_key: String,
    pub webhook_secret: String,
    pub trial_product_id: String,
    pub monthly_product_id: String,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_checkout_link(&self, user_id: &str) -> Result<String, PaymentError>;
    async fn create_checkout_session(&self, user_id: &str) -> Result<String, PaymentError>;
    async fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<WebhookEvent, PaymentError>;
    async fn get_subscription(&self, subscription_id: &str) -> Result<Subscription, PaymentError>;
    async fn create_trial(&self, user_id: &str) -> Result<Subscription, PaymentError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,
    #[error("Subscription not found")]
    SubscriptionNotFound,
    #[error("User already has active subscription")]
    AlreadySubscribed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebhookEvent {
    SubscriptionCreated(Subscription),
    SubscriptionUpdated(Subscription),
    SubscriptionCanceled(Subscription),
    TrialEnded(Subscription),
}

pub struct PaymentService {}
