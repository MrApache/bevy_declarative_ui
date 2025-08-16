#[derive(Clone, PartialEq, Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
}

impl Function {
    pub fn new(name: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }
}
