pub(super) struct Property {
    pub name: String,
    pub type_: String,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub(super) struct Module {
    pub name: String,
    pub path: String,
}
