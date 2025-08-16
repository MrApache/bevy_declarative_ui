mod file;
mod module;
mod structs;
mod static_field;
mod using;

pub use structs::*;
pub use module::*;
pub use file::*;

use std::fmt::{Display, Formatter};
use itertools::Itertools;
use bevy_declarative_ui_parser::values::bindings::filter::Filters;

#[derive(Default, Copy, Clone)]
pub enum Access {
    #[default]
    None,
    Public,
    Super,
    Crate
}

impl Display for Access {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::None   => Ok(()),
            Access::Public => write!(f, "pub"),
            Access::Super  => write!(f, "pub(super)"),
            Access::Crate  => write!(f, "pub(crate)"),
        }
    }
}

#[derive(Default, Copy, Clone)]
pub enum Ownership {
    #[default]
    Move,
    Ref,
    MutRef
}

impl Display for Ownership {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ownership::Move => Ok(()),
            Ownership::Ref => write!(f, "&"),
            Ownership::MutRef => write!(f, "&mut ")
        }
    }
}

pub struct Argument {
    name:      String,
    arg_type:  String,
    mutable:   bool,
    ownership: Ownership,
}

impl Argument {
    pub fn new(name: impl Into<String>, arg_type: impl Into<String>, mutable: bool, ownership: Ownership) -> Self {
        Self {
            name: name.into(),
            arg_type: arg_type.into(),
            mutable,
            ownership,
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mutable = if self.mutable { "mut" } else { "" };
        let name = &self.name;
        let arg_type = &self.arg_type;
        let ownership = self.ownership;
        write!(f, "{mutable} {name}: {ownership}{arg_type}")
    }
}

pub struct ReturnValue {
    value_type: String,
    ownership: Ownership,
}

impl Display for ReturnValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value_type = &self.value_type;
        let ownership = self.ownership;
        write!(f, "{ownership}{value_type}")
    }
}

pub struct Function {
    access: Access,
    name:   String,
    body:   String,
    args:   Vec<Argument>,
    result: Vec<ReturnValue>,
    self_arg: Option<Ownership>,
}

