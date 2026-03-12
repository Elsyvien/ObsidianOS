use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use regex::Regex;

#[derive(Debug, Clone, Default)]
pub struct FrontmatterData {
    pub raw: Option<String>,
    pub title: Option<String>,
    pub course: Option<String>,
    pub exam_date: Option<String>,
    pub tags: Vec<String>,
    pub concepts: Vec<String>,
    pub formulas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedNote {
    pub title: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub concepts: Vec<String>,
    pub formulas: Vec<String>,
    pub frontmatter: FrontmatterData,
}

pub fn parse_markdown(file_stem: &str, content: &str) -> ParsedNote {
    let (frontmatter, body) = extract_frontmatter(content);
    let headings = collect_headings(body);
    let links = collect_wikilinks(body);
    let inline_tags = collect_tags(body);
    let formulas = collect_formulas(body, &frontmatter.formulas);
    let concepts = collect_concepts(file_stem, body, &frontmatter, &headings);
    let excerpt = collect_excerpt(body);

    let mut tags = BTreeSet::new();
    for tag in frontmatter.tags.iter().chain(inline_tags.iter()) {
        tags.insert(normalize_display(tag));
    }

    ParsedNote {
        title: frontmatter
            .title
            .clone()
            .unwrap_or_else(|| normalize_display(file_stem)),
        excerpt,
        headings,
        links,
        tags: tags.into_iter().collect(),
        concepts,
        formulas,
        frontmatter,
    }
}

fn extract_frontmatter(content: &str) -> (FrontmatterData, &str) {
    let trimmed = content.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
        return (FrontmatterData::default(), trimmed);
    }

    let mut lines = trimmed.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        return (FrontmatterData::default(), trimmed);
    }

    let mut raw_lines = Vec::new();
    let mut body_start = 0usize;
    let mut consumed = first.len() + 1;

    for line in lines {
        if line.trim() == "---" {
            body_start = consumed + line.len() + 1;
            break;
        }
        raw_lines.push(line.to_string());
        consumed += line.len() + 1;
    }

    if body_start == 0 {
        return (FrontmatterData::default(), trimmed);
    }

    let raw = raw_lines.join("\n");
    let data = parse_frontmatter_lines(&raw);
    let body = trimmed.get(body_start..).unwrap_or("").trim_start();
    (data, body)
}

fn parse_frontmatter_lines(raw: &str) -> FrontmatterData {
    let mut data = FrontmatterData {
        raw: Some(raw.to_string()),
        ..FrontmatterData::default()
    };
    let mut current_list_key: Option<String> = None;

    for raw_line in raw.lines() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            current_list_key = None;
            continue;
        }

        if let Some(key) = current_list_key.as_deref() {
            let trimmed = line.trim_start();
            if let Some(item) = trimmed.strip_prefix("- ") {
                push_frontmatter_value(&mut data, key, item.trim());
                continue;
            }
        }

        current_list_key = None;
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_lowercase();
        let value = value.trim();

        if value.is_empty() {
            current_list_key = Some(key);
            continue;
        }

        if value.starts_with('[') && value.ends_with(']') {
            let inner = &value[1..value.len() - 1];
            for item in inner.split(',') {
                push_frontmatter_value(&mut data, &key, item.trim());
            }
            continue;
        }

        push_frontmatter_value(&mut data, &key, value);
    }

    data
}

fn push_frontmatter_value(data: &mut FrontmatterData, key: &str, raw_value: &str) {
    let value = raw_value.trim().trim_matches('"').trim_matches('\'');
    if value.is_empty() {
        return;
    }

    match key {
        "title" => data.title = Some(value.to_string()),
        "course" => data.course = Some(value.to_string()),
        "examdate" | "exam_date" | "exam-date" => data.exam_date = Some(value.to_string()),
        "tag" | "tags" => data.tags.push(value.to_string()),
        "concept" | "concepts" => data.concepts.push(value.to_string()),
        "formula" | "formulas" => data.formulas.push(value.to_string()),
        _ => {}
    }
}

