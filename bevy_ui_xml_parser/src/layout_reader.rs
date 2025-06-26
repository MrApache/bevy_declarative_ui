use xml_parser::{Token, XmlReader};
use crate::error::XmlLayoutError;
use crate::{UiNode, XmlLayout};
use crate::resources::{Resource};

pub struct LayoutReader<'a> {
    pub(super) inner: XmlReader<'a>,
}

impl<'a> LayoutReader<'a> {
    pub fn new(xml: &'a str, file: &'a str) -> Self {
        Self {
            inner: XmlReader::new(xml, file),
        }
    }

    pub fn parse_layout(&mut self) -> Result<XmlLayout, XmlLayoutError> {
        let mut layout: XmlLayout = Default::default();
        let mut has_layout_tag: bool = false;
        let mut has_global_resources: bool = false;
        let mut has_local_resources: bool = false;

        let mut using: bool = false;

        loop {
            match self.inner.read()? {
                Token::TagStart(tag) => {
                    match tag.identifier() {
                        "Layout" => {
                            if has_layout_tag {
                                return Err(self.err_multiple_layouts(&tag));
                            }
                            has_layout_tag = true;
                        }
                        "Use" => using = true,
                        "Container" => {
                            if !has_layout_tag {
                                return Err(self.err_missing_layout());
                            }

                            layout.root_nodes.push(self.parse_container(tag)?);
                        }
                        "GlobalResources" => {
                            if !has_layout_tag {
                                return Err(self.err_missing_layout());
                            }
                            has_global_resources = true;
                            self.parse_resources(&mut layout.global, Resource::Global)?
                        },
                        "LocalResources" => {
                            if !has_layout_tag {
                                return Err(self.err_missing_layout());
                            }
                            has_local_resources = true;
                            self.parse_resources(&mut layout.local, Resource::Local)?
                        },
                        "Template" => {
                            if !has_layout_tag {
                                return Err(self.err_missing_layout());
                            }
                            layout.templates.push(self.parse_template(tag)?)
                        },
                        unknown => panic!("Unknown tag: {}", unknown),
                    }
                }
                Token::TagEmpty(tag) => {
                    match tag.identifier() {
                        "Layout"          => return Err(XmlLayoutError::EmptyLayout),
                        "GlobalResources" => return Err(XmlLayoutError::EmptyGlobalResources),
                        "LocalResources"  => return Err(XmlLayoutError::EmptyLocalResources),
                        "Container" => panic!("container"),
                        "Template" => panic!("template"),
                        _ => layout.root_nodes.push(UiNode::component(tag)?),
                    }
                }
                Token::TagEnd(tag) => {
                    match tag.identifier() {
                        "Layout" => has_layout_tag = false,
                        "Use" => using = false,
                        _ => {}
                    }
                }
                Token::Text(text) => {
                    if !using {
                        continue;
                        //panic!("TODO");
                    }

                    layout.usings.insert(text);
                }
                Token::EOF => {
                    if !has_layout_tag {
                        break;
                    }

                    return Err(self.err_end_of_file());
                }
                _ => {}
            }
        }

        if layout.root_nodes.is_empty()
            && layout.templates.is_empty()
            && !has_local_resources
            && !has_global_resources {
            return Err(XmlLayoutError::EmptyLayout);
        }

        if has_global_resources && layout.global.is_empty() {
            return Err(XmlLayoutError::EmptyGlobalResources);
        }

        if has_local_resources && layout.local.is_empty() {
            return Err(XmlLayoutError::EmptyLocalResources);
        }

        Ok(layout)
    }
}