use thiserror::Error;

#[derive(Debug, Error)]
pub struct PosixError(pub i32);

impl std::fmt::Display for PosixError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Posix Error: {}", self.0)
    }
}
