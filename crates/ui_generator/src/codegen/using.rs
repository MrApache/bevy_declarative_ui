#[derive(Eq, Hash, PartialEq)]
pub struct Using {
    path: String,
}

impl Using {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl std::fmt::Display for Using {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "use {};", &self.path)
    }
}
