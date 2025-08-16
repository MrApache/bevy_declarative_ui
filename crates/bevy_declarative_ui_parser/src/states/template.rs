use crate::lexer::Token;
use crate::{
    Id, LayoutReader, XmlLayout, XmlLayoutError,
    states::{FSMContext, State},
};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMPLATE_ID: AtomicU64 = AtomicU64::new(0);

pub(super) fn template_execute(
    context: &mut FSMContext,
    reader: &mut LayoutReader,
) -> Result<State, XmlLayoutError> {
    let template = context.layout.templates.last_mut().unwrap();
    let (template_layout, id) = parse_template_layout(reader)?;

    template.id = id;
    template.nodes.extend(template_layout.root_nodes);
    Ok(State::Root)
}

fn parse_template_layout(reader: &mut LayoutReader) -> Result<(XmlLayout, Id), XmlLayoutError> {
    let mut template_context: FSMContext = FSMContext::default();

    let mut state = State::Content;
    loop {
        let token = reader.read()?;
        template_context.token = token;
        if let Token::TagEnd(tag) = &template_context.token {
            if tag.identifier() == "ItemTemplate" {
                break;
            }
        }
        state = state.execute(&mut template_context, reader)?
    }

    let id = Id::Template(TEMPLATE_ID.fetch_add(1, Ordering::SeqCst));
    let first = template_context.container_tmp.get_mut(0).unwrap();
    first.inner.id = id.clone();
    template_context
        .layout
        .root_nodes
        .extend(template_context.container_tmp.into_iter().map(|c| c.inner));
    Ok((template_context.layout, id))
}
