use super::*;
use axum::http::HeaderMap;
use reqwest::Client;

const API_BASE_URL: &str = "https://api.polar.sh/v1";

#[derive(Debug, Serialize, Deserialize)]
struct ProductsResponse {
    items: Vec<Product>,
    pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    description: String,
    is_recurring: bool,
    is_archived: bool,
    created_at: DateTime<Utc>,
    modified_at: Option<DateTime<Utc>>,
    prices: Vec<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    id: String,
    price_amount: i32,
    price_currency: String,
    #[serde(rename = "type")]
    price_type: String,
    recurring_interval: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    total_count: i32,
    max_page: i32,
}

pub struct PolarPaymentProvider {
    client: Client,
    config: PaymentConfig,
}

impl PolarPaymentProvider {
    pub fn new(config: PaymentConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", config.api_key).parse().unwrap());
        let client = Client::builder().default_headers(headers).build().unwrap();
        Self { client, config }
    }

    pub async fn get_products(&self) -> Result<Vec<Product>, PaymentError> {
        let response = self
            .client
            .get(&format!("{}/products", API_BASE_URL))
            .send()
            .await
            .map_err(|e| PaymentError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PaymentError::ApiError(format!(
                "Failed to get products: {}",
                response.status()
            )));
        }

        let products_response: ProductsResponse = response
            .json()
            .await
            .map_err(|e| PaymentError::ApiError(e.to_string()))?;

        Ok(products_response.items)
    }

    // 辅助方法：获取月度订阅价格
    async fn get_monthly_price_id(&self) -> Result<String, PaymentError> {
        let products = self.get_products().await?;

        for product in products {
            if product.id == self.config.monthly_product_id {
                if let Some(price) = product
                    .prices
                    .iter()
                    .find(|p| p.recurring_interval.as_deref() == Some("month") && p.price_currency == "usd")
                {
                    return Ok(price.id.clone());
                }
            }
        }

        Err(PaymentError::ApiError("Monthly price not found".into()))
    }
}

#[async_trait]
impl PaymentProvider for PolarPaymentProvider {
    async fn create_checkout_session(&self, user_id: &str) -> Result<String, PaymentError> {
        // 调用 Polar API 创建结账会话
        todo!()
    }

    async fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<WebhookEvent, PaymentError> {
        // 处理 Polar webhook
        todo!()
    }

    async fn get_subscription(&self, subscription_id: &str) -> Result<Subscription, PaymentError> {
        // 获取订阅信息
        todo!()
    }

    async fn create_trial(&self, user_id: &str) -> Result<Subscription, PaymentError> {
        // 创建试用订阅
        todo!()
    }

    async fn create_checkout_link(&self, user_id: &str) -> Result<String, PaymentError> {
        let price_id = self.get_monthly_price_id().await?;

        let response = self
            .client
            .post(&format!("{}/checkouts", API_BASE_URL))
            .json(&serde_json::json!({
                "price_id": price_id,
                "success_url": format!(
                    "http://127.0.0.1:8000/checkout/success?checkout_id={{CHECKOUT_ID}}"
                ),
                "cancel_url": "http://127.0.0.1:8000/checkout/cancel",
                "custom_data": {
                    "user_id": user_id
                }
            }))
            .send()
            .await
            .map_err(|e| PaymentError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PaymentError::ApiError(format!(
                "Failed to create checkout: {}",
                response.status()
            )));
        }

        let checkout: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PaymentError::ApiError(e.to_string()))?;

        // 从响应中提取 checkout URL
        checkout["url"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| PaymentError::ApiError("Checkout URL not found".into()))
    }
}

// {
//     "items": [
//       {
//         "created_at": "2025-01-24T12:28:45.117415Z",
//         "modified_at": "2025-01-24T12:28:45.799636Z",
//         "id": "8c401e1e-99b7-46f7-a407-030f849f2c91",
//         "name": "Premium Tier",
//         "description": "GramStash's premium user tier for exclusive features.",
//         "is_recurring": true,
//         "is_archived": false,
//         "organization_id": "b9d2997b-11bf-4aaf-a854-636df5a44cd9",
//         "metadata": {},
//         "prices": [
//           {
//             "created_at": "2025-01-24T12:28:45.803610Z",
//             "modified_at": null,
//             "id": "ba37f379-5fe8-423c-9cc0-c090269e6c54",
//             "amount_type": "fixed",
//             "is_archived": false,
//             "product_id": "8c401e1e-99b7-46f7-a407-030f849f2c91",
//             "price_currency": "usd",
//             "price_amount": 499,
//             "type": "recurring",
//             "recurring_interval": "month"
//           },
//           {
//             "created_at": "2025-01-24T12:28:45.803619Z",
//             "modified_at": null,
//             "id": "6dea7c65-eedd-4139-8f0e-d86f92f5944e",
//             "amount_type": "fixed",
//             "is_archived": false,
//             "product_id": "8c401e1e-99b7-46f7-a407-030f849f2c91",
//             "price_currency": "usd",
//             "price_amount": 4999,
//             "type": "recurring",
//             "recurring_interval": "year"
//           }
//         ],
//         "benefits": [],
//         "medias": [],
//         "attached_custom_fields": []
//       }
//     ],
//     "pagination": {
//       "total_count": 1,
//       "max_page": 1
//     }
//   }
