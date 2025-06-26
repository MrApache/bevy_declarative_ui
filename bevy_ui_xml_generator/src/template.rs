use bevy_ui_xml_parser::{Resources, Template, XmlLayout};

use crate::{
    generate_function_registrations,
    generate_property_registrations,
    resources::get_resources,
    module::{
        Module
    },
    utils::{
        join_usings,
        to_pascal_case,
        to_snake_case,
    }
};

pub(super) fn generate_templates(
    layout:         &XmlLayout,
    root_module:    &Module,
    properties_reg: &mut String,
    functions_reg:  &mut String,
) -> String {
    let mut output: String = String::with_capacity(1024);
    let mut module: String = String::with_capacity(256);
    layout.templates.iter().for_each(|template| {
        module.clear();
        module.push_str(&format!("pub mod {} {{", template.name));
        module.push_str("use bevy_ui_xml::prelude::*;");
        module.push_str(&join_usings(&layout.usings));

        let properties = get_resources(&template.resources, &mut module);

        let tmp_module:Module = Module {
            name: template.name.clone(),
            path: root_module.path.clone(),
        };

        let property_registrations = generate_property_registrations(&properties, &template.name, &format!("{}::{}", root_module.path, &template.name));
        let function_registrations = generate_function_registrations(&template.nodes, &mut output, &tmp_module.name);
        properties_reg.push_str(&property_registrations);
        functions_reg.push_str(&function_registrations);

        let struct_name = to_pascal_case(&template.name);
        if template.resources.is_empty() {
            module.push_str(&zero_properties_struct(&struct_name));
        }
        else {
            module.push_str(&struct_with_properties(&struct_name, &template));
        }

        let str_impl = match template.containers.len() {
            0 => panic!(),
            1 => single_container_impl(
                &struct_name,
                &template.name,
                template.containers.iter().next().unwrap(),
                &template.resources
            ),
            _ => String::new()
        };

        module.push_str(&str_impl);
        module.push('}');
        output.push_str(&module);
    });
    output
}

fn zero_properties_struct(struct_name: &str) -> String {
    format!("#[derive(Default)]\npub struct {struct_name};")
}

fn struct_with_properties(struct_name: &str, template: &Template) -> String {
    let mut fields: String = String::with_capacity(128);
    template.resources.iter().for_each(|(name, property)| {
        let name = to_snake_case(name);
        let type_ = &property.type_;
        fields.push_str(&format!("pub {name}: {type_},\n"));
    });

    format!(r#"
    #[derive(Default)]
    pub struct {struct_name} {{
        {fields}
    }}
    "#)
}

fn single_container_impl(struct_name: &str, template_name: &str, container: &str, resources: &Resources) -> String {
    let mut adds: String = String::with_capacity(128);
    resources.iter().for_each(|(name, _)| {
        let snake_case_name = to_snake_case(name);
        adds.push_str(&format!("resources.add_property::<{name}>(self.{snake_case_name});"));
    });
    format!(r#"
    impl IntoTemplate for {struct_name} {{
        fn into_template(self) -> TemplateRequest {{
            let mut resources: UiResources = UiResources::default();
            {adds}
            TemplateRequest::new("{template_name}", "{container}", resources)
        }}
    }}
    "#)
}