use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use crate::models::{
    ChatMessageRole, ChatScope, ExamAnswerValue, ExamBuilderInput, ExamQuestionType, ExamVerdict,
    FormulaCoach, FormulaDerivation, FormulaPractice,
};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamGenerationNoteInput {
    pub note_id: String,
    pub title: String,
    pub relative_path: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub concepts: Vec<String>,
    pub formulas: Vec<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedExamPayload {
    pub instructions: String,
    pub summary: String,
    pub questions: Vec<GeneratedExamQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedExamQuestion {
    #[serde(rename = "type")]
    pub question_type: ExamQuestionType,
    pub prompt: String,
    pub options: Vec<String>,
    pub correct_answer: String,
    pub explanation: String,
    pub source_note_id: String,
    pub source_note_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamGradingQuestionInput {
    pub question_id: String,
    pub index: usize,
    #[serde(rename = "type")]
    pub question_type: ExamQuestionType,
    pub prompt: String,
    pub options: Vec<String>,
    pub source_note_id: String,
    pub source_note_title: String,
    pub expected_answer: String,
    pub explanation: String,
    pub user_answer: ExamAnswerValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradedExamPayload {
    pub overall_feedback: String,
    pub results: Vec<GradedExamQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradedExamQuestion {
    pub question_id: String,
    pub verdict: ExamVerdict,
    pub expected_answer: String,
    pub explanation: String,
    pub feedback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaBriefPayload {
    pub coach: FormulaCoach,
    pub practice: FormulaPractice,
    pub derivation: FormulaDerivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatContextChunkInput {
    pub citation_id: String,
    pub note_id: String,
    pub note_title: String,
    pub relative_path: String,
    pub heading_path: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAnswerPayload {
    pub answer: String,
    pub citation_ids: Vec<String>,
    pub used_fallback: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProviderErrorEnvelope {
    error: Option<ProviderErrorBody>,
}

#[derive(Debug, Deserialize)]
struct ProviderErrorBody {
    message: Option<String>,
    metadata: Option<ProviderErrorMetadata>,
}

#[derive(Debug, Deserialize)]
struct ProviderErrorMetadata {
    raw: Option<String>,
    provider_name: Option<String>,
}

pub fn validate_settings(settings: &AiProviderSettings) -> Result<String> {
    if !settings.enabled {
        return Ok("AI is disabled; local parsing remains active.".to_string());
    }

    ensure_required(settings)?;
    if settings.base_url.starts_with("mock://") {
        return Ok(format!(
            "Validated {} against mock AI provider {}",
            settings.model, settings.base_url
        ));
    }

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

pub fn generate_exam(
    settings: &AiProviderSettings,
    course_name: &str,
    input: &ExamBuilderInput,
    notes: &[ExamGenerationNoteInput],
) -> Result<GeneratedExamPayload> {
    ensure_required(settings)?;
    if notes.is_empty() {
        return Err(anyhow!("exam source queue is empty"));
    }
    if settings.base_url.starts_with("mock://") {
        return build_mock_exam(course_name, input, notes);
    }

    let note_ids = notes
        .iter()
        .map(|note| note.note_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let notes_payload = notes
        .iter()
        .map(|note| {
            format!(
                "noteId: {}\ntitle: {}\npath: {}\nheadings: {}\nconcepts: {}\nformulas: {}\nlinks: {}\nexcerpt: {}",
                note.note_id,
                note.title,
                note.relative_path,
                note.headings.join(" | "),
                note.concepts.join(", "),
                note.formulas.join(" | "),
                note.links.join(", "),
                note.excerpt
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    let prompt = format!(
        "Generate a study exam for the course `{course_name}`. Return strict JSON only with keys \
         `instructions`, `summary`, and `questions`. `questions` must be an array of exactly {} items. \
         Produce exactly {} `multiple-choice` questions and exactly {} `short-answer` questions. \
         Every question object must contain `type`, `prompt`, `options`, `correctAnswer`, `explanation`, \
         `sourceNoteId`, and `sourceNoteTitle`. `sourceNoteId` must be one of [{note_ids}]. \
         For `multiple-choice`, provide exactly 4 options and make `correctAnswer` match one option exactly. \
         For `short-answer`, set `options` to an empty array. Difficulty: {:?}. Time limit: {} minutes. \
         Build the exam only from the provided notes.",
        input.multiple_choice_count + input.short_answer_count,
        input.multiple_choice_count,
        input.short_answer_count,
        input.difficulty,
        input.time_limit_minutes
    );

    let payload = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": "You generate production-grade exams and return JSON only." },
            { "role": "user", "content": prompt },
            { "role": "user", "content": notes_payload }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    let exam: GeneratedExamPayload = parse_json_blob(&content)?;
    validate_generated_exam(input, notes, exam)
}

pub fn grade_exam_attempt(
    settings: &AiProviderSettings,
    course_name: &str,
    exam_title: &str,
    questions: &[ExamGradingQuestionInput],
) -> Result<GradedExamPayload> {
    ensure_required(settings)?;
    if questions.is_empty() {
        return Err(anyhow!("exam has no questions to grade"));
    }
    if settings.base_url.starts_with("mock://") {
        return Ok(build_mock_grading(course_name, exam_title, questions));
    }

    let payload = json!({
        "model": settings.model,
        "temperature": 0,
        "messages": [
            {
                "role": "system",
                "content": "You are grading a study exam. Return strict JSON only."
            },
            {
                "role": "user",
                "content": format!(
                    "Grade the exam `{exam_title}` for the course `{course_name}`. Return strict JSON with keys \
                     `overallFeedback` and `results`. `results` must contain exactly one item per question. \
                     Each result must contain `questionId`, `verdict`, `expectedAnswer`, `explanation`, and `feedback`. \
                     `verdict` must be one of `correct`, `partial`, or `incorrect`. Use the provided expected answer \
                     and explanation to judge correctness. Preserve the given `questionId` values exactly."
                )
            },
            {
                "role": "user",
                "content": serde_json::to_string(questions).context("failed to serialize grading input")?
            }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    let grading: GradedExamPayload = parse_json_blob(&content)?;
    validate_grading_payload(questions, grading)
}

pub fn generate_formula_brief(
    settings: &AiProviderSettings,
    course_name: &str,
    latex: &str,
    related_concepts: &[String],
    headings: &[String],
    chunks: &[ChatContextChunkInput],
) -> Result<FormulaBriefPayload> {
    ensure_required(settings)?;

    if settings.base_url.starts_with("mock://") {
        return Ok(build_mock_formula_brief(
            course_name,
            latex,
            related_concepts,
            headings,
            chunks,
        ));
    }

    let chunk_payload = chunks
        .iter()
        .map(|chunk| {
            format!(
                "{} | {} | {} | {}\n{}",
                chunk.citation_id,
                chunk.note_title,
                chunk.relative_path,
                chunk.heading_path,
                chunk.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    let prompt = format!(
        "Generate a formula study brief for the course `{course_name}` and the formula `{latex}`. \
         Return strict JSON only with keys `coach`, `practice`, and `derivation`. \
         `coach` must contain `meaning`, `symbolBreakdown`, `useCases`, and `pitfalls`. \
         `practice` must contain `recallPrompts`, `shortAnswerDrills`, and `multipleChoiceChecks`. \
         `derivation` must contain `assumptions`, `intuition`, and `outline`. \
         Use the provided note context only.\n\
         Related concepts: {}\n\
         Headings: {}",
        related_concepts.join(", "),
        headings.join(" | ")
    );

    let payload = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            {
                "role": "system",
                "content": "You are a formula tutor. Return JSON only."
            },
            { "role": "user", "content": prompt },
            { "role": "user", "content": chunk_payload }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    let brief: FormulaBriefPayload = parse_json_blob(&content)?;
    validate_formula_brief(&brief)?;
    Ok(brief)
}

pub fn answer_chat_query(
    settings: &AiProviderSettings,
    scope: ChatScope,
    course_name: Option<&str>,
    transcript: &[(ChatMessageRole, String)],
    user_message: &str,
    chunks: &[ChatContextChunkInput],
    allow_fallback: bool,
) -> Result<ChatAnswerPayload> {
    ensure_required(settings)?;

    if settings.base_url.starts_with("mock://") {
        return Ok(build_mock_chat_answer(
            scope,
            course_name,
            user_message,
            chunks,
            allow_fallback,
        ));
    }

    let transcript_payload = transcript
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|(role, content)| match role {
            ChatMessageRole::User => format!("User: {content}"),
            ChatMessageRole::Assistant => format!("Assistant: {content}"),
        })
        .collect::<Vec<_>>()
        .join("\n");
    let chunk_payload = if chunks.is_empty() {
        "No matching note chunks were retrieved.".to_string()
    } else {
        chunks
            .iter()
            .map(|chunk| {
                format!(
                    "{} | {} | {} | {}\n{}",
                    chunk.citation_id,
                    chunk.note_title,
                    chunk.relative_path,
                    chunk.heading_path,
                    chunk.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n")
    };
    let scope_label = match scope {
        ChatScope::Course => "current course",
        ChatScope::Vault => "whole vault",
    };

    let payload = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            {
                "role": "system",
                "content": "You answer from study notes first. Return strict JSON only with keys `answer`, `citationIds`, `usedFallback`, and `fallbackReason`."
            },
            {
                "role": "user",
                "content": format!(
                    "Answer the user's question using the retrieved notes first. Scope: {scope_label}. Course: {}. \
                     {} \
                     Cite retrieved chunks by their `citationId` values in `citationIds` whenever notes support the answer.",
                    course_name.unwrap_or("vault-wide"),
                    if allow_fallback {
                        "If the retrieved notes are insufficient, you may still answer with a clearly labeled fallback by setting `usedFallback` to true and explaining why in `fallbackReason`."
                    } else {
                        "Do not use fallback knowledge. Stay within the retrieved notes."
                    }
                )
            },
            { "role": "user", "content": format!("Recent conversation:\n{transcript_payload}") },
            { "role": "user", "content": format!("Retrieved note chunks:\n{chunk_payload}") },
            { "role": "user", "content": format!("Current user message: {user_message}") }
        ]
    });

    let json = send_chat_request(settings, payload)?;
    let content = message_content(&json)?;
    let mut answer: ChatAnswerPayload = parse_json_blob(&content)?;
    validate_chat_answer(chunks, &mut answer);
    Ok(answer)
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
        .timeout(Duration::from_millis(timeout_ms.max(120_000)))
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
    let mut payload = payload;
    if let Some(object) = payload.as_object_mut() {
        object
            .entry("stream".to_string())
            .or_insert(Value::Bool(false));
    }

    let client = build_client(settings.timeout_ms)?;
    let url = endpoint(&settings.base_url, "chat/completions");
    let response = with_optional_bearer(client.post(url), &settings.api_key)
        .header(ACCEPT, "application/json")
        .json(&payload)
        .send()
        .map_err(|error| anyhow!("failed to reach chat completions endpoint: {error}"))?;

    let status = response.status();
    let body_bytes = response
        .bytes()
        .map_err(|error| anyhow!("failed to read provider response: {error}"))?;
    let body = String::from_utf8_lossy(body_bytes.as_ref())
        .trim()
        .to_string();
    if !status.is_success() {
        return Err(anyhow!("{}", format_provider_rejection(status, &body)));
    }

    serde_json::from_slice(body_bytes.as_ref()).with_context(|| {
        let preview = body.chars().take(320).collect::<String>();
        format!("provider returned invalid JSON: {preview}")
    })
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
    let extracted = extract_json_object(cleaned).unwrap_or(cleaned);

    parse_json_candidate(cleaned)
        .or_else(|_| parse_json_candidate(extracted))
        .with_context(|| {
            let preview = cleaned
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(220)
                .collect::<String>();
            format!("AI response was not valid JSON. Provider returned: {preview}")
        })
}

fn parse_json_candidate<T>(content: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let repaired = repair_json_like_blob(content);
    serde_json::from_str(content)
        .map_err(anyhow::Error::from)
        .or_else(|_| serde_json::from_str(&repaired).map_err(anyhow::Error::from))
        .or_else(|_| json5::from_str(content).map_err(anyhow::Error::from))
        .or_else(|_| json5::from_str(&repaired).map_err(anyhow::Error::from))
}

fn format_provider_rejection(status: StatusCode, body: &str) -> String {
    let preview = body.chars().take(320).collect::<String>();
    let parsed = serde_json::from_str::<ProviderErrorEnvelope>(body).ok();
    let provider_name = parsed
        .as_ref()
        .and_then(|envelope| envelope.error.as_ref())
        .and_then(|error| error.metadata.as_ref())
        .and_then(|metadata| metadata.provider_name.as_deref())
        .map(normalize_error_text)
        .filter(|value| !value.is_empty());
    let detail = parsed
        .as_ref()
        .and_then(|envelope| envelope.error.as_ref())
        .and_then(|error| {
            error
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.raw.as_deref())
                .or_else(|| error.message.as_deref())
        })
        .map(normalize_error_text)
        .filter(|value| !value.is_empty() && value != "Provider returned error");

    match status {
        StatusCode::TOO_MANY_REQUESTS => {
            let lead = provider_name
                .map(|name| format!("{name} rate-limited this request."))
                .unwrap_or_else(|| {
                    "The AI provider rate-limited this request. Retry shortly.".to_string()
                });
            match detail {
                Some(detail) => format!("{lead} {detail}"),
                None => lead,
            }
        }
        _ => match detail {
            Some(detail) => match provider_name {
                Some(name) => format!("{name} rejected the request ({status}). {detail}"),
                None => format!("Provider rejected the request ({status}). {detail}"),
            },
            None => format!("Provider rejected the request ({status}). {preview}"),
        },
    }
}

fn normalize_error_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn repair_json_like_blob(content: &str) -> String {
    let normalized = content
        .replace(['“', '”'], "\"")
        .replace(['‘', '’'], "'");
    let escaped = escape_invalid_backslashes(&normalized);
    remove_trailing_commas(&escaped)
}

fn escape_invalid_backslashes(content: &str) -> String {
    let mut result = String::with_capacity(content.len() + 16);
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut quote = '"';
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            if escaped {
                result.push(ch);
                escaped = false;
                continue;
            }

            if ch == '\\' {
                let next = chars.peek().copied();
                let valid_escape = next
                    .map(|next| is_valid_json_escape(next, quote))
                    .unwrap_or(false);
                result.push('\\');
                if !valid_escape {
                    result.push('\\');
                } else {
                    escaped = true;
                }
                continue;
            }

            if ch == quote {
                in_string = false;
            }

            result.push(ch);
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
        }

        result.push(ch);
    }

    result
}

fn is_valid_json_escape(ch: char, quote: char) -> bool {
    matches!(ch, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u') || ch == quote
}

fn remove_trailing_commas(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars = content.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_string = false;
    let mut quote = '"';
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            result.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
            result.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut lookahead = index + 1;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && matches!(chars[lookahead], '}' | ']') {
                index += 1;
                continue;
            }
        }

        result.push(ch);
        index += 1;
    }

    result
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

fn validate_generated_exam(
    input: &ExamBuilderInput,
    notes: &[ExamGenerationNoteInput],
    exam: GeneratedExamPayload,
) -> Result<GeneratedExamPayload> {
    let expected_total = input.multiple_choice_count + input.short_answer_count;
    if exam.questions.len() != expected_total {
        return Err(anyhow!(
            "AI generated {} questions, expected {}",
            exam.questions.len(),
            expected_total
        ));
    }

    let valid_note_ids = notes
        .iter()
        .map(|note| note.note_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let mut multiple_choice_count = 0usize;
    let mut short_answer_count = 0usize;

    for question in &exam.questions {
        if !valid_note_ids.contains(question.source_note_id.as_str()) {
            return Err(anyhow!(
                "AI generated a question for unknown source note {}",
                question.source_note_id
            ));
        }
        if question.prompt.trim().is_empty() {
            return Err(anyhow!("AI generated an empty exam question prompt"));
        }
        if question.correct_answer.trim().is_empty() {
            return Err(anyhow!(
                "AI generated an exam question without an answer key"
            ));
        }
        if question.explanation.trim().is_empty() {
            return Err(anyhow!(
                "AI generated an exam question without an explanation"
            ));
        }

        match question.question_type {
            ExamQuestionType::MultipleChoice => {
                multiple_choice_count += 1;
                if question.options.len() != 4 {
                    return Err(anyhow!(
                        "AI generated a multiple-choice question without exactly 4 options"
                    ));
                }
                if !question
                    .options
                    .iter()
                    .any(|option| option.trim() == question.correct_answer.trim())
                {
                    return Err(anyhow!(
                        "AI generated a multiple-choice answer key that is not in the options"
                    ));
                }
            }
            ExamQuestionType::ShortAnswer => {
                short_answer_count += 1;
                if !question.options.is_empty() {
                    return Err(anyhow!(
                        "AI generated options for a short-answer exam question"
                    ));
                }
            }
        }
    }

    if multiple_choice_count != input.multiple_choice_count {
        return Err(anyhow!(
            "AI generated {} multiple-choice questions, expected {}",
            multiple_choice_count,
            input.multiple_choice_count
        ));
    }
    if short_answer_count != input.short_answer_count {
        return Err(anyhow!(
            "AI generated {} short-answer questions, expected {}",
            short_answer_count,
            input.short_answer_count
        ));
    }

    Ok(exam)
}

fn validate_grading_payload(
    questions: &[ExamGradingQuestionInput],
    grading: GradedExamPayload,
) -> Result<GradedExamPayload> {
    if grading.results.len() != questions.len() {
        return Err(anyhow!(
            "AI returned {} grading results for {} questions",
            grading.results.len(),
            questions.len()
        ));
    }

    let expected_ids = questions
        .iter()
        .map(|question| question.question_id.as_str())
        .collect::<std::collections::HashSet<_>>();

    for result in &grading.results {
        if !expected_ids.contains(result.question_id.as_str()) {
            return Err(anyhow!(
                "AI returned grading for unknown question {}",
                result.question_id
            ));
        }
        if result.expected_answer.trim().is_empty() {
            return Err(anyhow!("AI grading result omitted the expected answer"));
        }
        if result.explanation.trim().is_empty() {
            return Err(anyhow!("AI grading result omitted the explanation"));
        }
        if result.feedback.trim().is_empty() {
            return Err(anyhow!("AI grading result omitted the feedback"));
        }
    }

    Ok(grading)
}

fn build_mock_exam(
    course_name: &str,
    input: &ExamBuilderInput,
    notes: &[ExamGenerationNoteInput],
) -> Result<GeneratedExamPayload> {
    if notes.is_empty() {
        return Err(anyhow!("exam source queue is empty"));
    }

    let note_titles = notes
        .iter()
        .map(|note| note.title.as_str())
        .collect::<Vec<_>>();
    let mut questions = Vec::new();

    for index in 0..input.multiple_choice_count {
        let note = &notes[index % notes.len()];
        let concept = note
            .concepts
            .get(index % note.concepts.len().max(1))
            .cloned()
            .unwrap_or_else(|| note.title.clone());
        let correct_answer = format!("{concept} anchors the note {}.", note.title);
        let first_alt = note.links.first().cloned().unwrap_or_else(|| {
            note_titles
                .first()
                .copied()
                .unwrap_or("the course graph")
                .to_string()
        });
        let second_alt = note
            .headings
            .first()
            .cloned()
            .unwrap_or_else(|| "an unrelated heading".to_string());
        let third_alt = note_titles
            .iter()
            .copied()
            .find(|title| *title != note.title)
            .unwrap_or("an unrelated chapter")
            .to_string();

        questions.push(GeneratedExamQuestion {
            question_type: ExamQuestionType::MultipleChoice,
            prompt: format!(
                "Which statement best matches the role of {concept} in {}?",
                note.title
            ),
            options: vec![
                correct_answer.clone(),
                format!("{concept} is disconnected from {first_alt}."),
                format!("{concept} is only a formatting label from {second_alt}."),
                format!(
                    "{concept} belongs to {third_alt} instead of {}.",
                    note.title
                ),
            ],
            correct_answer,
            explanation: format!(
                "{} frames {concept} as part of the note's core recall target.",
                note.title
            ),
            source_note_id: note.note_id.clone(),
            source_note_title: note.title.clone(),
        });
    }

    for index in 0..input.short_answer_count {
        let note = &notes[(index + input.multiple_choice_count) % notes.len()];
        let concept = note
            .concepts
            .get(index % note.concepts.len().max(1))
            .cloned()
            .unwrap_or_else(|| note.title.clone());
        let expected_answer = if note.excerpt.trim().is_empty() {
            format!(
                "{concept} should be explained with the main idea from {}.",
                note.title
            )
        } else {
            note.excerpt.clone()
        };

        questions.push(GeneratedExamQuestion {
            question_type: ExamQuestionType::ShortAnswer,
            prompt: format!(
                "In one or two sentences, explain how {concept} is used in {}.",
                note.title
            ),
            options: Vec::new(),
            correct_answer: expected_answer.clone(),
            explanation: format!(
                "A strong answer should reference {concept} and tie it back to {}.",
                note.title
            ),
            source_note_id: note.note_id.clone(),
            source_note_title: note.title.clone(),
        });
    }

    Ok(GeneratedExamPayload {
        instructions: format!(
            "Answer from memory first for {course_name}. Review the linked notes only after you submit."
        ),
        summary: format!(
            "Built from {} queued notes with a {:?} difficulty mix.",
            notes.len(),
            input.difficulty
        ),
        questions,
    })
}

fn build_mock_grading(
    _course_name: &str,
    _exam_title: &str,
    questions: &[ExamGradingQuestionInput],
) -> GradedExamPayload {
    let mut earned = 0.0f64;
    let mut results = Vec::with_capacity(questions.len());

    for question in questions {
        let verdict = mock_verdict(question);
        earned += match verdict {
            ExamVerdict::Correct => 1.0,
            ExamVerdict::Partial => 0.5,
            ExamVerdict::Incorrect => 0.0,
        };

        results.push(GradedExamQuestion {
            question_id: question.question_id.clone(),
            verdict,
            expected_answer: question.expected_answer.clone(),
            explanation: question.explanation.clone(),
            feedback: match verdict {
                ExamVerdict::Correct => {
                    "Correct. Keep this idea in your active recall rotation.".to_string()
                }
                ExamVerdict::Partial => {
                    "Partly right. Tighten the wording and the exact definition.".to_string()
                }
                ExamVerdict::Incorrect => {
                    "Incorrect. Bring this note back into focused review.".to_string()
                }
            },
        });
    }

    let score = if questions.is_empty() {
        0.0
    } else {
        (earned / questions.len() as f64) * 100.0
    };
    let overall_feedback = if score >= 80.0 {
        "Strong result. You can retire the clean notes and queue one harder mixed exam next."
    } else if score >= 60.0 {
        "Decent base, but the weaker notes still need another pass before you compress the material."
    } else {
        "This exam exposed real gaps. Move the missed notes into review and retry a shorter exam."
    };

    GradedExamPayload {
        overall_feedback: overall_feedback.to_string(),
        results,
    }
}

fn mock_verdict(question: &ExamGradingQuestionInput) -> ExamVerdict {
    match question.question_type {
        ExamQuestionType::MultipleChoice => {
            if normalize_answer_value(&question.user_answer)
                == normalize_text(&question.expected_answer)
            {
                ExamVerdict::Correct
            } else {
                ExamVerdict::Incorrect
            }
        }
        ExamQuestionType::ShortAnswer => {
            let user = normalize_answer_value(&question.user_answer);
            let expected = normalize_text(&question.expected_answer);
            if user.is_empty() {
                return ExamVerdict::Incorrect;
            }
            if user == expected || expected.contains(&user) || user.contains(&expected) {
                return ExamVerdict::Correct;
            }

            let keywords = question
                .expected_answer
                .split(|ch: char| !ch.is_alphanumeric())
                .map(normalize_text)
                .filter(|token| token.len() >= 5)
                .collect::<Vec<_>>();
            if keywords
                .iter()
                .filter(|token| !token.is_empty() && user.contains(token.as_str()))
                .take(2)
                .count()
                >= 1
            {
                ExamVerdict::Partial
            } else {
                ExamVerdict::Incorrect
            }
        }
    }
}

fn normalize_answer_value(value: &ExamAnswerValue) -> String {
    match value {
        ExamAnswerValue::Text(text) => normalize_text(text),
        ExamAnswerValue::Many(values) => values
            .iter()
            .map(|value| normalize_text(value))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_chat_answer(chunks: &[ChatContextChunkInput], answer: &mut ChatAnswerPayload) {
    let valid_ids = chunks
        .iter()
        .map(|chunk| chunk.citation_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    answer
        .citation_ids
        .retain(|citation_id| valid_ids.contains(citation_id.as_str()));

    if !chunks.is_empty() && answer.citation_ids.is_empty() && !answer.used_fallback {
        answer.citation_ids.push(chunks[0].citation_id.clone());
    }

    if chunks.is_empty() && !answer.used_fallback {
        answer.used_fallback = true;
        if answer.fallback_reason.is_none() {
            answer.fallback_reason =
                Some("No relevant note chunks were available for grounding.".to_string());
        }
    }
}

fn validate_formula_brief(brief: &FormulaBriefPayload) -> Result<()> {
    if brief.coach.meaning.trim().is_empty() {
        return Err(anyhow!("formula brief omitted coach meaning"));
    }
    if brief.coach.symbol_breakdown.is_empty() {
        return Err(anyhow!("formula brief omitted symbol breakdown"));
    }
    if brief.practice.recall_prompts.is_empty() {
        return Err(anyhow!("formula brief omitted recall prompts"));
    }
    if brief.derivation.intuition.trim().is_empty() {
        return Err(anyhow!("formula brief omitted derivation intuition"));
    }
    Ok(())
}

fn build_mock_formula_brief(
    course_name: &str,
    latex: &str,
    related_concepts: &[String],
    headings: &[String],
    chunks: &[ChatContextChunkInput],
) -> FormulaBriefPayload {
    let anchor_heading = headings
        .first()
        .cloned()
        .unwrap_or_else(|| "Core idea".to_string());
    let anchor_chunk = chunks
        .first()
        .map(|chunk| chunk.text.clone())
        .unwrap_or_else(|| format!("{latex} appears in the scanned notes for {course_name}."));

    FormulaBriefPayload {
        coach: FormulaCoach {
            meaning: format!("{latex} is a recurring formula in {course_name}. {anchor_chunk}"),
            symbol_breakdown: if related_concepts.is_empty() {
                vec![format!("Start from the notation used around {latex}.")]
            } else {
                related_concepts
                    .iter()
                    .take(4)
                    .map(|concept| format!("Relate the symbols back to {concept}."))
                    .collect()
            },
            use_cases: vec![
                format!("Use it when solving exercises from {anchor_heading}."),
                "Say when the formula applies before substituting values.".to_string(),
            ],
            pitfalls: vec![
                "Do not memorize the symbols without the assumptions.".to_string(),
                "Check which terms stay fixed and which vary in the derivation.".to_string(),
            ],
        },
        practice: FormulaPractice {
            recall_prompts: vec![
                format!("State {latex} from memory."),
                "Explain each symbol without looking at the note.".to_string(),
            ],
            short_answer_drills: vec![
                format!("When is {latex} the right tool?"),
                format!("Which assumption breaks {latex}?"),
            ],
            multiple_choice_checks: vec![
                format!("Pick the condition under which {latex} remains valid."),
                "Choose the interpretation that matches the note context.".to_string(),
            ],
        },
        derivation: FormulaDerivation {
            assumptions: vec![
                format!("Match the assumptions listed near {anchor_heading}."),
                "Keep track of what the source note treats as given.".to_string(),
            ],
            intuition: format!(
                "The derivation should connect {latex} back to the local note context rather than presenting it as an isolated identity."
            ),
            outline: vec![
                "Start from the note definition.".to_string(),
                "Transform the expression step by step.".to_string(),
                "State the final result and when it is useful.".to_string(),
            ],
        },
    }
}

fn build_mock_chat_answer(
    scope: ChatScope,
    course_name: Option<&str>,
    user_message: &str,
    chunks: &[ChatContextChunkInput],
    allow_fallback: bool,
) -> ChatAnswerPayload {
    if chunks.is_empty() {
        return ChatAnswerPayload {
            answer: format!(
                "I could not ground this in the {} notes, so this is a general fallback: start by checking the definitions and worked examples most related to “{}”.",
                match scope {
                    ChatScope::Course => "current course",
                    ChatScope::Vault => "vault",
                },
                user_message
            ),
            citation_ids: Vec::new(),
            used_fallback: allow_fallback,
            fallback_reason: Some(
                "No relevant note chunks were retrieved from the indexed notes.".to_string(),
            ),
        };
    }

    let labels = chunks
        .iter()
        .take(2)
        .map(|chunk| format!("{} ({})", chunk.note_title, chunk.heading_path))
        .collect::<Vec<_>>()
        .join(" and ");
    ChatAnswerPayload {
        answer: format!(
            "From the {} notes{}, the strongest support comes from {}. Those notes suggest focusing on the exact definition first, then the linked example pattern before extending the idea.",
            match scope {
                ChatScope::Course => "current course",
                ChatScope::Vault => "vault",
            },
            course_name
                .map(|name| format!(" for {name}"))
                .unwrap_or_default(),
            labels
        ),
        citation_ids: chunks
            .iter()
            .take(2)
            .map(|chunk| chunk.citation_id.clone())
            .collect(),
        used_fallback: allow_fallback && chunks.len() == 1,
        fallback_reason: if allow_fallback && chunks.len() == 1 {
            Some("Only one strongly relevant note chunk was retrieved.".to_string())
        } else {
            None
        },
    }
}

#[derive(Debug, Deserialize)]
struct FlashcardEnvelope {
    cards: Vec<FlashcardCard>,
}

#[cfg(test)]
mod tests {
    use super::{
        answer_chat_query, extract_json_object, format_provider_rejection, generate_exam,
        generate_formula_brief, grade_exam_attempt, message_content, parse_json_blob,
        AiProviderSettings,
        ChatContextChunkInput, ExamGenerationNoteInput, ExamGradingQuestionInput,
    };
    use crate::models::{
        ChatMessageRole, ChatScope, ExamAnswerValue, ExamBuilderInput, ExamDifficulty, ExamPreset,
        ExamQuestionType, ExamVerdict,
    };
    use reqwest::StatusCode;
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

    #[test]
    fn formats_provider_rate_limit_errors_without_raw_json_blob() {
        let message = format_provider_rejection(
            StatusCode::TOO_MANY_REQUESTS,
            r#"{"error":{"message":"Provider returned error","metadata":{"raw":"mistralai/mistral-small-3.1-24b-instruct:free is temporarily rate-limited upstream. Please retry shortly.","provider_name":"Venice"}}}"#,
        );

        assert_eq!(
            message,
            "Venice rate-limited this request. mistralai/mistral-small-3.1-24b-instruct:free is temporarily rate-limited upstream. Please retry shortly."
        );
    }

    #[test]
    fn repairs_invalid_backslashes_inside_model_json_strings() {
        let payload: SamplePayload =
            parse_json_blob("{\"summary\":\"Use \\sum normally\"}").expect("payload");
        assert_eq!(
            payload,
            SamplePayload {
                summary: "Use \\sum normally".to_string()
            }
        );
    }

    #[test]
    fn parses_json5_like_model_output_with_single_quotes_and_trailing_comma() {
        let payload: SamplePayload =
            parse_json_blob("{summary: 'ready',}").expect("payload");
        assert_eq!(
            payload,
            SamplePayload {
                summary: "ready".to_string()
            }
        );
    }

    #[test]
    fn mock_exam_generation_respects_requested_counts() {
        let settings = AiProviderSettings {
            base_url: "mock://exam".to_string(),
            model: "mock-model".to_string(),
            api_key: String::new(),
            enabled: true,
            timeout_ms: 2_000,
        };
        let input = ExamBuilderInput {
            course_id: "course-1".to_string(),
            preset: ExamPreset::Sprint,
            multiple_choice_count: 2,
            short_answer_count: 1,
            difficulty: ExamDifficulty::Mixed,
            time_limit_minutes: 10,
            generate_count: 1,
            title: None,
        };
        let notes = vec![ExamGenerationNoteInput {
            note_id: "note-1".to_string(),
            title: "Limits".to_string(),
            relative_path: "limits.md".to_string(),
            excerpt: "Limits describe how a function behaves near a point.".to_string(),
            headings: vec!["Definition".to_string()],
            concepts: vec!["limit".to_string()],
            formulas: vec!["\\lim_{x\\to a} f(x)".to_string()],
            links: vec!["continuity".to_string()],
        }];

        let payload = generate_exam(&settings, "Math", &input, &notes).expect("exam payload");
        assert_eq!(payload.questions.len(), 3);
        assert_eq!(
            payload
                .questions
                .iter()
                .filter(|question| question.question_type == ExamQuestionType::MultipleChoice)
                .count(),
            2
        );
        assert_eq!(
            payload
                .questions
                .iter()
                .filter(|question| question.question_type == ExamQuestionType::ShortAnswer)
                .count(),
            1
        );
    }

    #[test]
    fn mock_grading_returns_partial_for_close_short_answer() {
        let settings = AiProviderSettings {
            base_url: "mock://exam".to_string(),
            model: "mock-model".to_string(),
            api_key: String::new(),
            enabled: true,
            timeout_ms: 2_000,
        };
        let grading = grade_exam_attempt(
            &settings,
            "Math",
            "Sprint",
            &[ExamGradingQuestionInput {
                question_id: "question-1".to_string(),
                index: 1,
                question_type: ExamQuestionType::ShortAnswer,
                prompt: "Explain limits.".to_string(),
                options: Vec::new(),
                source_note_id: "note-1".to_string(),
                source_note_title: "Limits".to_string(),
                expected_answer: "Limits describe how a function behaves near a point.".to_string(),
                explanation: "Mention behavior near a point.".to_string(),
                user_answer: ExamAnswerValue::Text("function point".to_string()),
            }],
        )
        .expect("grading");

        assert_eq!(grading.results.len(), 1);
        assert_eq!(grading.results[0].verdict, ExamVerdict::Partial);
    }

    #[test]
    fn mock_formula_brief_includes_sections() {
        let settings = AiProviderSettings {
            base_url: "mock://formula".to_string(),
            model: "mock-model".to_string(),
            api_key: String::new(),
            enabled: true,
            timeout_ms: 2_000,
        };
        let brief = generate_formula_brief(
            &settings,
            "Math",
            "\\lim_{x\\to a} f(x)",
            &["Limits".to_string(), "Continuity".to_string()],
            &["Definition".to_string()],
            &[ChatContextChunkInput {
                citation_id: "C1".to_string(),
                note_id: "note-1".to_string(),
                note_title: "Limits".to_string(),
                relative_path: "limits.md".to_string(),
                heading_path: "Definition".to_string(),
                text: "Limits describe how a function behaves near a point.".to_string(),
            }],
        )
        .expect("brief");

        assert!(!brief.coach.meaning.is_empty());
        assert!(!brief.practice.recall_prompts.is_empty());
        assert!(!brief.derivation.outline.is_empty());
    }

    #[test]
    fn mock_chat_answer_flags_fallback_without_chunks() {
        let settings = AiProviderSettings {
            base_url: "mock://chat".to_string(),
            model: "mock-model".to_string(),
            api_key: String::new(),
            enabled: true,
            timeout_ms: 2_000,
        };
        let answer = answer_chat_query(
            &settings,
            ChatScope::Course,
            Some("Math"),
            &[(ChatMessageRole::User, "What is a limit?".to_string())],
            "What is a limit?",
            &[],
            true,
        )
        .expect("answer");

        assert!(answer.used_fallback);
        assert!(answer.citation_ids.is_empty());
    }
}
