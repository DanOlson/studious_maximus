use super::{Lms, dto};
use reqwest::{
    Client as ReqClient,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;

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

#[derive(Deserialize)]
struct Observee {
    pub id: i32,
    pub name: String,
}

impl From<&Observee> for dto::Student {
    fn from(observee: &Observee) -> Self {
        Self {
            id: observee.id.to_string(),
            name: observee.name.clone(),
        }
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
            .json::<Vec<Observee>>()
            .await?
            .iter()
            .map(dto::Student::from)
            .collect::<Vec<dto::Student>>();

        Ok(resp)
    }
}
