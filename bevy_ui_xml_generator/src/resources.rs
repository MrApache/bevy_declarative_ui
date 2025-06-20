use bevy_ui_xml_parser::RawResources;

use crate::module::Property;

pub(super) fn get_resources(resources: &RawResources, output: &mut String) -> Vec<Property> {
    let mut properties: Vec<Property> = Vec::new();
    write_resources(resources, output, &mut properties);
    properties
}

pub(super) fn write_resources(resources: &RawResources, output: &mut String, properties: &mut Vec<Property>) {
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