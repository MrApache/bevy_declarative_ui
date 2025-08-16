use std::sync::atomic::AtomicU64;
use bevy_declarative_ui_parser::{Id, UiNode};
use bevy_declarative_ui_parser::values::AttributeValue;
use bevy_declarative_ui_parser::into::Tag;
use crate::codegen::{Access, Function};
use crate::r#static::required::{Required, RequiredBinding};

pub fn print_spawn_function(required: &mut Required, nodes: &[UiNode]) -> Function {
    let mut function = Function::new("spawn_document");
    function
        .access(Access::Public)
        .commands_arg()
        .push_line_to_body("let mut root = commands.spawn_empty();")
        .push_line_to_body("root.with_children(|p| {");

    nodes.iter().for_each(|node| print_node(&mut function, required, node));

    if required.asset_server {
        function.asset_server_arg();
    }

    function.push_line_to_body("});");
    function
}

fn print_node(function: &mut Function, required: &mut Required, node: &UiNode) {
    let mut fields: Vec<String> = node.components.iter()
        .map(|c| format_component(required, &node.id, c))
        .collect();
    fields.push(format!("{}", node.id));
    required.ids.push(node.id.to_string());

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
        node.children.iter().for_each(|child| print_node(function, required, child));
        function.push_line_to_body("});");
    }
}

static RES_ID: AtomicU64 = AtomicU64::new(0);
static COMP_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) fn format_component(required: &mut Required, id: &Id, tag: &Tag) -> String {
    if tag.attributes.is_empty() {
        return format!("{}::default()", tag.name);
    }

    let mut fields = String::new();

    tag.attributes.iter().for_each(|attr| {
        let value: Option<String> = match &attr.value {
            AttributeValue::Value(value) => Some(value.to_string()),
            AttributeValue::Item(item) => Some(format!("target.{}", item.path)),
            AttributeValue::Asset(asset) => {
                required.asset_server = true;
                Some(format!("server.load(\"{}\")", asset.path))
            },
            AttributeValue::Binding(binding) => prepare_binding(required, id, &tag.name, &attr.name, binding),
            AttributeValue::Function { .. } => todo!(),
        };

        if let Some(value) = value {
            fields.push_str(&format!("    {}: {},\n", attr.name, value));
        }
    });

    if fields.is_empty() {
        format!("{}::default()", tag.name)
    }
    else {
        let mut result = format!("{} {{\n", tag.name);
        result.push_str(&fields);
        result.push_str("    ..default()\n");
        result.push('}');
        result
    }
}

fn create_required_binding(required: &mut Required, id: String, binding: Binding, component: String, field_name: String){
    let bindings = required.bindings.get_or_insert_empty(id);
    bindings.push(RequiredBinding {
        inner: binding,
        component,
        field_name,
    });
}

fn prepare_binding(required: &mut Required, id: &Id, component: &String, attribute: &String, binding: &Binding) -> Option<String> {
    match binding {
        Binding::Resource { params } => {
            if BindingMode::ReadOnce == params.mode {
                if !required.resources.contains_key(&params.target) {
                    let arg_id = RES_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let arg_id = format!("res{arg_id}");
                    required.resources.insert(params.target.clone(), arg_id);
                }
                let id = &required.resources.get(&params.target).unwrap();
                return Some(format!("{id}.{}", params.path.to_string()));
            }
        }
        Binding::Component { params } => {
            if BindingMode::ReadOnce == params.base.mode {
                if !required.components.contains_key(&params.base.target) {
                    let arg_id = COMP_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let arg_id = format!("cmp{arg_id}");
                    required.components.insert(params.base.target.clone(), (arg_id, params.filters.clone()));
                }
                let (id, _) = &required.components.get(&params.base.target).unwrap();
                return Some(format!("{id}.{}", params.base.path.clone()));
            }
        }
    }
    create_required_binding(required, id.to_string(), binding.clone(), component.clone(), attribute.clone());
    None
}