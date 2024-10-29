pub(super) const LLM_MODEL_NAME: &'static str = "gpt-4o-2024-08-06";
pub(super) const SYSTEM_PROMPT_NAME: &'static str = "system-prompt";
pub(super) const USER_QUERY_NAME: &'static str = "user-query";

pub(super) const SCRAPE_HTML_SYSTEM_PROMPT_SUM: &'static str = r#"
Here is the URL of the webpage:
<url>{URL}</url>

And here is the cleaned HTML content of that webpage:
<html>
{HTML}
</html>

Your task is to break down this HTML content into semantically relevant blocks, and for each block, generate a list of 
semantic tags that are relevant to the content.

There are following steps that you have to do:

1. Carefully read through the HTML content and identify logical breaks or shifts in the content that would warrant splitting it into separate blocks.

2. Make sure to escape any special characters in the HTML content, and also single or double quote to avoid parsing issues.

3. For each block:
   a. Assign it an index based on its order in the content.
   b. Analyze the content and generate a list of relevant semantic tags that describe what the block is about like article, menu, text, options, about.
   c. Extract the text content, clean it up if needed, and store it as a list of strings in the "content" field.

4. Ensure that the order of the blocks as they appear in the original HTML content is matched.

6. Iterate over each block and remove it if tags of this current block does not contains any tag like article, content, text, news, feeds.

7. Returns merged string data of filtered blocks content field into <blocks> tags.

Please provide your output within <blocks> tags, like this:

<blocks>
  This is the first paragraph of the article, which provides an introduction and overview of the main topic.
  This is the second paragraph, which delves into the history and background of the topic.
  It provides context and sets the ståage for tåhe rest of the article.
</blocks>

Remember, the output should be a complete and parsable text data in <blocks> tags, with no omissions or errors and without any sentenses before and after XML document. The XML document should semantically break down the content into relevant blocks, maintaining the original order.
"#;

pub(super) const SCRAPE_HTML_SYSTEM_PROMPT_AS_TEXT: &'static str = r#"
Here is the URL of the webpage:
<url>{URL}</url>

And here is the cleaned HTML content of that webpage:
<html>
{HTML}
</html>

Your task is to break down this HTML content into semantically relevant blocks, and for each block, generate a list of 
semantic tags that are relevant to the content.

There are following steps that you have to do:

1. Carefully read through the HTML content and identify logical breaks or shifts in the content that would warrant splitting it into separate blocks.

2. Make sure to escape any special characters in the HTML content, and also single or double quote to avoid parsing issues.

3. For each block:
   a. Assign it an index based on its order in the content.
   b. Analyze the content and generate a list of relevant semantic tags that describe what the block is about like article, menu, text, options, about.
   c. Extract the text content, clean it up if needed, and store it as a list of strings in the "content" field.

4. Ensure that the order of the blocks as they appear in the original HTML content is matched.

6. Iterate over each block and remove it if tags of this current block does not contains any tag like article, content, text, news, feeds.

7. Returns merged string data of filtered blocks content field into <blocks> tags.

Please provide your output within <blocks> tags, like this:

<blocks>
  This is the first paragraph of the article, which provides an introduction and overview of the main topic.
  This is the second paragraph, which delves into the history and background of the topic.
  It provides context and sets the ståage for tåhe rest of the article.
</blocks>

Remember, the output should be a complete and parsable text data in <blocks> tags, with no omissions or errors and without any sentenses before and after XML document. The XML document should semantically break down the content into relevant blocks, maintaining the original order.
"#;

pub(super) const _SCRAPE_HTML_SYSTEM_PROMPT_AS_XML: &'static str = r#"
Here is the URL of the webpage:
<url>{URL}</url>

And here is the cleaned HTML content of that webpage:
<html>
{HTML}
</html>


Your task is to break down this HTML content into semantically relevant blocks, and for each block, generate a XML document with the following keys:

- index: an integer representing the index of the block in the content
- tags: a list of semantic tags that are relevant to the content of the block
- content: a list of strings containing the text content of the block

To generate the XML document:

1. Carefully read through the HTML content and identify logical breaks or shifts in the content that would warrant splitting it into separate blocks.

2. Make sure to escape any special characters in the HTML content, and also single or double quote to avoid XML parsing issues.

3. For each block:
   a. Assign it an index based on its order in the content.
   b. Analyze the content and generate a list of relevant semantic tags that describe what the block is about like article, menu, text, options, about.
   c. Extract the text content, clean it up if needed, and store it as a list of strings in the "content" field.

4. Ensure that the order of the XML document matches the order of the blocks as they appear in the original HTML content.

5. Double-check that each XML key includes all required keys (index, tags, content) and that the values are in the expected format (integer, list of strings, etc.).

6. Make sure the generated XML is complete and parsable, with no errors or omissions.

Please provide your output within <blocks> tags, like this:

<blocks>
  <block>
    <index>0</index>
    <tags>
      <tag>introduction</tag>
      <tag>overview</tag>
    </tags>
    <contents>
      <content>"This is the first paragraph of the article, which provides an introduction and overview of the main topic."</content>
    </contents>
  </block>
  <block>
    <index>1</index>
    <tags>
      <tag>article</tag>
      <tag>text</tag>
    </tags>
    <contents>
      <content>"This is the second paragraph, which delves into the history and background of the topic."</content>
      <content>"It provides context and sets the ståage for tåhe rest of the article."</content>
    </contents>
  </block>
</blocks>

Remember, the output should be a complete, parsable XML wrapped in <blocks> tags, with no omissions or errors and without any sentenses before and after XML document. The XML document should semantically break down the content into relevant blocks, maintaining the original order.
"#;

pub(super) const _SCRAPE_HTML_SYSTEM_PROMPT_AS_JSON: &'static str = r#"
Here is the URL of the webpage:
<url>{URL}</url>

And here is the cleaned HTML content of that webpage:
<html>
{HTML}
</html>

Your task is to break down this HTML content into semantically relevant blocks, and for each block, generate a JSON object with the following keys:

- index: an integer representing the index of the block in the content
- tags: a list of semantic tags that are relevant to the content of the block
- content: a list of strings containing the text content of the block

To generate the JSON objects:

1. Carefully read through the HTML content and identify logical breaks or shifts in the content that would warrant splitting it into separate blocks.

2. Make sure to escape any special characters in the HTML content, and also single or double quote to avoid JSON parsing issues.

3. For each block:
   a. Assign it an index based on its order in the content.
   b. Analyze the content and generate a list of relevant semantic tags that describe what the block is about like article, menu, text, options, about.
   c. Extract the text content, clean it up if needed, and store it as a list of strings in the "content" field.

4. Ensure that the order of the JSON objects matches the order of the blocks as they appear in the original HTML content.

5. Double-check that each JSON object includes all required keys (index, tags, content) and that the values are in the expected format (integer, list of strings, etc.).

6. Make sure the generated JSON is complete and parsable, with no errors or omissions.

Please provide your output within <blocks> tags, like this:

<blocks>
[{
  "index": 0,
  "tags": ["introduction", "overview"],
  "content": [
    "This is the first paragraph of the article, which provides an introduction and overview of the main topic."
  ]
},
{
  "index": 1,
  "tags": ["history", "background"],
  "content": [
    "This is the second paragraph, which delves into the history and background of the topic.",
    "It provides context and sets the ståage for tåhe rest of the article."
  ]
}]
</blocks>

Remember, the output should be a complete, parsable JSON wrapped in <blocks> tags, with no omissions or errors and without any sentenses before and after JSON data. The JSON objects should semantically break down the content into relevant blocks, maintaining the original order.
"#;
