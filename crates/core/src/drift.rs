use std::collections::BTreeMap;

use serde::Serialize;

use crate::catalog::list_apis;
use crate::error::{KrxCliError, Result};

const KRX_BASE_URL: &str = "https://openapi.krx.co.kr";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceListEntry {
    pub category: String,
    pub detail_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ApiInventoryEntry {
    pub category: String,
    pub api_id: String,
    pub name: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangedEntry {
    pub api_id: String,
    pub fields: Vec<String>,
    pub expected: ApiInventoryEntry,
    pub actual: ApiInventoryEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DriftReport {
    pub expected_count: usize,
    pub actual_count: usize,
    pub missing_upstream: Vec<String>,
    pub missing_local: Vec<String>,
    pub changed: Vec<ChangedEntry>,
}

impl DriftReport {
    pub fn has_drift(&self) -> bool {
        !self.missing_upstream.is_empty()
            || !self.missing_local.is_empty()
            || !self.changed.is_empty()
    }
}

pub fn built_in_inventory() -> Vec<ApiInventoryEntry> {
    let mut entries: Vec<_> = list_apis()
        .iter()
        .map(|api| ApiInventoryEntry {
            category: api.category.to_string(),
            api_id: api.api_id.to_string(),
            name: api.name.to_string(),
            description: api.description.to_string(),
            path: api.path.to_string(),
        })
        .collect();
    entries.sort();
    entries
}

pub fn parse_service_list(html: &str) -> Result<Vec<ServiceListEntry>> {
    let tbody = slice_between(html, "<tbody>", "</tbody>").ok_or_else(|| {
        KrxCliError::InvalidInput("could not locate tbody in KRX service list".to_string())
    })?;

    let mut entries = Vec::new();
    let mut current_category: Option<String> = None;

    for row in tbody.split("<tr").skip(1) {
        let Some((row, _)) = row.split_once("</tr>") else {
            continue;
        };

        if let Some(category) = extract_category_cell(row) {
            current_category = Some(category);
        }

        let Some(href) = extract_anchor_href(row) else {
            continue;
        };
        let category = current_category.clone().ok_or_else(|| {
            KrxCliError::InvalidInput("service row appeared before category row".to_string())
        })?;

        entries.push(ServiceListEntry {
            category,
            detail_url: make_absolute_url(&href),
        });
    }

    if entries.is_empty() {
        return Err(KrxCliError::InvalidInput(
            "no services were parsed from KRX service list".to_string(),
        ));
    }

    Ok(entries)
}

pub fn parse_detail_page(category: &str, html: &str) -> Result<ApiInventoryEntry> {
    let path = extract_input_value(html, "path")
        .ok_or_else(|| KrxCliError::InvalidInput("missing path in KRX detail page".to_string()))?;
    let api_id = extract_input_value(html, "apiId")
        .ok_or_else(|| KrxCliError::InvalidInput("missing apiId in KRX detail page".to_string()))?;
    let name = extract_dd_after_dt(html, "API 명").ok_or_else(|| {
        KrxCliError::InvalidInput("missing API name in KRX detail page".to_string())
    })?;
    let description = extract_dd_after_dt(html, "설명").ok_or_else(|| {
        KrxCliError::InvalidInput("missing description in KRX detail page".to_string())
    })?;

    Ok(ApiInventoryEntry {
        category: category.to_string(),
        api_id,
        name,
        description,
        path,
    })
}

pub fn compare_inventories(
    expected: &[ApiInventoryEntry],
    actual: &[ApiInventoryEntry],
) -> DriftReport {
    let expected_map = expected
        .iter()
        .map(|entry| (entry.api_id.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();
    let actual_map = actual
        .iter()
        .map(|entry| (entry.api_id.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut missing_upstream = expected_map
        .keys()
        .filter(|api_id| !actual_map.contains_key(*api_id))
        .cloned()
        .collect::<Vec<_>>();
    let mut missing_local = actual_map
        .keys()
        .filter(|api_id| !expected_map.contains_key(*api_id))
        .cloned()
        .collect::<Vec<_>>();
    let mut changed = expected_map
        .iter()
        .filter_map(|(api_id, expected_entry)| {
            let actual_entry = actual_map.get(api_id)?;
            let mut fields = Vec::new();

            if expected_entry.category != actual_entry.category {
                fields.push("category".to_string());
            }
            if expected_entry.name != actual_entry.name {
                fields.push("name".to_string());
            }
            if expected_entry.path != actual_entry.path {
                fields.push("path".to_string());
            }

            if fields.is_empty() {
                None
            } else {
                Some(ChangedEntry {
                    api_id: api_id.clone(),
                    fields,
                    expected: expected_entry.clone(),
                    actual: actual_entry.clone(),
                })
            }
        })
        .collect::<Vec<_>>();

    missing_upstream.sort();
    missing_local.sort();
    changed.sort_by(|left, right| left.api_id.cmp(&right.api_id));

    DriftReport {
        expected_count: expected.len(),
        actual_count: actual.len(),
        missing_upstream,
        missing_local,
        changed,
    }
}

fn extract_category_cell(row: &str) -> Option<String> {
    let marker = "class=\"aC\">";
    let start = row.find(marker)?;
    let rest = &row[start + marker.len()..];
    let end = rest.find("</td>")?;
    Some(clean_html_text(&rest[..end]))
}

fn extract_anchor_href(row: &str) -> Option<String> {
    let marker = "<a href=\"";
    let start = row.find(marker)?;
    let rest = &row[start + marker.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_input_value(html: &str, name: &str) -> Option<String> {
    let marker = format!("name=\"{name}\"");
    let start = html.find(&marker)?;
    let rest = &html[start + marker.len()..];
    let value_marker = "value=\"";
    let value_start = rest.find(value_marker)?;
    let value_rest = &rest[value_start + value_marker.len()..];
    let value_end = value_rest.find('"')?;
    Some(clean_html_text(&value_rest[..value_end]))
}

fn extract_dd_after_dt(html: &str, label: &str) -> Option<String> {
    let marker = format!(">{label}</dt>");
    let start = html.find(&marker)?;
    let rest = &html[start + marker.len()..];
    let dd_start = rest.find("<dd")?;
    let dd_rest = &rest[dd_start..];
    let content_start = dd_rest.find('>')?;
    let content_rest = &dd_rest[content_start + 1..];
    let content_end = content_rest.find("</dd>")?;
    Some(clean_html_text(&content_rest[..content_end]))
}

fn slice_between<'a>(input: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_index = input.find(start)? + start.len();
    let rest = &input[start_index..];
    let end_index = rest.find(end)?;
    Some(&rest[..end_index])
}

fn make_absolute_url(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("{KRX_BASE_URL}{href}")
    }
}

fn clean_html_text(input: &str) -> String {
    let without_tags = strip_tags(input);
    decode_html_entities(without_tags.trim())
}

fn strip_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}
