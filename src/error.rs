use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct NoContainersError;
impl Error for NoContainersError {}

impl Display for NoContainersError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "No running containers")
    }
}

#[derive(Debug)]
pub struct NoImagesError;
impl Error for NoImagesError {}

impl Display for NoImagesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "No images found")
    }
}

#[derive(Debug)]
pub struct NoSelectionError;
impl Error for NoSelectionError {}

impl Display for NoSelectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "No selection")
    }
}
