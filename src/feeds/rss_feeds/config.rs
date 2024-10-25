use getset::{CopyGetters, Getters, Setters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters, Setters)]
pub struct RssConfig {
    #[getset(get = "pub", set = "pub")]
    target_url: String,
    #[getset(get_copy = "pub")]
    max_retries: u32,
    #[getset(get_copy = "pub")]
    timeout: u64,
    #[getset(get_copy = "pub", set = "pub")]
    interval_secs: u64,
}
