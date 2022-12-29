use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum FileType {
  JSON,
}

impl Display for FileType {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      FileType::JSON => write!(f, "JSON"),
    }
  }
}

#[derive(Debug)]
pub struct Error {
  pub filetype: FileType,
  pub message:  String,
  pub index:    usize,
}

impl Error {
  pub fn new(filetype: FileType, message: String, index: usize) -> Self {
    Self {
      filetype,
      message,
      index,
    }
  }

  pub fn json(message: String, index: usize) -> Self { Self::new(FileType::JSON, message, index) }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "ERROR: {} in {} at index {}",
      self.message, self.filetype, self.index
    )
  }
}
