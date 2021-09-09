/// The wallet error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// iota.rs error.
    #[error("`{0}`")]
    ClientError(Box<iota_client::Error>),
}