use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use bevy_ui_xml_parser::parse_layout;

use crate::resources::write_resources;
use crate::template::generate_templates;
use crate::functions::generate_functions;
use crate::module::{GeneratedModule, Module};
use crate::utils::join_usings;

mod resources;
mod template;
mod functions;
mod utils;
mod module;

fn collect_xml_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                collect_xml_files(&path, files);
            } else if path.extension().map_or(false, |ext| ext == "xml") {
                files.push(path);
            }
        }
    }
}

fn normalize_path(path: &str) -> String {
    let without_xml = path.strip_suffix(".xml").unwrap_or(path);
    without_xml.replace(['/', '\\'], "::")
}

fn generate_plugin(usings: &str, output_dir: &str, generated: HashMap<Module, GeneratedModule>) {
    let mut modules = String::new();
    let mut property_registrations: String = String::new();
    let mut function_registrations: String = String::new();

    generated.iter().for_each(|(module_def, module_decl)| {
        if module_def.name.contains("::") {
            modules.push_str(&format!("use self::{};\n", module_def.name));
        }
        else {
            modules.push_str(&format!("pub mod {};\n", module_def.name));
        }
        for property in &module_decl.properties {
            let name = &property.name;
            let type_ = &property.type_;
            let property_path: String = format!("{}::{}", module_def.name, name);
            let path = &module_def.path;
            let closure: String = format!("r\"{path}\", \"{name}\" , || Box::<TypedStorage<{type_}>>::new(TypedStorage::default())");
            property_registrations.push_str(&format!("library.add_property::<{property_path}>({closure});\n"));
        }

        for function in &module_decl.functions {
            let path: String = format!("{}::{}", module_def.name, function);
            function_registrations.push_str(&format!("functions.register(\"{function}\", {path});"));
        }
    });

    let content = format!(r#"
    {usings}
    {modules}
    pub struct XmlGeneratedPlugin;
    impl bevy::prelude::Plugin for XmlGeneratedPlugin {{
        fn build(&self, app: &mut bevy::prelude::App) {{
            use bevy_ui_xml::prelude::*;
            use bevy::prelude::Mut;
            use bevy::ecs::system::SystemState;

            let mut world = app.world_mut();
            let mut library: Mut<XmlLibrary> = world.resource_mut::<XmlLibrary>();
            {property_registrations}

            drop(library);

            let mut state = SystemState::<UiFunctionRegistry>::new(world);
            let mut functions = state.get_mut(world);
            {function_registrations}

            state.apply(world);
        }}
    }}
    "#);

    let generated_file = Path::new(output_dir).join("bevy_ui_xml_generated/mod.rs");
    let mut file = File::create(&generated_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    let _ = Command::new("rustfmt").arg(&generated_file).status();

}

pub fn generate_modules(assets_dir: PathBuf, output_dir: &str) -> Vec<PathBuf> {
    let mut usings: HashSet<String> = HashSet::default();
    let mut generated: HashMap<Module, GeneratedModule> = HashMap::new();
    let mut xml_files: Vec<PathBuf> = Vec::new();
    let ap: &Path = Path::new(&assets_dir);
    collect_xml_files(&ap, &mut xml_files);

    let mut output = String::with_capacity(2048);
    for path in &xml_files {
        output.clear();

        let layout_path = normalize_path(&path.strip_prefix(ap).unwrap().to_str().unwrap());
        let content = fs::read_to_string(&path).unwrap();
        let filename = path.file_stem().unwrap().to_string_lossy() + "_xml";
        let module_dir = Path::new(output_dir)
            .join("bevy_ui_xml_generated")
            .join(filename.to_string());

        fs::create_dir_all(&module_dir).unwrap();
        let module = Module {
            name: filename.to_string(),
            path: layout_path.clone(),
        };
        generated.insert(module.clone(), GeneratedModule::default());

        let result = parse_layout(&content);

        if let Ok(layout) = result {
            usings.extend(layout.usings.iter().cloned());
            output.push_str("use bevy::ecs::system::*;");
            output.push_str("use bevy_ui_xml::prelude::*;");
            output.push_str(&join_usings(&layout.usings));

            let mut properties = vec![];
            write_resources(&layout.global, &mut output, &mut properties);
            write_resources(&layout.local, &mut output, &mut properties);
            
            if !layout.templates.is_empty() {
                output.push_str(&generate_templates(&layout, &filename, &layout_path, &mut generated));
            }

            let functions = generate_functions(&layout.root_nodes);
            output.push_str(&functions.output);
            generated.get_mut(&module).unwrap().functions.extend(functions.names);

            properties.into_iter().for_each(|property| {
                generated.get_mut(&module).unwrap().properties.push(property);
            });

            let generated_file = module_dir.join("mod.rs");
            let mut file = File::create(&generated_file).unwrap();
            file.write_all(output.as_bytes()).unwrap();

            let _ = Command::new("rustfmt").arg(&generated_file).status();
        }
    }

    generate_plugin(&join_usings(&usings), output_dir, generated);
    xml_files
}