fn collect_headings(body: &str) -> Vec<String> {
    static HEADING_RE: OnceLock<Regex> = OnceLock::new();
    let regex =
        HEADING_RE.get_or_init(|| Regex::new(r"(?m)^#{1,6}\s+(.+)$").expect("heading regex"));
    regex
        .captures_iter(body)
        .filter_map(|capture| capture.get(1).map(|value| clean_token(value.as_str())))
        .filter(|value| !value.is_empty())
        .collect()
}

fn collect_wikilinks(body: &str) -> Vec<String> {
    static LINK_RE: OnceLock<Regex> = OnceLock::new();
    let regex = LINK_RE.get_or_init(|| Regex::new(r"\[\[([^\]]+)\]\]").expect("link regex"));
    let mut links = BTreeSet::new();

    for capture in regex.captures_iter(body) {
        let Some(value) = capture.get(1) else {
            continue;
        };
        let normalized = value
            .as_str()
            .split('|')
            .next()
            .unwrap_or_default()
            .split('#')
            .next()
            .unwrap_or_default()
            .trim()
            .replace('\\', "/");

        if !normalized.is_empty() {
            links.insert(normalized);
        }
    }

    links.into_iter().collect()
}

fn collect_tags(body: &str) -> Vec<String> {
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    let regex =
        TAG_RE.get_or_init(|| Regex::new(r"(?m)(^|\s)#([A-Za-z][\w/-]+)").expect("tag regex"));
    let mut tags = BTreeSet::new();

    for capture in regex.captures_iter(body) {
        let Some(value) = capture.get(2) else {
            continue;
        };
        tags.insert(normalize_display(value.as_str()));
    }

    tags.into_iter().collect()
}

fn collect_formulas(body: &str, frontmatter_formulas: &[String]) -> Vec<String> {
    static BLOCK_RE: OnceLock<Regex> = OnceLock::new();
    static INLINE_RE: OnceLock<Regex> = OnceLock::new();
    let block_regex =
        BLOCK_RE.get_or_init(|| Regex::new(r"(?s)\$\$(.+?)\$\$").expect("block formula regex"));
    let inline_regex = INLINE_RE
        .get_or_init(|| Regex::new(r"\$([^$\n][^$\n]+?)\$").expect("inline formula regex"));

    let mut formulas = BTreeSet::new();
    for item in frontmatter_formulas {
        let cleaned = clean_formula(item);
        if should_keep_formula(&cleaned) {
            formulas.insert(cleaned);
        }
    }

    for capture in block_regex.captures_iter(body) {
        if let Some(value) = capture.get(1) {
            let cleaned = clean_formula(value.as_str());
            if should_keep_formula(&cleaned) {
                formulas.insert(cleaned);
            }
        }
    }

    for capture in inline_regex.captures_iter(body) {
        if let Some(value) = capture.get(1) {
            let cleaned = clean_formula(value.as_str());
            if should_keep_formula(&cleaned) {
                formulas.insert(cleaned);
            }
        }
    }

    formulas.into_iter().collect()
}

pub fn should_keep_formula(value: &str) -> bool {
    let cleaned = clean_formula(value);
    if cleaned.is_empty() {
        return false;
    }

    let compact = cleaned.replace(' ', "");
    if compact.len() < 3 {
        return false;
    }

    if let Some((left, relation, right)) = split_relation(&cleaned) {
        return !is_trivial_relation(left, relation, right);
    }

    false
}

fn has_structural_math(value: &str) -> bool {
    const COMPLEX_COMMANDS: [&str; 12] = [
        "\\frac",
        "\\sum",
        "\\prod",
        "\\int",
        "\\sqrt",
        "\\lim",
        "\\log",
        "\\sin",
        "\\cos",
        "\\tan",
        "\\begin",
        "\\det",
    ];

    COMPLEX_COMMANDS.iter().any(|command| value.contains(command))
        || value.contains(['^', '_', '+', '*', '/'])
}

