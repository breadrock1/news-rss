pub mod config;
mod errors;
mod prompt;

use crate::crawler::llm::config::LlmConfig;
use crate::crawler::llm::errors::LlmError;
use crate::crawler::llm::prompt::*;
use crate::crawler::CrawlerService;
use crate::ServiceConnect;

use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::*;
use regex::Regex;
use std::sync::Arc;

const FIND_LLM_BLOCKS_REGEX: &str = r#"<blocks>[\w\W]+?<\/blocks>"#;

pub struct LlmCrawler {
    client: Arc<Client>,
}

#[async_trait::async_trait]
impl ServiceConnect for LlmCrawler {
    type Config = LlmConfig;
    type Error = LlmError;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let api_key = config.api_key().clone();
        let llm_address = config.base_url();
        let client = Client::new_with_base(llm_address, api_key);
        let llm_client = Arc::new(client);
        Ok(LlmCrawler { client: llm_client })
    }
}

#[async_trait::async_trait]
impl CrawlerService for LlmCrawler {
    type Error = anyhow::Error;

    async fn scrape(&self, text_data: &str) -> Result<String, Self::Error> {
        let system_prompt_msg = Self::create_system_prompt();
        let user_query_msg = Self::create_user_query(text_data);
        let completion = ChatCompletionParametersBuilder::default()
            .model(LLM_MODEL_NAME)
            .messages(vec![system_prompt_msg, user_query_msg])
            .response_format(ChatCompletionResponseFormat::Text)
            .build()?;

        let response = self.client().chat().create(completion).await?;
        let chat_message = response.choices[0].message.clone();
        let ChatMessage::Assistant { content, .. } = chat_message else {
            let err = anyhow::Error::msg("returned incorrect chat message from llm");
            return Err(err);
        };

        let Some(content_data) = content else {
            let err = anyhow::Error::msg("returned empty response from llm");
            return Err(err);
        };
        
        Self::extract_semantic_blocks(&content_data.to_string())
    }

    async fn scrape_by_url(&self, url: &str) -> Result<String, Self::Error> {
        let response = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?;

        let html_str = response.text().await?;
        self.scrape(&html_str).await
    }
}

impl LlmCrawler {
    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
    }

    fn create_system_prompt() -> ChatMessage {
        ChatMessage::System {
            name: Some(SYSTEM_PROMPT_NAME.to_string()),
            content: ChatMessageContent::Text(SCRAPE_HTML_SYSTEM_PROMPT_AS_TEXT.to_string()),
        }
    }

    fn create_user_query(query: &str) -> ChatMessage {
        ChatMessage::User {
            name: Some(USER_QUERY_NAME.to_string()),
            content: ChatMessageContent::Text(query.to_string()),
        }
    }

    pub fn extract_semantic_blocks(text_data: &str) -> Result<String, anyhow::Error> {
        let Some(founded) = Regex::new(FIND_LLM_BLOCKS_REGEX)?.find(text_data) else {
            tracing::warn!("failed to match blocks into llm response by regex");
            return Ok(text_data.to_string())
        };

        let founded_string = founded
            .as_str()
            .replace("<blocks>", "")
            .replace("</blocks>", "");

        Ok(founded_string)
    }
}
