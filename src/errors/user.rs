use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserCreationError {
    #[error("Account with this email already exists")]
    EmailAlreadyExists,
    #[error("Unexpected")]
    Other,
}