impl Function {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            access: Access::None,
            name:   name.into(),
            body:   String::new(),
            args:   vec![],
            result: vec![],
            self_arg: None,
        }
    }

    pub const fn access(&mut self, access: Access) -> &mut Self {
        self.access = access;
        self
    }

    pub fn arg(&mut self, arg: Argument) -> &mut Self {
        self.args.push(arg);
        self
    }

    pub fn local_mut_arg(&mut self, arg_name: impl Into<String>, arg_type: impl Into<String>) -> &mut Self {
        let arg_type = format!("Local<{}>", arg_type.into());
        let arg = Argument::new(arg_name.into(), arg_type, true, Ownership::Move);
        self.args.push(arg);
        self
    }

    pub fn query_ref_arg(&mut self, arg_name: impl Into<String>, arg_type: impl Into<String>, filters: &Filters) -> &mut Self {
        let arg_type = if filters.is_empty() {
            format!("Query<&{0}>", arg_type.into())
        }
        else {
            format!("Query<&{0}, {1}>", arg_type.into(), filters.to_filter_bundle())
        };

        let argument = Argument::new(arg_name, arg_type, false, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn query_mut_bundle_arg(&mut self, arg_name: impl Into<String>, bundle: Vec<&String>, filters: Filters) -> &mut Self {
        if bundle.len() == 1 {
            return self.query_mut_arg(arg_name, *bundle.get(0).unwrap(), &filters);
        }
        let bundle = bundle.iter().map(|s| format!("&mut {s}")).join(", ");
        let arg_type = if filters.is_empty() {
            format!("Query<({bundle})>")
        }
        else {
            format!("Query<({bundle}), {}>", filters.to_filter_bundle())
        };

        let argument = Argument::new(arg_name, arg_type, true, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn query_mut_arg(&mut self, arg_name: impl Into<String>, arg_type: impl Into<String>, filters: &Filters) -> &mut Self {
        let arg_type = if filters.is_empty() {
            format!("Query<&mut {0}>", arg_type.into())
        }
        else {
            format!("Query<&mut {0}, {1}>", arg_type.into(), filters.to_filter_bundle())
        };

        let argument = Argument::new(arg_name, arg_type, true, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn asset_server_arg(&mut self) -> &mut Self {
        let argument = Argument::new("server", "Res<AssetServer>", false, Ownership::Move);
        self.args.insert(0, argument);
        self
    }

    pub fn commands_arg(&mut self) -> &mut Self {
        let argument = Argument::new("commands", "Commands", true, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn single_arg(&mut self, arg_name: impl Into<String>, arg_type: &str, filters: &Filters) -> &mut Self {
        let arg_type = if filters.is_empty() {
            format!("Single<{arg_type}>")
        }
        else {
            let bundle = filters.to_filter_bundle();
            format!("Single<{arg_type}, {bundle}>")
        };
        let argument = Argument::new(arg_name, arg_type, false, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn single_ref_arg(&mut self, arg_name: impl Into<String>, arg_type: &str, filters: &Filters) -> &mut Self {
        let arg_type = if filters.is_empty() {
            format!("Single<&{arg_type}>")
        }
        else {
            let bundle = filters.to_filter_bundle();
            format!("Single<&{arg_type}, {bundle}>")
        };
        let argument = Argument::new(arg_name, arg_type, false, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn single_mut_arg(&mut self, arg_name: impl Into<String>, arg_type: &str, filters: &Filters) -> &mut Self {
        let arg_type = if filters.is_empty() {
            format!("Single<&mut {arg_type}>")
        }
        else {
            let bundle = filters.to_filter_bundle();
            format!("Single<&mut {arg_type}, {bundle}>")
        };
        let argument = Argument::new(arg_name, arg_type, true, Ownership::Move);
        self.args.push(argument);
        self
    }

    pub fn resource_arg(&mut self, res_type: impl Into<String>, arg_name: impl Into<String>) -> &mut Self {
        let res_type = res_type.into();
        let arg_type = format!("Res<{res_type}>");
        let argument = Argument {
            name: arg_name.into(),
            arg_type,
            mutable: false,
            ownership: Ownership::Move,
        };

        self.args.push(argument);
        self
    }

    pub fn resource_mut(&mut self, res_name: impl Into<String>, arg_name: impl Into<String>) -> &mut Self {
        let arg_type = format!("ResMut<{}>", res_name.into());
        let argument = Argument {
            name: arg_name.into(),
            arg_type,
            mutable: true,
            ownership: Ownership::Move,
        };

        self.args.push(argument);
        self
    }

    pub fn result(&mut self, value: ReturnValue) -> &mut Self {
        self.result.push(value);
        self
    }

    pub const fn self_arg(&mut self, ownership: Option<Ownership>) -> &mut Self {
        self.self_arg = ownership;
        self
    }

    pub fn push_line_to_body(&mut self, value: impl Into<String>) -> &mut Self {
        self.body.push_str("    ");
        self.body.push_str(&value.into());
        self.body.push('\n');
        self
    }

    pub fn push_to_body(&mut self, value: impl Into<String>) -> &mut Self {
        self.body.push_str(&value.into());
        self
    }

    pub fn skip_line(&mut self) -> &mut Self {
        self.body.push('\n');
        self
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0} fn {1}", self.access, self.name)?;

        let self_arg = if let Some(self_arg) = self.self_arg {
            format!("{self_arg}self")
        } else {
            String::new()
        };

        let args = self.args.iter().map(Argument::to_string).collect::<Vec<_>>().join(", ");
        let args = if self_arg.is_empty() {
            args
        }
        else {
            format!("{self_arg}, {args}")
        };

        write!(f, "({args})")?;

        if !self.result.is_empty() {
            let result = self.result.iter().map(ReturnValue::to_string).collect::<Vec<_>>().join(", ");
            let returns = if self.result.len() > 1 {
                format!("({})", result)
            } else {
                format!("{}", result)
            };

            write!(f, " -> {returns}")?;
        }

        write!(f, " {{\n{}\n}}", self.body)
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::{Access, Function};
    use crate::codegen::file::RustFile;
    use crate::codegen::module::Module;
    use crate::codegen::structs::Struct;

    #[test]
    fn test() {
        let mut function = Function::new("some_fn");
        function.access(Access::Crate);
        let string = function.to_string();

        let mut struc = Struct::new("Printer");
        struc.access(Access::Public);
        struc.field(Access::None, "value_0", "bool");
        struc.field(Access::Public, "value_1", "i32");
        struc.field(Access::Super, "value_2", "char");
        struc.field(Access::Crate, "value_3", "String");
        struc.derives(&["Component", "Copy", "Clone"]);

        let mut module = Module::new("test");
        module.with_struct(struc);

        let mut file = RustFile::default();
        file.with_module(module);
        file.with_function(function);

        let result = file.into_string(true);

        println!();
    }
}