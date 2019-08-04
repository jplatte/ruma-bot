use std::{
    fmt::{self, Debug},
    ops::Deref,
};

#[derive(Clone, Copy)]
pub struct State<T: Send + Sync> {
    inner: T,
}

impl<T> Debug for State<T>
where
    T: Debug + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<T> Deref for State<T>
where
    T: Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
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
