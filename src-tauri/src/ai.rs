use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AiProviderSettings {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub enabled: bool,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiNoteInsightPayload {
    pub summary: String,
    pub takeaways: Vec<String>,
    pub exam_questions: Vec<String>,
    pub connection_opportunities: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCourseBriefPayload {
    pub summary: String,
    pub revision_priorities: Vec<String>,
    pub weak_spots: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardCard {
    pub question: String,
    pub answer: String,
}

pub fn validate_settings(settings: &AiProviderSettings) -> Result<String> {
    if !settings.enabled {
        return Ok("AI is disabled; local parsing remains active.".to_string());
    }

    ensure_required(settings)?;
    let payload = json!({
        "model": settings.model,
        "messages": [
            { "role": "system", "content": "Reply with OK." },
            { "role": "user", "content": "OK" }
        ],
        "max_tokens": 8,
        "temperature": 0
    });

    send_chat_request(settings, payload).context("failed to validate AI provider settings")?;

    Ok(format!(
        "Validated {} against {}",
        settings.model, settings.base_url
    ))
}

pub fn generate_flashcards(
    settings: &AiProviderSettings,
    course_name: &str,
    note_payload: &str,
) -> Result<Vec<FlashcardCard>> {
    ensure_required(settings)?;

    let prompt = format!(
        "Generate up to 12 study flashcards for the course `{course_name}` from the provided note payload. \
         Return strict JSON with one key `cards`, where the value is an array of objects with `question` and `answer`."
    );

    let payload = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": "You are a flashcard generator. Return JSON only." },
            { "role": "user", "content": prompt },
            { "role": "user", "content": note_payload }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    let response: FlashcardEnvelope = parse_json_blob(&content)?;
    Ok(response.cards)
}

pub fn generate_note_insight(
    settings: &AiProviderSettings,
    title: &str,
    excerpt: &str,
    headings: &[String],
    concepts: &[String],
    formulas: &[String],
    links: &[String],
) -> Result<AiNoteInsightPayload> {
    ensure_required(settings)?;

    let prompt = format!(
        "Analyze this study note for exam preparation. Return strict JSON with keys \
         `summary`, `takeaways`, `examQuestions`, and `connectionOpportunities`. \
         `summary` must be a compact paragraph. The arrays should contain short, concrete strings. \
         Focus on what the student should remember, what they may be asked, and what to link next.\n\
         Title: {title}\n\
         Headings: {}\n\
         Concepts: {}\n\
         Formulas: {}\n\
         Links: {}\n\
         Excerpt: {excerpt}",
        headings.join(" | "),
        concepts.join(", "),
        formulas.join(" | "),
        links.join(", ")
    );

    let payload = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": "You are an exam study coach. Return JSON only." },
            { "role": "user", "content": prompt }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    parse_json_blob::<AiNoteInsightPayload>(&content)
}

pub fn generate_course_brief(
    settings: &AiProviderSettings,
    course_name: &str,
    top_concepts: &[String],
    weak_notes: &[String],
    note_payload: &str,
) -> Result<AiCourseBriefPayload> {
    ensure_required(settings)?;

    let prompt = format!(
        "You are preparing a student for the course `{course_name}`. Return strict JSON with keys \
         `summary`, `revisionPriorities`, `weakSpots`, and `nextActions`. The summary must be a compact \
         paragraph. The arrays should contain short, concrete bullets. Use the note insights to describe \
         what to revise next, what is underlinked, and how to study efficiently.\n\
         Top concepts: {}\n\
         Weak notes: {}\n\
         Note insights:\n{}",
        top_concepts.join(", "),
        weak_notes.join(" | "),
        note_payload
    );

    let payload = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": "You are an exam preparation strategist. Return JSON only." },
            { "role": "user", "content": prompt }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    parse_json_blob::<AiCourseBriefPayload>(&content)
}

fn ensure_required(settings: &AiProviderSettings) -> Result<()> {
    if settings.base_url.trim().is_empty() {
        return Err(anyhow!("base URL is required"));
    }
    if settings.model.trim().is_empty() {
        return Err(anyhow!("model is required"));
    }
    Ok(())
}

fn build_client(timeout_ms: u64) -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(2_000)))
        .build()
        .context("failed to build HTTP client")
}

