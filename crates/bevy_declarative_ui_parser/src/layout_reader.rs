use std::io::Cursor;
use crate::errors::XmlLayoutError;
use crate::position::{Location, Span};
use crate::states::{FSMContext, State};
use crate::XmlLayout;

pub struct LayoutReader<'a> {
    pub(crate) file: String,
    pub(crate) inner: Cursor<&'a str>,
    pub(crate) location: Location,
    pub(crate) start_of_line: usize,
    pub(crate) current_span: Span,
}

impl<'a> LayoutReader<'a> {
    pub fn new(content: &'a str, file: &'a str) -> Self {
        Self {
            file: String::from(file),
            inner: Cursor::new(content),
            location: Location::new(1, 1, 0),
            current_span: Span::new(0, 0),
            start_of_line: 1,
        }
    }

    pub fn parse(&mut self) -> Result<XmlLayout, XmlLayoutError> {
        let mut context = FSMContext::default();
        let mut state = State::Layout;
        while state != State::Break {
            context.token = self.read()?;
            let result = state.execute(&mut context, self);
            //println!("State: {state}");
            state = result?;
        }

        Ok(context.layout)
    }
}