pub mod config;
mod errors;
mod prompt;

use crate::crawler::llm::config::LlmConfig;
use crate::crawler::llm::errors::LlmError;
use crate::crawler::llm::prompt::*;
use crate::crawler::CrawlerService;
use crate::ServiceConnect;

use getset::{CopyGetters, Getters};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::*;
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;

const REMOVE_BLOCKS_REGEX: &str = r#"<[^>]*>"#;
const REGEX_MATCH_SPLIT_AMOUNT: usize = 4;

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

    async fn scrape_text(&self, text_data: &str) -> Result<String, Self::Error> {
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

        let content_data = content_data.to_string();
        Self::extract_json_data(&content_data)
    }
}

#[derive(Debug, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
struct SemanticBlock {
    #[allow(dead_code)]
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    index: i32,
    tags: Vec<String>,
    content: Vec<String>,
}

impl LlmCrawler {
    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
    }

    fn create_system_prompt() -> ChatMessage {
        ChatMessage::System {
            name: Some(SYSTEM_PROMPT_NAME.to_string()),
            content: ChatMessageContent::Text(SCRAPE_HTML_SYSTEM_PROMPT.to_string()),
        }
    }

    fn create_user_query(query: &str) -> ChatMessage {
        ChatMessage::User {
            name: Some(USER_QUERY_NAME.to_string()),
            content: ChatMessageContent::Text(query.to_string()),
        }
    }

    fn extract_json_data(text_data: &str) -> Result<String, anyhow::Error> {
        let necessary_tags = vec!["article", "content"];
        let split_text_data = Regex::new(REMOVE_BLOCKS_REGEX)?
            .splitn(text_data, REGEX_MATCH_SPLIT_AMOUNT)
            .collect::<Vec<&str>>();

        let json_str_data = split_text_data[2];
        let semantic_blocks = serde_json::from_str::<Vec<SemanticBlock>>(json_str_data)?;

        let merged_data = semantic_blocks
            .into_iter()
            .filter(|it| {
                it.tags()
                    .iter()
                    .any(|i| necessary_tags.contains(&i.as_str()))
            })
            .map(|it| it.content().join("\n"))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(merged_data)
    }
}
