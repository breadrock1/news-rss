use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
pub struct LocalCacheConfig {
    #[getset(get_copy = "pub")]
    expired: u64,
}
