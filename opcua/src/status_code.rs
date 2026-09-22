use std::fmt;

include!(concat!(env!("OUT_DIR"), "/status_code.rs"));

impl StatusCode {
    pub fn check(&self) -> Result<(), Self> {
        match self {
            StatusCode::Good => Ok(()),
            _ => Err(*self),
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
