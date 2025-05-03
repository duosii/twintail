pub mod crypt_config;
pub mod download_ab_config;
pub mod fetch_config;

pub trait OptionalBuilder: Sized {
    fn map<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(v) => f(self, v),
            None => self,
        }
    }
}
