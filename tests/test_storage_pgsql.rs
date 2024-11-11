mod tests_helper;

#[cfg(feature = "storage-pgsql")]
mod test_storage_pgsql {
    use crate::tests_helper;

    use news_rss::config::ServiceConfig;
    use news_rss::logger;
    use news_rss::storage::LoadTopic;

    #[tokio::test]
    async fn test_load_all() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let storage = tests_helper::build_pgsql_storage(&config).await?;

        let result = storage.load_all().await?;
        assert!(result.len() > 0);

        let result = storage.load_at_launch().await?;
        assert!(result.len() >= 1);

        Ok(())
    }
}
