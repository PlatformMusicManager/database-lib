use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionCreationError {
    #[error("Id already exists")]
    IdAlreadyExists,
    #[error("User not found")]
    UserNotFound,
}

#[derive(Error, Debug)]
pub enum SessionUpdateError {
    #[error("Session not found")]
    NotFound,
    #[error("Old token")]
    InvalidSerialNumber,
    #[error("Session expired")]
    Expired,
}
