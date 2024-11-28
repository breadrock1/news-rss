use getset::Getters;
use regex::{Captures, Regex, Replacer};
use serde::Deserialize;

const FILTER_BLOCKS_TAGS: [&str; 5] = ["article", "content", "text", "war", "world"];
const FIND_JSON_OBJECTS_REGEX: &str = r#"\{(?:[^{}]|(?R))*}"#;
const FIND_LLM_BLOCKS_REGEX: &str = r#"<blocks>(?:[\W|\w])*<\/blocks>"#;

#[derive(Debug, Getters, Deserialize)]
#[getset(get = "pub")]
#[allow(dead_code)]
pub struct SemanticBlock {
    index: u32,
    tags: Vec<String>,
    content: Vec<String>,
}

struct DoubleQuotesReplacer;

impl Replacer for DoubleQuotesReplacer {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(&caps["first"]);
        dst.push_str("");
        dst.push_str(&caps["last"]);
    }
}

pub fn extract_semantic_blocks(text_data: &str) -> Result<String, anyhow::Error> {
    let trim_str = text_data.trim();
    let Some(founded) = Regex::new(FIND_LLM_BLOCKS_REGEX)?.find(trim_str) else {
        let msg = "cant found blocks data into llm response";
        return Err(anyhow::Error::msg(msg));
    };

    let result = founded
        .as_str()
        .replace("<blocks>", "")
        .replace("</blocks>", "");

    Ok(result)
}

#[allow(dead_code)]
pub fn extract_json_semantic_blocks(text_data: &str) -> Result<String, anyhow::Error> {
    let trim_str = text_data.trim();
    let founded_data = Regex::new(FIND_JSON_OBJECTS_REGEX)?
        .find_iter(trim_str)
        .filter_map(|it| match extract_json_object(it.as_str()) {
            Ok(data) => Some(data),
            Err(err) => {
                tracing::warn!(err=?err, "failed while extracting json object");
                None
            }
        })
        .collect::<Vec<SemanticBlock>>();

    let joined_strings = founded_data
        .into_iter()
        .filter(|it| {
            let lowercase_tags = it
                .tags()
                .iter()
                .map(|tag| tag.to_lowercase())
                .collect::<Vec<String>>();

            lowercase_tags
                .iter()
                .map(|it| it.as_str())
                .any(|it| FILTER_BLOCKS_TAGS.contains(&it))
        })
        .map(|it| it.content.join(" "))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(joined_strings)
}

fn extract_json_object(repaired: &str) -> Result<SemanticBlock, anyhow::Error> {
    let repaired = Regex::new(r#"(\n|  +)"#)?.replace_all(repaired, "");
    let repaired = Regex::new(r#""""#)?.replace_all(&repaired, "\",\"");
    let repaired = Regex::new(r#"}\{"#)?.replace_all(&repaired, "\",\"");
    let repaired = Regex::new(r"(\w+)\s*:")?.replace_all(&repaired, "\"$1\":");
    let repaired = Regex::new(r",\s*[}\]]")?.replace_all(&repaired, "$0");
    let repaired = Regex::new(r#"(?<first>(\w|\w ))"(?<last>(\w| ))"#)?
        .replace_all(&repaired, DoubleQuotesReplacer);

    let repaired_str = repaired.to_string();
    let sem_block = serde_json::from_str::<SemanticBlock>(&repaired_str)?;
    Ok(sem_block)
}

#[cfg(test)]
mod test_llm_retriever {
    use super::*;

    const BROKEN_CNN_JSON: &str = include_str!("../../../tests/resources/cnn-json-llm-resp.txt");
    const BROKEN_NDTV_JSON: &str = include_str!("../../../tests/resources/ndtv-news-llm-resp.txt");

    #[test]
    fn test_cnn_retriever() -> Result<(), anyhow::Error> {
        let result = extract_json_semantic_blocks(BROKEN_CNN_JSON)?;
        println!("{:#?}", result);
        assert_eq!(result.len(), 1527);
        Ok(())
    }

    #[test]
    fn test_ndtv_retriever() -> Result<(), anyhow::Error> {
        let result = extract_json_semantic_blocks(BROKEN_NDTV_JSON)?;
        println!("{:#?}", result);
        assert_eq!(result.len(), 1275);
        Ok(())
    }
}
