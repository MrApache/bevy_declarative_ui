use xml_parser::span::{ErrorSpan, Location, Span};
use crate::error::XmlLayoutError;
use crate::layout_reader::LayoutReader;
use crate::XmlTag;

impl<'a> LayoutReader<'a> {
    pub(super) fn err_multiple_layouts(&self, tag: &XmlTag) -> XmlLayoutError {
        XmlLayoutError::MultipleLayouts {
            file: self.inner.file().to_string(),
            location: self.inner.location(),
            error: ErrorSpan::new(self.inner.substring_other(&tag.span()), tag.location().column(), tag.identifier().len()),
        }
    }

    pub(super) fn err_missing_layout(&self) -> XmlLayoutError {
        XmlLayoutError::MissingLayout {
            file: self.inner.file().to_string(),
        }
    }

    pub(super) fn err_end_of_file(&self) -> XmlLayoutError {
        XmlLayoutError::EndOfFile {
            file: self.inner.file().to_string(),
            location: self.inner.location(),
        }
    }

    pub(super) fn err_expected_value(&self, tag: &XmlTag) -> XmlLayoutError {
        XmlLayoutError::ExceptedValue {
            file: self.inner.file().to_string(),
            location: self.inner.location(),
            error: ErrorSpan::new(self.inner.substring_other(&tag.span()), tag.location().column(), tag.identifier().len()),
        }
    }

    pub(super) fn err_unexpected_tag(&self, tag: XmlTag, expected: Vec<&'static str>) -> XmlLayoutError {
        XmlLayoutError::UnexpectedTag {
            file: self.inner.file().to_string(),
            location: self.inner.location(),
            error: self.error_span(&tag.span(), tag.location(), tag.identifier().len()),
            current: tag.into_identifier(),
            expected,
        }
    }

    pub(super) fn err_empty_attribute(&self, tag: &XmlTag, attribute_name: &'static str) -> XmlLayoutError {
        let attribute = tag.attributes()
            .iter()
            .find(|a| a.name() == attribute_name)
            .unwrap();

        //let start = self.location.column - (self.current_span.start - self.start_of_line.min(self.current_span.start)) - 1;
        XmlLayoutError::EmptyAttribute {
            file: self.inner.file().to_string(),
            location: attribute.location(),
            error: self.error_span(&tag.span(), attribute.location(), attribute_name.len() + 3),
            attribute: attribute_name
        }
    }

    pub(super) fn err_missing_attribute(&self, tag: &XmlTag, attribute_name: &'static str) -> XmlLayoutError {
        XmlLayoutError::MissingAttribute {
            file: self.inner.file().to_string(),
            location: tag.location(),
            error: self.error_span(&tag.span(), tag.location(), tag.identifier().len()),
            attribute: attribute_name,
        }
    }

    fn error_span(&self, span: &Span, location: Location, length: usize) -> ErrorSpan {
        let inner = self.inner.substring_other(span);
        let start = location.column() - (span.start() - location.position());
        ErrorSpan::new(inner, start, length)
    }
}