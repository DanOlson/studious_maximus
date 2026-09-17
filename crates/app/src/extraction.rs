use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PARSER_VERSION: &str = "front-page-deterministic-v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExtractedFrontPageItem {
    pub title: String,
    pub all_day_date: String,
    pub evidence: String,
    pub confidence: f64,
}

pub fn content_hash(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn normalize_html(html: &str) -> String {
    let mut output = String::new();
    let mut tag = String::new();
    let mut in_tag = false;
    let mut skip_until: Option<&'static str> = None;

    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                let lower = tag.trim().to_ascii_lowercase();
                let name = lower
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                match name {
                    "script" if !lower.starts_with('/') => skip_until = Some("script"),
                    "style" if !lower.starts_with('/') => skip_until = Some("style"),
                    "script" | "style" if lower.starts_with('/') => skip_until = None,
                    "br" | "p" | "div" | "tr" | "li" | "h1" | "h2" | "h3" | "h4" | "table" => {
                        push_space(&mut output, '\n')
                    }
                    "td" | "th" => push_space(&mut output, ' '),
                    _ => {}
                }
                tag.clear();
                in_tag = false;
            } else {
                tag.push(ch);
            }
            continue;
        }

        if ch == '<' {
            in_tag = true;
            continue;
        }

        if skip_until.is_none() {
            output.push(ch);
        }
    }

    decode_basic_entities(&output)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn extract_deterministic_items(
    html: &str,
    school_year_start: i32,
) -> Vec<ExtractedFrontPageItem> {
    let text = normalize_html(html);
    let mut items = Vec::new();

    for line in text.lines() {
        for date in find_numeric_dates(line, school_year_start) {
            items.push(item_from_line(line, date));
        }
        for date in find_month_name_dates(line, school_year_start) {
            let item = item_from_line(line, date);
            if !items.iter().any(|existing: &ExtractedFrontPageItem| {
                existing.all_day_date == item.all_day_date && existing.evidence == item.evidence
            }) {
                items.push(item);
            }
        }
    }

    items
}

fn item_from_line(line: &str, date: NaiveDate) -> ExtractedFrontPageItem {
    ExtractedFrontPageItem {
        title: concise_title(line),
        all_day_date: date.to_string(),
        evidence: line.trim().to_string(),
        confidence: 0.8,
    }
}

fn concise_title(line: &str) -> String {
    let trimmed = line.trim();
    let without_date = trimmed
        .split(['-', '–', ':'])
        .next_back()
        .unwrap_or(trimmed)
        .trim();
    let title = if without_date.len() >= 4 {
        without_date
    } else {
        trimmed
    };
    title.chars().take(120).collect()
}

fn find_numeric_dates(line: &str, default_year: i32) -> Vec<NaiveDate> {
    let bytes = line.as_bytes();
    let mut dates = Vec::new();
    for start in 0..bytes.len() {
        if !bytes[start].is_ascii_digit() {
            continue;
        }
        let Some((month, pos)) = read_number(bytes, start) else {
            continue;
        };
        if bytes.get(pos) != Some(&b'/') {
            continue;
        }
        let Some((day, pos)) = read_number(bytes, pos + 1) else {
            continue;
        };
        let mut year = default_year;
        if bytes.get(pos) == Some(&b'/')
            && let Some((parsed_year, _)) = read_number(bytes, pos + 1)
        {
            year = if parsed_year < 100 {
                2000 + parsed_year
            } else {
                parsed_year
            };
        }
        if let Some(date) = NaiveDate::from_ymd_opt(year, month as u32, day as u32) {
            dates.push(date);
        }
    }
    dates
}

fn find_month_name_dates(line: &str, default_year: i32) -> Vec<NaiveDate> {
    let cleaned = line.replace(',', " ");
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let mut dates = Vec::new();

    for window in words.windows(2) {
        let Some(month) = month_number(window[0]) else {
            continue;
        };
        let day = window[1].trim_end_matches(|c: char| !c.is_ascii_digit());
        let Ok(day) = day.parse::<u32>() else {
            continue;
        };
        if let Some(date) = NaiveDate::from_ymd_opt(default_year, month, day) {
            dates.push(date);
        }
    }

    for window in words.windows(3) {
        let Some(month) = month_number(window[0]) else {
            continue;
        };
        let Ok(day) = window[1].parse::<u32>() else {
            continue;
        };
        let Ok(year) = window[2].parse::<i32>() else {
            continue;
        };
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            dates.push(date);
        }
    }

    dates
}

fn read_number(bytes: &[u8], start: usize) -> Option<(i32, usize)> {
    let mut pos = start;
    while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
        pos += 1;
    }
    if pos == start {
        return None;
    }
    let number = std::str::from_utf8(&bytes[start..pos]).ok()?.parse().ok()?;
    Some((number, pos))
}

fn month_number(word: &str) -> Option<u32> {
    match word
        .to_ascii_lowercase()
        .trim_matches(|c: char| !c.is_ascii_alphabetic())
    {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "sept" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn decode_basic_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn push_space(output: &mut String, ch: char) {
    if !output.ends_with(ch) {
        output.push(ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_sha256() {
        assert_eq!(
            content_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn normalizes_tables_lists_and_ignores_scripts() {
        let html = "<h2>Week</h2><ul><li>Essay &amp; draft</li></ul><script>9/9</script><table><tr><td>Quiz</td><td>10/2</td></tr></table>";

        let text = normalize_html(html);

        assert!(text.contains("Week"));
        assert!(text.contains("Essay & draft"));
        assert!(text.contains("Quiz 10/2"));
        assert!(!text.contains("9/9"));
    }

    #[test]
    fn extracts_unambiguous_numeric_and_month_dates() {
        let html = "<p>10/02 - Photosynthesis quiz</p><p>Project due October 7, 2025</p>";

        let items = extract_deterministic_items(html, 2025);

        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.all_day_date == "2025-10-02"));
        assert!(items.iter().any(|item| item.all_day_date == "2025-10-07"));
    }
}
