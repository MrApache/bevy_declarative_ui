use bevy_declarative_ui_parser::values::{Filter, Filters};
use crate::codegen::{Access, Argument, Function, Ownership, Struct};
use crate::r#static::binding::{prepare_filters, Observer};
use crate::utils::ToSnakeCase;

pub(super) fn binding_component_read_printer(name: &str, observers: Vec<Observer>, mut filters: Filters) -> Function {
    let mut function = Function::new(format!("component_{}_binding_read", name.to_snake_case()));
    let filters = filters.with(Filter::Changed(name.to_string()));
    function.single_ref_arg("target", &name, &filters);

    let mut index = 0;
    for observer in &observers {
        let target = observer.target;
        let set = &observer.set;
        let get = &observer.get;
        let filters = prepare_filters(observer, &observers);
        function.single_mut_arg(format!("observer_{index}"), target, &filters);
        function.push_line_to_body(&format!("observer_{index}.{set} = target.{get};"));
        index += 1;
    }

    function
}

pub(super) fn binding_component_write_printer(name: &str, targets: Vec<Observer>, filters: Filters) -> Function {
    let mut function = Function::new(format!("component_{}_binding_write", name.to_snake_case()));
    function.single_mut_arg("observer", &name, &filters);

    let mut index = 0;
    for target in &targets {
        let component = target.target;
        let set = &target.set;
        let get = &target.get;
        let mut filters = prepare_filters(target, &targets);
        filters.with(Filter::Changed(component.to_string()));
        function.single_ref_arg(format!("target_{index}"), component, &filters);
        function.push_line_to_body(&format!("observer.{set} = target_{index}.{get};"));
        index += 1;
    }

    function
}

pub(super) fn binding_component_read_write_printer(name: &str, targets: Vec<Observer>, filters: Filters) -> (Struct, Function) {
    let mut r#struct: Struct = Struct::new(format!("Component{name}BindingReadWrite"));
    r#struct.derive("Default");
    r#struct.field(Access::None, "cmp0", "bool");

    let mut function = Function::new(format!("component_{}_binding_read_write", name.to_snake_case()));
    function.arg(Argument::new("local", format!("Local<Component{name}BindingReadWrite>"), true, Ownership::Move));
    function.single_mut_arg("cmp0", &name, &filters);

    let mut index = 1;
    function.push_line_to_body("if cmp0.is_changed() && !local.cmp0 {");
    for target in &targets {
        let set = &target.set;
        let get = &target.get;
        function.push_line_to_body(&format!("cmp{index}.{set} = cmp0.{get};"));
        index += 1;
    }
    function.push_line_to_body("local.cmp0 = true;");
    function.push_line_to_body("}\n");

    index = 1;
    for target in &targets {
        let set = &target.set;
        let get = &target.get;
        let filters = prepare_filters(target, &targets);
        let identifier = format!("cmp{index}");
        r#struct.field(Access::None, &identifier, "bool");
        function.single_mut_arg(&identifier, target.target, &filters);
        function.push_line_to_body(&format!("if {identifier}.is_changed() && !local.{identifier} {{"));
        function.push_line_to_body(&format!("cmp0.{get} = {identifier}.{set};"));
        function.push_line_to_body(format!("local.{identifier} = true;"));
        function.push_line_to_body("}");
        function.push_line_to_body(format!("else if local.{identifier} {{"));
        function.push_line_to_body(format!("local.{identifier} = false;"));
        function.push_line_to_body("}\n");
        index += 1;
    }

    (r#struct, function)
}
