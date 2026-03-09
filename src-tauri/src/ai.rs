use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
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
    let client = build_client(settings.timeout_ms)?;
    let url = endpoint(&settings.base_url, "models");
    let response = client
        .get(url)
        .bearer_auth(&settings.api_key)
        .send()
        .context("failed to reach model provider")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(anyhow!("provider rejected validation ({status}): {body}"));
    }

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
        "response_format": { "type": "json_object" },
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
        "response_format": { "type": "json_object" },
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
        "response_format": { "type": "json_object" },
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
    if settings.api_key.trim().is_empty() {
        return Err(anyhow!("API key is required when AI is enabled"));
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
    let response = client
        .post(url)
        .bearer_auth(&settings.api_key)
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
    let value = json
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("provider response did not contain a message content field"))?;
    Ok(value.to_string())
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

    serde_json::from_str(cleaned).context("AI response was not valid JSON")
}

#[derive(Debug, Deserialize)]
struct FlashcardEnvelope {
    cards: Vec<FlashcardCard>,
}
