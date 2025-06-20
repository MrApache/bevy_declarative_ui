pub(super) struct Property {
    pub name: String,
    pub type_: String,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub(super) struct Module {
    pub name: String,
    pub path: String,
}


#[derive(Default)]
pub(super) struct GeneratedModule {
    pub properties: Vec<Property>,
    pub functions:  Vec<String>, //Function names
}