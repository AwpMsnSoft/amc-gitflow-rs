use anyhow::{Result as AnyResult, anyhow};
use auto_context::auto_context as anyhow_context;

pub trait IntoAnyResult<T> {
    fn into_anyresult(self) -> AnyResult<T>;
}

impl<T> IntoAnyResult<T> for Option<T> {
    #[anyhow_context]
    fn into_anyresult(self) -> AnyResult<T> {
        self.ok_or_else(|| anyhow!("called `Option::unwrap()` on a `None` value"))
    }
}
