use std::collections::HashMap;
use itertools::Itertools;
use bevy_declarative_ui_parser::{ItemTemplate, UiNode};
use bevy_declarative_ui_parser::values::{AttributeValue, Binding, Filter, Filters};
use crate::codegen::{Argument, Function, Module, Ownership};
use crate::r#static::required::Required;
use crate::utils::{GetOrInsertEmpty, ToSnakeCase};

fn get_access_to_target(function: &mut Function, binding: &Binding) {
    match binding {
        Binding::Resource { params } => {
            function.resource_arg(&params.target, "target");
            function.push_line_to_body(format!("let target = target.{};", params.path));
        }
        Binding::Component { params } => {
            function.single_ref_arg("target", &params.base.target, &params.filters);
            function.push_line_to_body(format!("let target = target.{};", params.base.path));
        }
    }
}

pub fn print_template_functions(templates: &[ItemTemplate], global_required: &mut Required) -> Module {
    let mut required = Required::default();

    let mut module = Module::new("templates");

    templates.iter().for_each(|template| {
        required.ids.push(template.id.to_string());
        template_workload(template, &mut module);

        let mut function = Function::new(format!("spawn_{}", template.id.to_string().to_snake_case()));
        get_access_to_target(&mut function, &template.source);
        function.commands_arg();
        function.single_arg("container", "Entity", &Filters::single(Filter::With(template.owner.to_string())));
        function.push_line_to_body("let mut container = commands.entity(container);");
        function.push_line_to_body("container.with_children(|p| {");

        template.nodes.iter().for_each(|node| print_node(&mut function, &mut required, node));

        function.push_line_to_body("});");
        if required.asset_server {
            function.asset_server_arg();
            required.asset_server = false;
        }

        module.with_function(function);
    });

    global_required.ids.extend(required.ids);

    module

}

