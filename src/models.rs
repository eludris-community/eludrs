/// Error type alias
pub type Error<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
