use std::collections::HashSet;
use std::io::{Read, Seek, Write};
use std::process::Command;
use itertools::Itertools;
use tempfile::NamedTempFile;
use crate::codegen::Function;
use crate::codegen::module::Module;
use crate::codegen::structs::Struct;
use crate::codegen::using::Using;

#[derive(Default)]
pub struct RustFile {
    modules:   Vec<Module>,
    structs:   Vec<Struct>,
    functions: Vec<Function>,
    usings:    HashSet<Using>
}

impl RustFile {
    pub fn into_string(self, format: bool) -> String {
        let mut result = String::with_capacity(2048);

        let usings = &self.usings.iter().join("\n");
        let modules = &self.modules.iter().join("\n\n");
        let structs = &self.structs.iter().join("\n\n");
        let functions = &self.functions.iter().join("\n\n");

        if !self.usings.is_empty() {
            result.push_str(&usings);
            if !self.modules.is_empty() || !self.structs.is_empty() || !self.functions.is_empty() {
                result.push_str("\n\n");
            }
        }

        if !self.modules.is_empty() {
            result.push_str(&modules);
            if !self.structs.is_empty() || !self.functions.is_empty() {
                result.push_str("\n\n");
            }
        }

        if !self.structs.is_empty() {
            result.push_str(&structs);
            if !self.functions.is_empty() {
                result.push_str("\n\n");
            }
        }

        if !self.functions.is_empty() {
            result.push_str(&functions);
        }

        if format {
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file.write_all(result.as_bytes()).unwrap();
            temp_file.flush().unwrap();
            temp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
            let _ = Command::new("rustfmt").arg(temp_file.path()).status();
            result.clear();
            temp_file.read_to_string(&mut result).unwrap();
        }

        result
    }

    pub fn with_using(&mut self, path: impl Into<String>) -> &mut Self {
        self.usings.insert(Using::new(path));
        self
    }

    pub fn with_module(&mut self, module: Module) -> &mut Self {
        self.modules.push(module);
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
}