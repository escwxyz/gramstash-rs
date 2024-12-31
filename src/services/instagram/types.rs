use serde::{Deserialize, Serialize};

// Response structures
#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    pub authenticated: bool,
    pub user: bool,
    pub two_factor_required: Option<bool>,
    pub two_factor_info: Option<TwoFactorInfo>,
    pub checkpoint_url: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TwoFactorInfo {
    pub two_factor_identifier: String,
}

// Session related structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionData {
    pub cookies: Vec<SerializableCookie>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub csrf_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Clone)]
pub struct TwoFactorAuthPending {
    pub user: String,
    pub two_factor_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Carousel,
    Story,
}

// TODO: improve this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub url: String,
    pub media_type: MediaType,
    pub carousel_items: Vec<CarouselItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselItem {
    pub url: String,
    pub media_type: MediaType,
}
