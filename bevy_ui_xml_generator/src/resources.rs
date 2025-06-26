use bevy_ui_xml_parser::Resources;
use crate::module::Property;

pub(super) fn get_resources(resources: &Resources, output: &mut String) -> Vec<Property> {
    let mut properties: Vec<Property> = Vec::new();
    write_resources(resources, output, &mut properties);
    properties
}

pub(super) fn write_resources(resources: &Resources, output: &mut String, properties: &mut Vec<Property>) {
    resources.iter().for_each(|(name,property_value)| {
        let type_name = property_value.type_.as_str();
        output.push_str(&format!(r#"
            ///Type: {type_name}
            #[derive(Default)]
            pub struct {name};
            impl PropertyType for {name} {{
                type Type = {type_name};
            }}
            "#
        ));

        properties.push(Property {
            name: name.clone(),
            type_: type_name.to_string(),
        });
    });
}

pub(super) fn generate_property_registrations(properties: &Vec<Property>, module_name: &str, module_path: &str) -> String {
    let mut output: String = String::new();
    for property in properties {
        let name = &property.name;
        let type_ = &property.type_;
        let property_path = if module_name.is_empty() {
            name.clone()
        }
        else {
            format!("{}::{}", module_name, name)
        };

        let closure: String = format!("r\"{module_path}\", \"{name}\", || Box::<TypedStorage<{type_}>>::new(TypedStorage::default())");
        output.push_str(&format!("library.add_property::<{property_path}>({closure});\n"));
    }
    output
}