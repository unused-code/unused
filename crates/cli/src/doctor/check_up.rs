pub enum Status {
    OK(String),
    Warn(String),
    Error(String),
}

pub trait CheckUp {
    fn name(&self) -> &str;
    fn status(&self) -> Status;
}
