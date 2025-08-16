use crate::codegen::static_field::StaticField;
use crate::codegen::structs::Struct;
use crate::codegen::using::Using;
use crate::codegen::{Access, Function};
use std::fmt::{Display, Formatter};

pub struct Module {
    //TODO: outer access
    name: String,
    access: Access,
    usings: Vec<Using>,
    fields: Vec<StaticField>,
    structs: Vec<Struct>,
    functions: Vec<Function>,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            access: Access::None,
            usings: vec![],
            fields: vec![],
            structs: vec![],
            functions: vec![],
        }
    }

    pub const fn access(&mut self, access: Access) -> &mut Self {
        self.access = access;
        self
    }

    pub fn with_using(&mut self, using: impl Into<String>) -> &mut Self {
        self.usings.push(Using::new(using.into()));
        self
    }

    pub fn with_struct(&mut self, s: Struct) -> &mut Self {
        self.structs.push(s);
        self
    }

    pub fn with_function(&mut self, f: Function) -> &mut Self {
        self.functions.push(f);
        self
    }

    pub fn with_field(
        &mut self,
        name: impl Into<String>,
        r#type: impl Into<String>,
        default: impl Into<String>,
    ) -> &mut Self {
        self.fields.push(StaticField::new(name, r#type, default));
        self
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.access)?;
        writeln!(f, " mod {} {{", &self.name)?;

        let usings = self
            .usings
            .iter()
            .map(Using::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let fields = self
            .fields
            .iter()
            .map(StaticField::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let structs = &self
            .structs
            .iter()
            .map(Struct::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let functions = &self
            .functions
            .iter()
            .map(Function::to_string)
            .collect::<Vec<_>>()
            .join("\n\n");

        if !usings.is_empty() {
            writeln!(f, "{usings}")?;
        }

        if !fields.is_empty() {
            writeln!(f, "{fields}")?;
        }

        if !structs.is_empty() {
            writeln!(f, "{structs}")?;
        }

        if !functions.is_empty() {
            write!(f, "{functions}")?;
        }

        write!(f, "\n}}")
    }
}
