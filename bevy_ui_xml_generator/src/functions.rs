use core::panic;
use std::{collections::HashMap, str::FromStr};
use bevy_ui_xml_parser::{NodeValue, Tag, UiNode};

#[derive(Default)]
pub(super) struct Functions {
    pub output: String,
    pub names:  Vec<String>
}

fn detect_type(value: &str) -> String {
    if let Some((enum_type, _variant)) = value.split_once("::") {
        return format!("{}", enum_type);
    }

    if let Ok(_) = i8::from_str(value) {
        return "i8".to_string();
    }

    if let Ok(_) = i16::from_str(value) {
        return "i16".to_string();
    }

    if let Ok(_) = i32::from_str(value) {
        return "i32".to_string();
    }

    if let Ok(_) = i64::from_str(value) {
        return "i64".to_string();
    }

    if let Ok(_) = u8::from_str(value) {
        return "u8".to_string();
    }

    if let Ok(_) = u16::from_str(value) {
        return "u16".to_string();
    }

    if let Ok(_) = u32::from_str(value) {
        return "u32".to_string();
    }

    if let Ok(_) = u64::from_str(value) {
        return "u64".to_string();
    }

    if let Ok(_) = value.parse::<f32>() {
        return "f32".to_string();
    }

    if let Ok(_) = value.parse::<f64>() {
        return "f64".to_string();
    }

    if let "true" | "false" = value {
        return "bool".to_string();
    }

    if value.starts_with("\"") && value.ends_with("\"") {
        return "&str".to_string();
    }

    if value.len() == 3 && value.starts_with('\'') && value.ends_with('\'') {
        return "char".to_string();
    }

    panic!("Unknown type: {value}");
}


fn generate_pattern_matching(args: Vec<String>) -> String {
    let mut output: String = String::default();
    output.push_str("let args = match args.get(context.owner_entity()).unwrap().arguments(context.caller()) {");

    args.into_iter().for_each(|arg| {
        let key: String = arg.replacen("context, ", "", 1);
        output.push_str(&format!("\"{key}\" => ({arg}),"));
    });
    output.push_str("other => panic!(\"Incorrect arguments: {other}\"),");
    output.push_str("};");

    output
}

fn generate_function(name: &str, args: &Vec<String>) -> Function {
    let mut types: String = String::new();
    let mut values: String = String::new();
    values.push_str("context");
    types.push_str("CallbackContext");
    args.iter().for_each(|arg| {
        let t = detect_type(arg.as_str());
        types.push_str(&format!(", {t}"));
        values.push_str(&format!(", {arg}"));
    });

    let output: String = format!(r#"
    pub(super) fn {name}(
        In(context):  In<CallbackContext>,
        functions:    Res<UiFunctions>,
        args:         Query<&CallbacksArguments>,
        mut commands: Commands) {{

        if let Some(handler) = functions.get_event_handler("{name}") {{
            let id = handler.as_any().downcast_ref::<SystemId<In<({types})>>>().unwrap();
            ___ARGS___
            commands.run_system_with(*id, args);
        }} else {{
            error!("[Ui Functions] Function `{name}` is not bound");
            return;
        }}
    }}
    "#);

    Function {
        name: name.to_string(),
        body: output,
        args: vec![values],
    }
}

struct Function {
    name: String,
    body: String,
    args: Vec<String>,
}

pub(super) fn generate_functions(nodes: &Vec<UiNode>) -> Functions {
    let mut prepared_functions: Functions = Functions::default();

    let mut functions = HashMap::<String, Function>::new();
    nodes.iter().for_each(|node| {
        if node.tag != Tag::Container {
            return;
        }
        let functions_tmp = generate_functions(&node.children);
        prepared_functions.names.extend(functions_tmp.names);
        prepared_functions.output.push_str(&functions_tmp.output);

        node.attributes.iter().for_each(|attribute| {
            let func: Function = match &attribute.value {
                NodeValue::CallFunction { name, args } => generate_function(name, args),
                //NodeValue::CallPropertyFunction { name, args } => generate_bindable_function(name, args),
                _ => return,
            };

            if !functions.contains_key(&func.name) {
                functions.insert(func.name.clone(), func);
            } else {
                functions.get_mut(&func.name).unwrap().args.extend(func.args);
            }
        });
    });

    functions.into_iter().for_each(|(name, function)| {
        let args: String = generate_pattern_matching(function.args);
        let body: String = function.body.replace("___ARGS___", &args);
        prepared_functions.names.push(name);
        prepared_functions.output.push_str(&body);
    });

    prepared_functions
}