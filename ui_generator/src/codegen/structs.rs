use std::fmt::{Display, Formatter};
use crate::codegen::Access;


pub struct Struct {
    name:    String,
    access:  Access,
    fields:  Vec<Field>,
    derives: Vec<String>,
    //TODO: impls, fields ownership
}

impl Struct {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            access: Access::None,
            name:   name.into(),
            fields: vec![],
            derives: vec![],
        }
    }

    pub const fn access(&mut self, access: Access) -> &mut Self {
        self.access = access;
        self
    }

    pub fn field(&mut self, access: Access, field_name: impl Into<String>, value_type: impl Into<String>) -> &mut Self {
        self.fields.push(Field {
            access,
            name: field_name.into(),
            value_type: value_type.into(),
        });
        self
    }

    pub fn derive(&mut self, derive: impl Into<String>) -> &mut Self {
        self.derives.push(derive.into());
        self
    }

    pub fn derives(&mut self, derives: &[&str]) -> &mut Self {
        derives.iter().for_each(|derive| {
            self.derive(derive.to_string());
        });
        self
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = &self.name;
        let access = self.access;

        let derives = if self.derives.is_empty() {
            String::new()
        } else {
            format!("#[derive({})]\n", self.derives.join(", "))
        };

        if self.fields.is_empty() {
            write!(f, "{derives}{access} struct {name};\n")
        }
        else {
            let fields = self.fields.iter().map(Field::to_string).collect::<Vec<_>>().join(",\n");
            write!(f, "{derives}{access} struct {name} {{\n{fields}\n}}\n")
        }
    }
}

pub struct Field {
    access: Access,
    name:   String,
    value_type: String
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let access = self.access;
        let name = &self.name;
        let value_type = &self.value_type;
        write!(f, "{access} {name}: {value_type}")
    }
}