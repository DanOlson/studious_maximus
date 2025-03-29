use lms::Lms;

mod lms;

pub struct App<T: Lms> {
    lms: T,
}

impl<T> App<T>
where
    T: Lms,
{
    pub fn new(lms: T) -> Self {
        Self { lms }
    }
}