fn split_relation(value: &str) -> Option<(&str, &str, &str)> {
    const RELATIONS: [&str; 12] = [
        "\\notin",
        "\\subseteq",
        "\\subset",
        "\\leq",
        "\\geq",
        "\\neq",
        "<=",
        ">=",
        "!=",
        "\\in",
        "=",
        "<",
    ];

    for relation in RELATIONS {
        if let Some((left, right)) = value.split_once(relation) {
            let left = left.trim();
            let right = right.trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, relation, right));
            }
        }
    }

    if let Some((left, right)) = value.split_once('>') {
        let left = left.trim();
        let right = right.trim();
        if !left.is_empty() && !right.is_empty() {
            return Some((left, ">", right));
        }
    }

    None
}

fn is_trivial_relation(left: &str, relation: &str, right: &str) -> bool {
    matches!(relation, "=" | "!=" | "\\neq" | "<" | ">" | "<=" | ">=" | "\\leq" | "\\geq" | "\\in" | "\\notin" | "\\subset" | "\\subseteq")
        && is_simple_formula_atom(left)
        && is_simple_formula_atom(right)
}

fn is_simple_formula_atom(value: &str) -> bool {
    let stripped = value
        .trim()
        .trim_matches(|character| matches!(character, '(' | ')' | '[' | ']' | '{' | '}'));
    if stripped.is_empty() {
        return true;
    }

    let unsigned = stripped
        .strip_prefix('+')
        .or_else(|| stripped.strip_prefix('-'))
        .unwrap_or(stripped)
        .trim();
    if unsigned.is_empty() {
        return true;
    }

    if has_structural_math(unsigned) {
        return false;
    }

    if unsigned.contains("\\left") || unsigned.contains("\\right") {
        return false;
    }

    unsigned
        .chars()
        .all(|character| character.is_alphanumeric() || matches!(character, '\\' | '.' | ',' | '{' | '}'))
}

fn collect_concepts(
    file_stem: &str,
    body: &str,
    frontmatter: &FrontmatterData,
    headings: &[String],
) -> Vec<String> {
    static BOLD_RE: OnceLock<Regex> = OnceLock::new();
    static DEFINITION_RE: OnceLock<Regex> = OnceLock::new();
    let bold_regex =
        BOLD_RE.get_or_init(|| Regex::new(r"\*\*([^*\n]{3,80})\*\*").expect("bold regex"));
    let definition_regex = DEFINITION_RE
        .get_or_init(|| Regex::new(r"(?m)^([A-Z][^:\n]{2,64}):\s+").expect("definition regex"));

    let mut concepts = BTreeSet::new();
    concepts.insert(normalize_display(file_stem));

    for concept in &frontmatter.concepts {
        concepts.insert(normalize_display(concept));
    }

    for heading in headings {
        if should_keep_concept(heading) {
            concepts.insert(normalize_display(heading));
        }
    }

    for capture in bold_regex.captures_iter(body) {
        if let Some(value) = capture.get(1) {
            let cleaned = clean_token(value.as_str());
            if should_keep_concept(&cleaned) {
                concepts.insert(cleaned);
            }
        }
    }

    for capture in definition_regex.captures_iter(body) {
        if let Some(value) = capture.get(1) {
            let cleaned = clean_token(value.as_str());
            if should_keep_concept(&cleaned) {
                concepts.insert(cleaned);
            }
        }
    }

    concepts.into_iter().collect()
}

fn collect_excerpt(body: &str) -> String {
    let excerpt = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("- "))
        .collect::<Vec<_>>()
        .join(" ");

    let excerpt = excerpt
        .replace("\\ (", "\\(")
        .replace("\\ )", "\\)")
        .replace("\\ [", "\\[")
        .replace("\\ ]", "\\]");

    let mut truncated = excerpt.chars().take(280).collect::<String>();
    while truncated.ends_with([' ', '\\']) {
        truncated.pop();
    }

    truncated
}

fn should_keep_concept(value: &str) -> bool {
    let lowered = value.to_lowercase();
    !matches!(
        lowered.as_str(),
        "overview" | "summary" | "introduction" | "references" | "example" | "examples"
    ) && value.len() >= 3
}

