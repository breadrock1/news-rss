use serde::Serialize;

//const BROKEN_JSON_DATA: &str = include_str!("resources/responses/ndtv-uk-news-llm-response.txt");
const BROKEN_JSON_DATA: &str = include_str!("resources/responses/cnn-news-llm-response.txt");

const FIND_LLM_BLOCKS_REGEX: &str = r#"<blocks>[\w\W]+?<\/blocks>"#;

#[derive(Debug, Clone, Serialize)]
enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

struct JsonParser {
    input: String,
    position: usize,
}

#[test]
pub fn test_json_repair() -> Result<(), anyhow::Error> {
    let json_str = BROKEN_JSON_DATA;

    let matched = regex::Regex::new(FIND_LLM_BLOCKS_REGEX)?
        .find(json_str)
        .unwrap();
    let matched_str = matched.as_str();
    let mut lines = matched_str.split('\n').collect::<Vec<&str>>();
    lines.remove(lines.len() - 1);
    lines.remove(0);

    let json_str = lines.join("\n");
    let mut parser = JsonParser::new(json_str);
    parser.repair_input();
    let result = match parser.parse() {
        Ok(value) => {
            println!("{:#?}", &value);
            value
        }
        Err(e) => {
            println!("Error parsing JSON: {}", &e);
            return Err(anyhow::Error::msg("Error parsing JSON: {e}"));
        }
    };

    let result_str = serde_json::to_string(&result).unwrap();
    println!("{}", result_str);

    Ok(())
}

impl JsonParser {
    fn new(input: String) -> Self {
        JsonParser { input, position: 0 }
    }

    fn repair_input(&mut self) {
        // Replace invalid escape sequences
        let mut repaired = String::new();
        let mut chars = self.input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.peek() {
                    Some(&'"') | Some(&'\\') | Some(&'/') | Some(&'b') | Some(&'f')
                    | Some(&'n') | Some(&'r') | Some(&'t') => {
                        repaired.push(c);
                    }
                    Some(&'u') => {
                        repaired.push(c);
                    }
                    _ => {
                        // Invalid escape sequence, remove backslash
                        continue;
                    }
                }
            } else {
                repaired.push(c);
            }
        }

        // Fix unquoted keys
        let re_unquoted_key = regex::Regex::new(r"(\w+)\s*:").unwrap();
        repaired = re_unquoted_key
            .replace_all(&repaired, "\"$1\":")
            .to_string();

        // Fix missing commas
        let re_missing_comma = regex::Regex::new(r"}\s*{").unwrap();
        repaired = re_missing_comma.replace_all(&repaired, "},{").to_string();

        // Fix trailing commas
        let re_trailing_comma = regex::Regex::new(r",\s*[}\]]").unwrap();
        repaired = re_trailing_comma.replace_all(&repaired, "$0").to_string();

        self.input = repaired;
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            Some(c) => Err(format!("Unexpected character: {} at {}", c, self.position)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume '{'
        let mut pairs = Vec::new();

        self.skip_whitespace();
        if self.peek_char() == Some('}') {
            self.consume_char();
            return Ok(JsonValue::Object(pairs));
        }

        loop {
            self.skip_whitespace();

            // Parse key
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string as object key".to_string()),
            };

            self.skip_whitespace();
            if self.consume_char() != ':' {
                return Err("Expected ':' after object key".to_string());
            }

            // Parse value
            self.skip_whitespace();
            let value = self.parse()?;
            pairs.push((key, value));

            self.skip_whitespace();
            match self.consume_char() {
                ',' => continue,
                '}' => break,
                // mut c => {
                //     c = ' ';
                //     continue;
                // },
                c => {
                    // return Err(format!("Expected ',' or '}}', found {}", c))
                    return Ok(JsonValue::Object(pairs));
                }
            }
        }

        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume '['
        let mut values = Vec::new();

        self.skip_whitespace();
        if self.peek_char() == Some(']') {
            self.consume_char();
            return Ok(JsonValue::Array(values));
        }

        loop {
            self.skip_whitespace();
            let parsed = self.parse()?;
            values.push(parsed);

            self.skip_whitespace();
            match self.consume_char() {
                ',' => continue,
                ']' => break,
                // mut c if c == '"' => {
                //     let cc = self.peek_char().unwrap();
                //     c = ',';
                //     continue;
                // },
                c => {
                    // return Err(format!("Expected ',' or ']', found {}", c))
                    return Ok(JsonValue::Array(values));

                    // println!("Expected ',' or ']', found {c}");
                    // break;
                }
            }
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume opening quote
        let mut result = String::new();

        while let Some(c) = self.peek_char() {
            match c {
                '"' => {
                    self.consume_char();
                    return Ok(JsonValue::String(result));
                }
                '\\' => {
                    self.consume_char();
                    match self.consume_char() {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        c => return Err(format!("Invalid escape sequence: \\{}", c)),
                    }
                }
                c => {
                    self.consume_char();
                    result.push(c);
                }
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut number_str = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '-' || c == '.' || c == 'e' || c == 'E' || c == '+' {
                self.consume_char();
                number_str.push(c);
            } else {
                break;
            }
        }

        number_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        if self.input[self.position..].starts_with("true") {
            self.position += 4;
            Ok(JsonValue::Boolean(true))
        } else if self.input[self.position..].starts_with("false") {
            self.position += 5;
            Ok(JsonValue::Boolean(false))
        } else {
            Err("Invalid boolean value".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.position..].starts_with("null") {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Invalid null value".to_string())
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.position..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.position += next_pos;
        cur_char
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char();
            } else {
                println!("{c}");
                break;
            }
        }
    }
}
