#[cfg(feature = "publish-offline")]
use crate::publish::pgsql::config::PgsqlConfig;

use crate::publish::rabbit::config::RabbitConfig;

use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct PublishConfig {
    rmq: RabbitConfig,
    #[cfg(feature = "publish-offline")]
    pgsql: PgsqlConfig,
}
