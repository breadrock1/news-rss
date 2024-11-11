use serde::Deserialize;
use sqlx::FromRow;

#[derive(FromRow, Deserialize)]
pub struct PgsqlTopicModel {
    pub id: i32,
    pub name: String,
    pub link: String,
    pub run_at_launch: bool,
}