fn print_node(function: &mut Function, required: &mut Required, node: &UiNode) {
    let mut fields: Vec<String> = node.components.iter()
        .map(|c| crate::r#static::spawn_function::format_component(required, &node.id, c))
        .collect();
    let runtime_id = format!("Runtime{}", node.id.to_string());
    fields.push(runtime_id.clone());
    required.ids.push(runtime_id);

    if fields.len() > 1 {
        function.push_line_to_body("p.spawn((");
        function.push_to_body(fields.join(", "));
        function.push_to_body("))");
    }
    else {
        function.push_line_to_body("p.spawn(");
        function.push_line_to_body(fields.get(0).unwrap());
        function.push_to_body(")");
    }

    if node.children.is_empty() {
        function.push_to_body(';');
    } else {
        function.push_line_to_body(".with_children(|p| {");
        node.children.iter().for_each(|container| print_node(function, required, container));
        function.push_line_to_body("});");
    }
}

fn template_workload(template: &ItemTemplate, module: &mut Module) {
    module.with_using("std::cmp::Ordering");
    module.with_using("bevy::prelude::*");
    module.with_using("super::ids::*");
    module.with_function(template_instance_limiter(template));
    module.with_function(template_binding(template));
}

fn template_instance_limiter(template: &ItemTemplate) -> Function {
    let mut function: Function = Function::new(format!("{}_instance_limiter", template.id.to_string().to_snake_case()));
    function.commands_arg();
    function.arg(Argument::new("instances", format!("Query<Entity, With<{}>>", template.id), false, Ownership::Move));
    function.local_mut_arg("previous", "usize");

    let content = format!(r#"
    let count = instances.iter().len();
    match count.cmp(&*previous) {{
        Ordering::Less => {{
            instances.iter().skip(count).for_each(|entity| {{
                commands.entity(entity).despawn();
            }});
            *previous = count;
        }},
        Ordering::Greater => {{
            let count = count.abs_diff(*previous);
            (0..count).for_each(|_| commands.run_system_cached(spawn_{0}));
            *previous = count;
        }},
        Ordering::Equal => return,
    }}
    "#, template.id.to_string().to_snake_case());
    function.push_line_to_body(content);

    function
}

fn template_binding(template: &ItemTemplate) -> Function {
    let mut function: Function = Function::new(format!("{}_binding", template.id.to_string().to_snake_case()));

    match &template.source {
        Binding::Resource { params } => {
            function.resource_arg(&params.target, "target");
            function.push_line_to_body("if !target.is_changed() {{\n return;\n}}");
            function.push_line_to_body(format!("let target = &target.{};", &params.path));
        },
        Binding::Component { params } => {
            let mut filters = params.filters.clone();
            let filters = filters.with(Filter::Changed(params.base.target.clone()));
            function.single_ref_arg("target", &params.base.target, &filters);
            function.push_line_to_body(format!("let target = &target.{};", &params.base.path));
        }
    }

    let mut observers = HashMap::new();
    create_observers(&mut 0, &mut observers, &template.nodes);
    let mut observers = observers.into_iter().collect_vec();
    observers.sort_by(|((_, ident_a), _), ((_, ident_b), _)| ident_a.cmp(ident_b));
    prepare_arguments(&mut function, &mut observers);

    if observers.len() > 1 {
        let mut cmp_binds = Vec::new();
        let mut zip_calls = String::new();
        let mut setters = String::new();

        observers.iter().skip(1).for_each(|((_, ident), _)| {
            zip_calls.push_str(&format!(".zip({ident}.iter_mut())\n"));
        });

        observers.into_iter().for_each(|((_, ident), components)| {
            cmp_binds.push(ident.clone());
            let components_in_bundle = components.len();

            components.into_iter().enumerate().for_each(|(i, (_, observers))| {

                observers.iter().for_each(|observer| {
                    if components_in_bundle > 1 {
                        setters.push_str(
                            &format!("{ident}.{i}.{field} = target.{path};\n",
                                     field = observer.field,
                                     path = observer.path
                            ));
                    }
                    else {
                        setters.push_str(
                            &format!("{ident}.{field} = target.{path};\n",
                                     field = observer.field,
                                     path = observer.path
                            ));
                    }
                });
            });
        });

        let cmp_binds = nested_join(cmp_binds);
        function.push_line_to_body(format!(r#"
        cmp0.iter()
            {zip_calls}
            .enumerate()
            .for_each(|(i, {cmp_binds})| {{
                let target = target.get(i).unwrap();
                {setters}
            }});
        "#));
    }
    function
}

fn create_observers<'a>(
    i: &mut u32,
    observers: &mut HashMap<(String, String), HashMap<String, Vec<Observer<'a>>>>, //(Owner, Ident), Component, Observer
    nodes: &'a [UiNode],
) {
    for node in nodes {
        let mut components = HashMap::new();
        for component in &node.components {
            for attribute in &component.attributes {
                if let AttributeValue::Item(item) = &attribute.value {
                    let observers: &mut Vec<Observer> = components.get_or_insert_empty(component.name.clone());
                    observers.push(Observer {
                        field: &attribute.name,
                        path: &item.path,
                    });
                }
            }
        }
        observers.insert((format!("Runtime{}", node.id.to_string()), format!("cmp{i}")), components);
        *i += 1;
        create_observers(i, observers, &node.children);
    }
}

struct Observer<'a> {
    field: &'a str,
    path:  &'a str
}

fn nested_join(items: Vec<String>) -> String {
    match items.len() {
        0 => unreachable!(),
        1 => items[0].clone(),
        2 => format!("({}, {})", items[0], items[1]),
        _ => {
            let mut iter = items.into_iter();
            let mut acc = format!("({}, {})", iter.next().unwrap(), iter.next().unwrap());
            iter.for_each(|item| acc = format!("({acc}, {item})"));
            acc
        }
    }
}

fn prepare_arguments(
    function: &mut Function,
    observers: &Vec<((String, String), HashMap<String, Vec<Observer>>)>
) {
    observers.iter().for_each(|((owner, ident), components)| {
        let bundle = components.keys().collect_vec();

        let mut filters = Filters::single(Filter::With(owner.to_string()));
        for ((other_owner, _), other_components) in observers.iter() {
            if other_owner == owner {
                continue;
            }

            for component in &bundle {
                if other_components.contains_key(*component) {
                    filters.with(Filter::Without(other_owner.clone()));
                }
            }
        }

        function.query_mut_bundle_arg(ident, bundle, filters);
    });
}