use bevy_declarative_ui_parser::values::bindings::filter::Filter;
use crate::codegen::{Access, Function, Struct};
use crate::r#static::binding::{prepare_filters, Observer};
use crate::utils::ToSnakeCase;

pub(super) fn binding_resource_read_printer(name: &str, observers: &Vec<Observer>) -> Function {
    let mut function = Function::new(format!("resource_{}_binding_read", name.to_snake_case()));
    function.resource_arg(name, "target");
    function.push_line_to_body("if !target.is_changed() {\nreturn;\n}");

    let mut index = 0;
    for observer in observers {
        let filters = prepare_filters(observer, observers);
        let target = observer.target;
        let set = &observer.set;
        let get = &observer.get;

        function.single_mut_arg(format!("observer_{index}"), target, &filters);
        function.push_line_to_body(&format!("observer_{index}.{set} = target.{get};"));
        index += 1;
    }

    function
}

pub(super) fn binding_resource_write_printer(name: &str, components: &Vec<Observer>) -> Function {
    let mut function = Function::new(format!("resource_{}_binding_write", name.to_snake_case()));
    function.resource_mut(name, "observer");

    let mut index = 0;
    for component in components {
        let mut filters = prepare_filters(component, components);
        let target = component.target;
        filters.with(Filter::Changed(target.to_string()));
        let set = &component.set;
        let get = &component.get;

        function.single_ref_arg(format!("target_{index}"), target, &filters);
        function.push_line_to_body(&format!("observer.{set} = target.{get};"));
        index += 1;
    }

    function
}

pub(super) fn binding_resource_read_write_printer(name: &str, observers: &Vec<Observer>) -> (Struct, Function) {
    let mut r#struct: Struct = Struct::new(format!("Resource{name}BindingReadWrite"));
    r#struct.derive("Default");
    r#struct.field(Access::None, "res", "bool");

    let mut index = 0;
    let mut function = Function::new(format!("resource_{}_binding_write", name.to_snake_case()));
    function.resource_mut(name, "res");
    function.push_line_to_body("if res.is_changed() && !local.res {");
    for target in observers {
        let set = &target.set;
        let get = &target.get;
        function.push_line_to_body(&format!("cmp{index}.{set} = res.{get};"));
        index += 1;
    }
    function.push_line_to_body("local.res = true;");
    function.push_line_to_body("}\n");

    index = 0;
    for target in observers {
        let set = &target.set;
        let get = &target.get;
        let filters = prepare_filters(target, &observers);
        let identifier = format!("cmp{index}");
        r#struct.field(Access::None, &identifier, "bool");
        function.single_mut_arg(&identifier, target.target, &filters);
        function.push_line_to_body(&format!("if {identifier}.is_changed() && !local.{identifier} {{"));
        function.push_line_to_body(&format!("res.{get} = {identifier}.{set};"));
        function.push_line_to_body(format!("local.{identifier} = true;"));
        function.push_line_to_body("}");
        function.push_line_to_body(format!("else if local.{identifier} {{"));
        function.push_line_to_body(format!("local.{identifier} = false;"));
        function.push_line_to_body("}\n");
        index += 1;
    }

    (r#struct, function)
}