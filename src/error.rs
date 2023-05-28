use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// an optional error type for serde
///
/// useful if you don't have an error type handy!
///
/// Basically just a string.
pub struct PseudoError(String);

impl Error for PseudoError {}

impl Debug for PseudoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Display for PseudoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl serde::de::Error for PseudoError {
    fn custom<T>(msg: T) -> Self where T: Display {
        Self(msg.to_string())
    }
}