use crate::lexer::Token;
use crate::states::{FSMContext, State};
use crate::{LayoutReader, XmlLayoutError};

pub(super) fn content_execute(
    context: &mut FSMContext,
    reader: &LayoutReader,
) -> Result<State, XmlLayoutError> {
    match &context.token {
        Token::TagStart(tag) => match tag.identifier() {
            "Use" => Ok(State::Use),
            "ItemTemplate" => context.create_template(reader, tag.clone()),
            "Container" => context.create_nested_container(reader, tag.clone()),
            other => panic!("TODO: Unknown tag: {other}"),
        },
        Token::TagEmpty(tag) => context.create_component_node(reader, tag.clone()),
        Token::EOF => Err(reader.err_end_of_file()),
        _ => Ok(State::Content),
    }
}
