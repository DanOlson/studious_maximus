use super::{Lms, dto};
use reqwest::{
    Client as ReqClient, Url,
    header::{HeaderMap, HeaderValue},
};

pub struct Client {
    base_url: String,
    client: ReqClient,
}

impl Client {
    pub fn new(base_url: String, token: &str) -> Self {
        let mut headers = HeaderMap::new();
        let value = format!("Bearer {token}");
        let value = HeaderValue::from_str(&value).expect("HeaderValue from auth token");
        headers.insert("Authorization", value);

        let client = ReqClient::builder()
            .connect_timeout(std::time::Duration::from_millis(5_000))
            .default_headers(headers)
            .timeout(std::time::Duration::from_millis(10_000))
            .build()
            .expect("A configured reqwest::Client");

        Self { base_url, client }
    }
}

impl Lms for Client {
    async fn get_students(&self) -> anyhow::Result<Vec<dto::Student>> {
        let url = format!("{}/api/v1/users/self/observees", self.base_url);
        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Vec<dto::Student>>()
            .await?;

        Ok(resp)
    }

    async fn get_active_courses(&self, account_id: i64) -> anyhow::Result<Vec<dto::Course>> {
        let url = format!(
            "{}/api/v1/users/{}/courses?enrollment_state=active",
            self.base_url, account_id
        );
        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Vec<dto::Course>>()
            .await?;

        Ok(resp)
    }

    async fn get_course_assignments(
        &self,
        account_id: i64,
        course_id: i64,
    ) -> anyhow::Result<Vec<dto::Assignment>> {
        let mut assignments = Vec::new();
        let url = format!(
            "{}/api/v1/users/{account_id}/courses/{course_id}/assignments",
            self.base_url,
        );

        let mut next_url = Some(Url::parse(&url)?);

        while let Some(url) = next_url {
            let resp = self.client.get(url.clone()).send().await?;
            let headers = resp.headers().clone();

            let mut page: Vec<dto::Assignment> = resp.json().await?;
            assignments.append(&mut page);

            next_url = parse_next_link(headers.get("link"));
        }

        Ok(assignments)
    }

    async fn get_course_submissions(
        &self,
        course_id: i64,
        student_id: i64,
    ) -> anyhow::Result<Vec<dto::Submission>> {
        let mut submissions = Vec::new();

        let url = format!(
            "{}/api/v1/courses/{course_id}/students/submissions?student_ids[]={student_id}",
            self.base_url,
        );

        let mut next_url = Some(Url::parse(&url)?);

        while let Some(url) = next_url {
            let resp = self.client.get(url.clone()).send().await?;
            let headers = resp.headers().clone();

            let mut page: Vec<dto::Submission> = resp.json().await?;
            submissions.append(&mut page);

            next_url = parse_next_link(headers.get("link"));
        }

        Ok(submissions)
    }
}

fn parse_next_link(link_header: Option<&reqwest::header::HeaderValue>) -> Option<Url> {
    let header_value = link_header?.to_str().ok()?;

    for part in header_value.split(',') {
        let sections: Vec<&str> = part.trim().split(';').collect();
        if sections.len() < 2 {
            continue;
        }

        let url_part = sections[0].trim();
        let rel_part = sections[1].trim();

        if rel_part == r#"rel="next""# {
            let url_str = url_part.trim_start_matches('<').trim_end_matches('>');

            return Url::parse(url_str).ok();
        }
    }

    None
}
