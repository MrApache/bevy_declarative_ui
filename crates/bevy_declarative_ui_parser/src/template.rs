use crate::{
    Id,
    UiNode,
    XmlTag,
    errors::XmlLayoutError,
    layout_reader::LayoutReader,
    values::AttributeValue,
};
use crate::values::TemplateBinding;

#[derive(Debug)]
pub struct ItemTemplate {
    ///Always equals Id::Template
    pub id:     Id,
    pub owner:  Id,
    pub source: TemplateBinding,
    pub nodes:  Vec<UiNode>,
}

impl ItemTemplate {
    pub fn new(reader: &LayoutReader, tag: &XmlTag, owner: Id) -> Result<Self, XmlLayoutError> {
        let source = reader.parse_required_attribute(&tag, "source")?;
        if source.value().is_empty() {
            return Err(reader.err_empty_attribute(&tag, "source"));
        }

        if let AttributeValue::Template(source) = AttributeValue::parse(reader, &source, true)? {
            Ok(ItemTemplate {
                id: Id::Template(0),
                owner,
                source,
                nodes: vec![],
            })
        }
        else {
            panic!("Invalid binding input: {:?}", source);
        }
    }
}