pub fn normalize_key(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else if !normalized.ends_with(' ') {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clean_token(value: &str) -> String {
    value
        .trim()
        .trim_matches('*')
        .trim_matches('_')
        .trim_matches('`')
        .replace("  ", " ")
}

fn clean_formula(value: &str) -> String {
    value
        .trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_display(value: &str) -> String {
    let cleaned = clean_token(value);
    let mut words = Vec::new();
    for word in cleaned.split_whitespace() {
        let lowered = word.trim_matches(|character: char| !character.is_alphanumeric());
        if !lowered.is_empty() {
            words.push(lowered.to_string());
        }
    }
    words.join(" ")
}

pub fn note_title_candidates(title: &str, relative_path: &str) -> HashMap<String, String> {
    let mut candidates = HashMap::new();
    candidates.insert(normalize_key(title), relative_path.to_string());

    let relative = relative_path.replace('\\', "/");
    candidates.insert(normalize_key(&relative), relative.clone());

    if let Some(file_name) = relative.rsplit('/').next() {
        let stem = file_name.trim_end_matches(".md");
        candidates.insert(normalize_key(stem), relative.clone());
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::{normalize_key, parse_markdown};

    #[test]
    fn parses_frontmatter_lists_links_and_math() {
        let content = r#"---
title: Graph Theory
exam_date: 2026-06-18
tags:
  - discrete
  - exam
concepts: [Euler path, Hamiltonian cycle]
---

# Graphs

**Adjacency matrix** stores the edge relation.
Graph Connectivity: describes how vertices are reachable.
See [[Trees|tree notes]] and [[proofs/Connectivity#examples]].

Inline formula $A = D^{-1}L$ and block:
$$
\deg(v) = \sum_{u \in V} a_{uv}
$$
"#;

        let parsed = parse_markdown("graph-theory", content);
        assert_eq!(parsed.title, "Graph Theory");
        assert!(parsed.headings.iter().any(|value| value == "Graphs"));
        assert!(parsed.links.iter().any(|value| value == "Trees"));
        assert!(parsed
            .links
            .iter()
            .any(|value| value == "proofs/Connectivity"));
        assert!(parsed.tags.iter().any(|value| value == "discrete"));
        assert!(parsed.concepts.iter().any(|value| value == "Euler path"));
        assert!(parsed
            .concepts
            .iter()
            .any(|value| value == "Adjacency matrix"));
        assert!(parsed.formulas.iter().any(|value| value.contains("deg(v)")));
        assert_eq!(normalize_key("Graph Theory"), "graph theory");
    }

    #[test]
    fn filters_trivial_math_snippets_from_formula_library() {
        let content = r#"
Frontmatter formula list:

Inline snippets $\epsilon > 0$, $a \in U$, $k \in N$, and $x = -1$ should stay out.
Useful formulas like $y = mx + b$ and $f(x) = x^2$ should remain.
Standalone math like $\mathbb{R}^n$ and markdown noise like $**stetig** in$ should also stay out.

$$
\sum_{k=1}^{n} k = \frac{n(n+1)}{2}
$$
"#;

        let parsed = parse_markdown("analysis", content);

        assert!(parsed.formulas.iter().any(|value| value == "y = mx + b"));
        assert!(parsed.formulas.iter().any(|value| value == "f(x) = x^2"));
        assert!(parsed
            .formulas
            .iter()
            .any(|value| value.contains("\\sum_{k=1}^{n} k = \\frac{n(n+1)}{2}")));
        assert!(!parsed.formulas.iter().any(|value| value == "\\epsilon > 0"));
        assert!(!parsed.formulas.iter().any(|value| value == "a \\in U"));
        assert!(!parsed.formulas.iter().any(|value| value == "k \\in N"));
        assert!(!parsed.formulas.iter().any(|value| value == "x = -1"));
        assert!(!parsed.formulas.iter().any(|value| value == "\\mathbb{R}^n"));
        assert!(!parsed.formulas.iter().any(|value| value == "**stetig** in"));
    }
}
