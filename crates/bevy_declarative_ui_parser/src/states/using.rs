use crate::XmlLayoutError;
use crate::lexer::Token;
use crate::states::{FSMContext, State};

pub(super) fn use_execute(context: &mut FSMContext) -> Result<State, XmlLayoutError> {
    match &context.token {
        Token::Text(text) => {
            context.layout.usings.insert(text.clone());
            Ok(State::Use)
        }
        Token::TagEnd(tag) => {
            if tag.identifier() != "Use" {
                panic!("TODO: Tag mismatch");
            }
            Ok(State::Content)
        }
        other => panic!("Unsupported token: {}", other),
    }
}