fn endpoint(base_url: &str, suffix: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        suffix.trim_start_matches('/')
    )
}

fn send_chat_request(settings: &AiProviderSettings, payload: Value) -> Result<Value> {
    let client = build_client(settings.timeout_ms)?;
    let url = endpoint(&settings.base_url, "chat/completions");
    let response = with_optional_bearer(client.post(url), &settings.api_key)
        .json(&payload)
        .send()
        .context("failed to reach chat completions endpoint")?;

    let status = response.status();
    let body = response
        .text()
        .context("failed to read provider response")?;
    if !status.is_success() {
        return Err(anyhow!("provider rejected request ({status}): {body}"));
    }

    serde_json::from_str(&body).context("provider returned invalid JSON")
}

fn message_content(json: &Value) -> Result<String> {
    if let Some(value) = json.pointer("/choices/0/message/content") {
        if let Some(content) = extract_content_text(value) {
            return Ok(content);
        }
    }

    if let Some(value) = json.pointer("/choices/0/text").and_then(Value::as_str) {
        return Ok(value.trim().to_string());
    }

    Err(anyhow!(
        "provider response did not contain readable message content"
    ))
}

fn parse_json_blob<T>(content: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let trimmed = content.trim();
    let cleaned = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|value| value.trim())
        .and_then(|value| value.strip_suffix("```").map(str::trim))
        .unwrap_or(trimmed);

    serde_json::from_str(cleaned)
        .or_else(|_| {
            extract_json_object(cleaned)
                .ok_or_else(|| anyhow!("AI response was not valid JSON"))
                .and_then(|value| {
                    serde_json::from_str(value).context("AI response was not valid JSON")
                })
        })
        .context("AI response was not valid JSON")
}

fn with_optional_bearer(request: RequestBuilder, api_key: &str) -> RequestBuilder {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        request
    } else {
        request.bearer_auth(trimmed)
    }
}

fn extract_content_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.trim().to_string()),
        Value::Array(parts) => {
            let joined = parts
                .iter()
                .filter_map(|part| match part {
                    Value::String(text) => Some(text.trim().to_string()),
                    Value::Object(map) => map
                        .get("text")
                        .and_then(Value::as_str)
                        .map(|text| text.trim().to_string()),
                    _ => None,
                })
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("\n");

            if joined.is_empty() {
                None
            } else {
                Some(joined)
            }
        }
        Value::Object(map) => map
            .get("text")
            .and_then(Value::as_str)
            .map(|text| text.trim().to_string()),
        _ => None,
    }
}

fn extract_json_object(content: &str) -> Option<&str> {
    let mut depth = 0usize;
    let mut start = None;
    let mut in_string = false;
    let mut escape = false;

    for (index, ch) in content.char_indices() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }

            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start_index) = start {
                        return Some(&content[start_index..=index]);
                    }
                }
            }
            _ => {}
        }
    }

    None
}

#[derive(Debug, Deserialize)]
struct FlashcardEnvelope {
    cards: Vec<FlashcardCard>,
}

#[cfg(test)]
mod tests {
    use super::{extract_json_object, message_content, parse_json_blob};
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq)]
    struct SamplePayload {
        summary: String,
    }

    #[test]
    fn parses_json_inside_code_fence_with_extra_text() {
        let payload: SamplePayload =
            parse_json_blob("Here you go:\n```json\n{\"summary\":\"ready\"}\n```\n")
                .expect("payload");
        assert_eq!(
            payload,
            SamplePayload {
                summary: "ready".to_string()
            }
        );
    }

    #[test]
    fn extracts_first_balanced_json_object() {
        let extracted =
            extract_json_object("noise {\"summary\":\"ready\",\"nested\":{\"ok\":true}} tail")
                .expect("json object");
        assert_eq!(
            extracted,
            "{\"summary\":\"ready\",\"nested\":{\"ok\":true}}"
        );
    }

    #[test]
    fn reads_message_content_from_content_parts() {
        let response = json!({
            "choices": [
                {
                    "message": {
                        "content": [
                            { "type": "output_text", "text": "line one" },
                            { "type": "text", "text": "line two" }
                        ]
                    }
                }
            ]
        });

        let content = message_content(&response).expect("content");
        assert_eq!(content, "line one\nline two");
    }
}
