use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("failed to connect to llm client: {0}")]
    Connect(String),
}
