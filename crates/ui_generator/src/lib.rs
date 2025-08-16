use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use bevy_declarative_ui_parser::{values::AttributeValue, LayoutReader, UiNode, XmlLayout};

use crate::functions::{generate_function_registrations};
use crate::utils::join_usings;
use crate::module::Module;

mod functions;
mod utils;
mod module;
mod r#static;
mod codegen;

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

pub fn generate_modules(assets_dir: PathBuf, output_dir: &str) -> Vec<PathBuf> {
    let mut xml_files: Vec<PathBuf> = Vec::new();
    let ap: &Path = Path::new(&assets_dir);
    collect_xml_files(&ap, &mut xml_files);

    for path in &xml_files {
        let layout_path = normalize_path(&path.strip_prefix(ap).unwrap().to_str().unwrap());
        let filename = path.file_stem().unwrap().to_string_lossy() + "_xml";
        let module_dir = Path::new(output_dir)
            .join("bevy_ui_xml_generated")
            .join(filename.to_string());
        fs::create_dir_all(&module_dir).unwrap();
        let module = Module {
            name: filename.to_string(),
            path: layout_path.clone(),
        };

        let content = fs::read_to_string(&path).unwrap();
        let mut reader = LayoutReader::new(&content, path.to_str().unwrap());
        let result = reader.parse();
        if result.is_err() {
            panic!("{}", result.unwrap_err());
            //continue;
        }
        generate_module(result.unwrap(), &module, &module_dir);
    }

    xml_files
}

fn generate_module(layout: XmlLayout, module: &Module, module_dir: &PathBuf) {
    let mut output: String = String::new();
    output.push_str("use bevy::ecs::system::*;");
    output.push_str("use bevy_declarative_ui::prelude::*;");
    output.push_str("use bevy::prelude::{Mut, App, Plugin};");
    output.push_str(&join_usings(&layout.usings));

    let mut function_registrations = generate_function_registrations(&layout.root_nodes, &mut output, &String::new());
    let binding_registrations = generate_binding_registration(&layout.root_nodes);

    output.push_str(&format!(r#"
    pub struct XmlGeneratedPlugin;
    impl Plugin for XmlGeneratedPlugin {{
        fn build(&self, app: &mut App) {{

            let mut world = app.world_mut();

            let mut state = SystemState::<UiFunctionRegistry>::new(world);
            let mut functions = state.get_mut(world);
            {function_registrations}

            {binding_registrations}

            state.apply(world);
        }}
    }}
    "#));

    let generated_file = module_dir.join("mod.rs");
    let mut file = File::create(&generated_file).unwrap();
    file.write_all(output.as_bytes()).unwrap();
    let _ = Command::new("rustfmt").arg(&generated_file).status();
}

fn generate_binding_registration(nodes: &Vec<UiNode>) -> String {
    let mut output: String = String::new();
    nodes.iter().for_each(|node| {
        node.tag.attributes.iter().for_each(|attribute| {
            match &attribute.value {
                AttributeValue::Binding(value) => {
                    //output.push_str(&format!("functions.register(\"{value}\", {value});")); TODO fix
                }
                _ => {},
            }
        });
        output.push_str(&generate_binding_registration(&node.children));
    });

    output
}