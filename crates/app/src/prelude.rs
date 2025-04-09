use std::fmt::Debug;

pub trait Stable: Send + Sync + Debug + 'static {}
impl<T: Send + Sync + Debug + 'static> Stable for T {}
