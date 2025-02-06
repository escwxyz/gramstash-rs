use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub total_requests: u32,        // combined request number to all resources today
    pub total_used_requests: usize, // total requests to different resources used today
    pub remaining_requests: usize,  // remaining requests to different resources today
    pub reset_time: String,         // timestamp of the next reset
}
