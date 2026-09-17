use std::time::Duration;

use anyhow::{Context, bail};
use reqwest::{
    Client as ReqClient, Method, Response, StatusCode, Url,
    header::{HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use tokio::time::sleep;

use super::{Lms, dto};

const MAX_RETRIES: usize = 3;

pub struct Client {
    base_url: Url,
    client: ReqClient,
}

impl Client {
    pub fn new(base_url: String, token: &str) -> Self {
        let mut headers = HeaderMap::new();
        let value = format!("Bearer {token}");
        let value = HeaderValue::from_str(&value).expect("HeaderValue from auth token");
        headers.insert("Authorization", value);

        let client = ReqClient::builder()
            .connect_timeout(Duration::from_millis(5_000))
            .default_headers(headers)
            .timeout(Duration::from_millis(10_000))
            .build()
            .expect("A configured reqwest::Client");

        let base_url = Url::parse(&base_url).expect("valid Canvas base URL");
        Self { base_url, client }
    }

    async fn get_paginated<T>(&self, path: &str) -> anyhow::Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let mut items = Vec::new();
        let mut next_url = Some(self.base_url.join(path)?);

        while let Some(url) = next_url {
            let response = self.send_with_retries(Method::GET, url.clone()).await?;
            let headers = response.headers().clone();
            let mut page = response
                .json::<Vec<T>>()
                .await
                .with_context(|| format!("decoding Canvas response from {url}"))?;
            items.append(&mut page);
            next_url = parse_next_link(headers.get("link"));
        }

        Ok(items)
    }

    async fn send_with_retries(&self, method: Method, url: Url) -> anyhow::Result<Response> {
        let mut attempt = 0;
        loop {
            let response = self
                .client
                .request(method.clone(), url.clone())
                .send()
                .await
                .with_context(|| format!("sending Canvas request to {url}"))?;

            if response.status().is_success() {
                return Ok(response);
            }

            if !is_retryable(response.status()) || attempt >= MAX_RETRIES {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                bail!("Canvas request failed: {status} {url}: {body}");
            }

            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or_else(|| Duration::from_millis(250 * 2_u64.pow(attempt as u32)));
            sleep(retry_after).await;
            attempt += 1;
        }
    }
}

impl Lms for Client {
    async fn get_students(&self) -> anyhow::Result<Vec<dto::Student>> {
        self.get_paginated("/api/v1/users/self/observees?per_page=100")
            .await
            .context("fetching Canvas observees")
    }

    async fn get_active_courses(&self, account_id: i64) -> anyhow::Result<Vec<dto::Course>> {
        self.get_paginated(&format!(
            "/api/v1/users/{account_id}/courses?enrollment_state=active&per_page=100"
        ))
        .await
        .with_context(|| format!("fetching active Canvas courses for observed user {account_id}"))
    }

    async fn get_course_assignments(
        &self,
        account_id: i64,
        course_id: i64,
    ) -> anyhow::Result<Vec<dto::Assignment>> {
        self.get_paginated(&format!(
            "/api/v1/users/{account_id}/courses/{course_id}/assignments?include[]=overrides&per_page=100"
        ))
        .await
        .with_context(|| {
            format!(
                "fetching Canvas assignments for observed user {account_id} in course {course_id}"
            )
        })
    }

    async fn get_course_submissions(
        &self,
        course_id: i64,
        student_id: i64,
    ) -> anyhow::Result<Vec<dto::Submission>> {
        self.get_paginated(&format!(
            "/api/v1/courses/{course_id}/students/submissions?student_ids[]={student_id}&per_page=100"
        ))
        .await
        .with_context(|| {
            format!("fetching Canvas submissions for student {student_id} in course {course_id}")
        })
    }
}

fn is_retryable(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn parse_next_link(link_header: Option<&HeaderValue>) -> Option<Url> {
    let header_value = link_header?.to_str().ok()?;

    for part in header_value.split(',') {
        let mut sections = part.trim().split(';');
        let url_part = sections.next()?.trim();
        let is_next = sections.any(|section| section.trim() == r#"rel="next""#);

        if is_next {
            let url_str = url_part.trim_start_matches('<').trim_end_matches('>');
            return Url::parse(url_str).ok();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_next_link_finds_next_regardless_of_order() {
        let header = HeaderValue::from_static(
            r#"<https://canvas.example/api/v1/things?page=1>; rel="current", <https://canvas.example/api/v1/things?page=2>; rel="next""#,
        );

        let next = parse_next_link(Some(&header)).unwrap();

        assert_eq!(next.as_str(), "https://canvas.example/api/v1/things?page=2");
    }

    #[test]
    fn parse_next_link_returns_none_without_next() {
        let header = HeaderValue::from_static(
            r#"<https://canvas.example/api/v1/things?page=1>; rel="current""#,
        );

        assert!(parse_next_link(Some(&header)).is_none());
    }
}
