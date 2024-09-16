use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct SelectionError;

impl Error for SelectionError {}

impl Display for SelectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "It is possible that the selection is incorrectly specified.")
    }
}
