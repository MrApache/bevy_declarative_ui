use std::collections::HashMap;
use syn::ItemStruct;
use serde::Serialize;
use syn::visit::Visit;

#[derive(Debug, Serialize)]
struct FieldInfo {
    name: String,
    type_name: String,
    attributes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct StructAnalysis {
    file: String,
    module_path: Vec<String>,
    fields: Vec<FieldInfo>,
}

struct XmlComponentVisitor {
    file: String,
    current_module: Vec<String>,
    components: HashMap<String, StructAnalysis>,
}

impl<'ast> Visit<'ast> for XmlComponentVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.current_module.push(node.ident.to_string());
        syn::visit::visit_item_mod(self, node);
        self.current_module.pop();
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        println!("Visit struct");
        let is_xml_component = node.attrs.iter().any(|attr| {
            attr.path().is_ident("derive") &&
                attr.parse_args::<syn::Path>()
                    .map_or(false, |p| p.is_ident("XmlComponent"))
        });

        if is_xml_component {
            let fields = match &node.fields {
                syn::Fields::Named(fields) => fields.named.iter()
                    .map(|f| {
                        FieldInfo {
                            name: f.ident.as_ref().unwrap().to_string(),
                            type_name: type_to_string(&f.ty),
                            attributes: f.attrs.iter()
                                .map(|a| a.path().get_ident().unwrap().to_string())
                                .collect(),
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            };

            self.components.insert(
                node.ident.to_string(),
                StructAnalysis {
                    file: self.file.clone(),
                    module_path: self.current_module.clone(),
                    fields,
                },
            );
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        println!("Visit impl");
        if let Some((_, path, _)) = &node.trait_ {
            if path.is_ident("XmlComponent") {
                if let syn::Type::Path(type_path) = &*node.self_ty {
                    let struct_name = type_path.path.segments.last().unwrap().ident.to_string();

                    // Получаем или создаем запись о структуре
                    let entry = self.components.entry(struct_name)
                        .or_insert(StructAnalysis {
                            file: self.file.clone(),
                            module_path: self.current_module.clone(),
                            fields: Vec::new(),
                        });

                    // Если поля еще не заполнены, попробуем найти структуру
                    if entry.fields.is_empty() {
                        if let Some(item) = find_struct_in_ast(&node.items, &type_path.path) {
                            entry.fields = extract_fields(&item.fields);
                        }
                    }
                }
            }
        }
    }
}

fn extract_fields(fields: &syn::Fields) -> Vec<FieldInfo> {
    match fields {
        syn::Fields::Named(fields) => fields.named.iter()
            .map(|f| FieldInfo {
                name: f.ident.as_ref().unwrap().to_string(),
                type_name: type_to_string(&f.ty),
                attributes: f.attrs.iter()
                    .map(|a| a.path().get_ident().unwrap().to_string())
                    .collect(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn find_struct_in_ast<'a>(items: &'a [syn::ImplItem], path: &syn::Path) -> Option<&'a ItemStruct> {
    for item in items {
        if let syn::ImplItem::Type(ty) = item {
            if let syn::Type::Path(ty_path) = &ty.ty {
                if ty_path.path == *path {
                    // Здесь нужно больше контекста для реальной реализации
                    // В упрощенном варианте возвращаем None
                    return None;
                }
            }
        }
    }
    None
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            type_path.path.segments.iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        }
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use syn::visit::Visit;
    use walkdir::WalkDir;
    use crate::r#static::type_analyzer::XmlComponentVisitor;

    #[test]
    fn it_works() {
        let mut visitor = XmlComponentVisitor {
            file: String::new(),
            current_module: Vec::new(),
            components: HashMap::new(),
        };

        // Рекурсивный обход всех Rust-файлов в проекте
        for entry in WalkDir::new("/home/irisu/bevy_declarative_ui/lexer/src").into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(ast) = syn::parse_file(&content) {
                        visitor.file = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                        visitor.visit_file(&ast);
                    }
                }
            }
        }

        println!();
        return;
        // Сохраняем результат в JSON
        let output = std::env::var("OUT_DIR").unwrap();
        let output_path = Path::new(&output).join("xml_components.json");
        std::fs::write(
            output_path,
            serde_json::to_string_pretty(&visitor.components).unwrap(),
        ).unwrap();
    }
}