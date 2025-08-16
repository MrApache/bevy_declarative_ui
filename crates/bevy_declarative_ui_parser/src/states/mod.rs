mod content;
mod layout;
mod template;
mod using;

use crate::lexer::Token;
use crate::values::AttributeValue;
use crate::{Id, ItemTemplate, LayoutReader, UiNode, XmlLayout, XmlLayoutError, XmlTag, into::Tag};
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

struct Container {
    depth: usize,
    inner: UiNode,
}

#[derive(Default)]
pub(super) struct FSMContext {
    depth: usize,
    container_tmp: Vec<Container>,
    pub(crate) layout: XmlLayout,
    pub(crate) token: Token,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl FSMContext {
    fn create_root_container(&mut self) {
        self.container_tmp.push(Container {
            depth: 1,
            inner: UiNode {
                tag: Tag {
                    name: "Container".to_string(),
                    attributes: vec![],
                },
                id: Id::Custom("Root".into()),
                components: vec![],
                children: vec![],
            },
        })
    }

    fn create_container_node(
        &mut self,
        reader: &LayoutReader,
        tag: XmlTag,
    ) -> Result<UiNode, XmlLayoutError> {
        let mut node: UiNode = UiNode::new(reader, tag.clone())?;
        node.id = if let Some(attr) = node.tag.attributes.iter().find(|attr| attr.name == "id") {
            if let AttributeValue::Value(value) = &attr.value {
                Id::Custom(value.clone())
            } else {
                return Err(reader.err_expected_value(&tag));
            }
        } else {
            Id::Default(ID.fetch_add(1, Ordering::SeqCst))
        };

        Ok(node)
    }

    pub fn create_nested_container(
        &mut self,
        reader: &LayoutReader,
        tag: XmlTag,
    ) -> Result<State, XmlLayoutError> {
        let node = self.create_container_node(reader, tag)?;
        self.container_tmp.push(Container {
            depth: self.depth,
            inner: node,
        });
        self.depth += 1;
        Ok(State::Root)
    }

    pub fn create_component_node(
        &mut self,
        reader: &LayoutReader,
        tag: XmlTag,
    ) -> Result<State, XmlLayoutError> {
        self.container_tmp
            .last_mut()
            .unwrap()
            .inner
            .components
            .push(Tag::from(reader, tag.clone())?);
        Ok(State::Root)
    }

    pub fn push_nested_containers_in_parent(&mut self) {
        let mut temp: Vec<UiNode> = Vec::new();
        self.depth -= 1;

        while let Some(node) = self.container_tmp.last() {
            if node.depth == self.depth {
                if let Some(node) = self.container_tmp.pop() {
                    temp.push(node.inner);
                }
            } else {
                break;
            }
        }

        if self.container_tmp.is_empty() {
            temp.into_iter().rev().for_each(|node| {
                self.container_tmp.push(Container {
                    depth: 0,
                    inner: node,
                })
            });
            return;
        }

        self.container_tmp
            .last_mut()
            .unwrap()
            .inner
            .children
            .extend(temp);
    }

    pub fn create_template(
        &mut self,
        reader: &LayoutReader,
        tag: XmlTag,
    ) -> Result<State, XmlLayoutError> {
        let owner = self.container_tmp.last_mut().unwrap().inner.id.clone();
        self.layout
            .templates
            .push(ItemTemplate::new(reader, &tag, owner)?);
        Ok(State::ItemTemplate)
    }
}

#[derive(PartialEq)]
pub(super) enum State {
    Layout,
    Content,
    Use,

    ItemTemplate,

    Root,
    Break,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Layout => write!(f, "Layout"),
            State::Content => write!(f, "Content"),
            State::Use => write!(f, "Use"),
            State::ItemTemplate => write!(f, "ItemTemplate"),
            State::Root => write!(f, "Root"),
            State::Break => write!(f, "Break"),
        }
    }
}

impl State {
    pub fn execute(
        &self,
        context: &mut FSMContext,
        reader: &mut LayoutReader,
    ) -> Result<State, XmlLayoutError> {
        match self {
            State::Layout => layout::layout_execute(context, reader),
            State::Content => content::content_execute(context, reader),
            State::Use => using::use_execute(context),
            State::ItemTemplate => template::template_execute(context, reader),
            State::Root => root_execute(context, reader),
            State::Break => Ok(State::Break),
        }
    }
}

fn root_execute(context: &mut FSMContext, reader: &LayoutReader) -> Result<State, XmlLayoutError> {
    match &context.token {
        Token::TagStart(tag) => match tag.identifier() {
            "ItemTemplate" => context.create_template(reader, tag.clone()),
            "Container" => context.create_nested_container(reader, tag.clone()),
            _ => Err(reader.err_unexpected_tag(
                tag.clone(),
                vec!["ItemTemplate", "Container", "Any component"],
            )),
        },
        Token::TagEmpty(tag) => {
            context
                .container_tmp
                .last_mut()
                .unwrap()
                .inner
                .components
                .push(Tag::from(reader, tag.clone())?);
            Ok(State::Root)
        }
        Token::TagEnd(tag) => match tag.identifier() {
            "Container" => {
                context.push_nested_containers_in_parent();
                Ok(State::Root)
            }
            "Layout" => {
                let temp = std::mem::take(&mut context.container_tmp);
                context
                    .layout
                    .root_nodes
                    .extend(temp.into_iter().map(|c| c.inner));
                Ok(State::Break)
            }
            other => panic!("Unsupported tag: {}", other),
        },
        Token::EOF => Err(reader.err_end_of_file()),
        _ => Ok(State::Root),
    }
}

#[cfg(test)]
mod tests {
    use crate::LayoutReader;
    use crate::states::{FSMContext, State};

    #[test]
    fn test() {
        let mut context = FSMContext::default();

        let xml = std::fs::read_to_string(
            "/home/irisu/bevy_declarative_ui/bevy_declarative_ui/assets/injection_count_10.xml",
        )
        .unwrap();
        let mut reader = LayoutReader::new(
            &xml,
            "/home/irisu/bevy_declarative_ui/bevy_declarative_ui/assets/injection_count_10.xml",
        );

        let mut state = State::Layout;
        while state != State::Break {
            context.token = reader.read().unwrap();
            let result = state.execute(&mut context, &mut reader);
            if result.is_err() {
                panic!("{}", result.err().unwrap());
            } else {
                state = result.unwrap();
            }
        }

        println!();
    }
}
