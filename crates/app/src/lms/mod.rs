mod canvas;
mod dto;

pub trait Lms {
    async fn get_students(&self) -> anyhow::Result<Vec<dto::Student>>;
}
