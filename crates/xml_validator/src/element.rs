use std::collections::HashMap;

use crate::r#type::TypeReference;


pub struct ElementMetadata {
    path: String
}

pub struct Element {
    metadata: ElementMetadata,
    min_occurs: u32,
    max_occurs: u32,
    attributes: HashMap<String, Attribute>,
}

pub struct AttributeMetadata {
    field: Option<String>
}

pub struct Attribute {
    reference: TypeReference,
    metadata: AttributeMetadata,
}

pub struct GroupDeclaration {
    compositor: Compositor,
    elements: Vec<Element>,
}

pub enum Group {
    Declaration(GroupDeclaration),
    Ref(String)
}

pub enum Compositor {
    Sequence,
    Choice,
    All
}

pub struct ComplexType {
    pub elements: Vec<Element>,            // дочерние элементы или вложенные группы
    pub attributes: Vec<Attribute>,        // атрибуты типа
    pub content_model: Option<Compositor>, // sequence, choice или all
    pub base: Option<TypeReference>,       // базовый тип (для расширения или ограничения)
}
