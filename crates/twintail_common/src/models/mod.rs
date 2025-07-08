pub mod enums;
pub mod serde;

pub trait OptionalBuilder: Sized {
    fn map<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(v) => f(self, v),
            None => self,
        }
    }
}
