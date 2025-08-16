use bevy_declarative_ui_parser::LayoutReader;
use crate::codegen::RustFile;
use crate::r#static::binding::binding_printer;
use crate::r#static::ids::generate_ids;
use crate::r#static::required::Required;
use crate::r#static::spawn_function::print_spawn_function;
use crate::r#static::template_function::print_template_functions;

mod spawn_function;
mod binding;
mod ids;
mod required;
mod type_analyzer;
mod template_function;

#[allow(dead_code)]
pub fn generate_file_content(path: &str, xml: &str) -> String {
    let mut reader = LayoutReader::new(&xml, path);
    let mut result = reader.parse().unwrap();

    let mut required = Required::default();
    let mut spawn_function = print_spawn_function(&mut required, &result.root_nodes);
    let templates = print_template_functions(&mut result.templates, &mut required);

    let ids_module = generate_ids(&required.ids);
    let bindings = binding_printer(&required.bindings);

    required.components.iter().for_each(|(component, (arg_name, filters))| {
        spawn_function.single_ref_arg(arg_name, component, filters);
    });

    required.resources.iter().for_each(|(resource_name, arg_name)| {
        spawn_function.resource_arg(resource_name, arg_name);
    });

    let mut file = RustFile::default();
    result.usings.iter().for_each(|using| { file.with_using(using); });
    file.with_using("bevy::prelude::*");
    file.with_using("ids::*");
    file.with_module(ids_module);
    file.with_module(bindings);
    file.with_module(templates);
    file.with_function(spawn_function);

    file.into_string(true)
}

#[cfg(test)]
mod tests {
    use crate::r#static::generate_file_content;

    #[test]
    fn test() {
        let path = "/home/irisu/bevy_declarative_ui/bevy_declarative_ui/assets/injection_count_10.xml";
        let xml = std::fs::read_to_string(path).unwrap();
        let _result = generate_file_content(path, &xml);

        println!("{_result}");
    }
}

//Make query
//Get access
//Put ref in local variable
//Read value