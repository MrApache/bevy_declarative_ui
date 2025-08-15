use crate::codegen::{Access, Module, Struct};

pub(super) fn generate_ids(ids: &Vec<String>) -> Module {
    let mut module = Module::new("ids");
    module.with_using("bevy::prelude::*");
    module.access(Access::Public);
    ids.iter().for_each(|id| {
        let mut r#struct = Struct::new(id);
        r#struct.access(Access::Public);
        r#struct.derive("Component");
        module.with_struct(r#struct);
    });
    module
}