use crate::lexer::Token;
use crate::{LayoutReader, XmlLayoutError};
use crate::states::{FSMContext, State};

pub(super) fn layout_execute(context: &mut FSMContext, reader: &LayoutReader) -> Result<State, XmlLayoutError> {
    context.create_root_container();
    match &context.token {
        Token::TagStart(tag) => {
            if tag.identifier() != "Layout" {
                return Err(reader.err_missing_layout());
            }
            Ok(State::Content)
        }
        _ => Err(reader.err_missing_layout())
    }
}
