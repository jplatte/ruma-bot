use std::ops::Deref;

#[derive(Clone, Copy)]
pub struct State<T: Send + Sync>(pub(crate) T);

impl<T> Deref for State<T>
where
    T: Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

#[derive(Clone)]
pub struct MsgContent(pub String);

impl Deref for MsgContent {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}
