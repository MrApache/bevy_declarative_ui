use crate::XmlTag;
use crate::errors::Duplicates;
use crate::errors::ErrorContext;
use crate::errors::XmlLayoutError;
use crate::layout_reader::LayoutReader;
use crate::lexer::Value;
use crate::position::*;

impl<'a> LayoutReader<'a> {
    pub(super) fn err_missing_layout(&self) -> XmlLayoutError {
        XmlLayoutError::MissingLayout {
            file: self.file.to_string(),
        }
    }

    pub(super) fn err_end_of_file(&self) -> XmlLayoutError {
        XmlLayoutError::EndOfFile {
            file: self.file.to_string(),
            location: self.location,
        }
    }

    pub(super) fn err_expected_value(&self, tag: &XmlTag) -> XmlLayoutError {
        XmlLayoutError::ExceptedValue {
            context: self.context(
                tag.span(),
                self.location(),
                tag.location(),
                tag.identifier().len(),
            ),
        }
    }

    pub(super) fn err_unexpected_tag(
        &self,
        tag: XmlTag,
        expected: Vec<&'static str>,
    ) -> XmlLayoutError {
        XmlLayoutError::UnexpectedTag {
            context: self.context(
                tag.span(),
                self.location(),
                tag.location(),
                tag.identifier().len(),
            ),
            current: tag.into_identifier(),
            expected,
        }
    }

    pub(super) fn err_empty_attribute(
        &self,
        tag: &XmlTag,
        attribute_name: &'static str,
    ) -> XmlLayoutError {
        let attribute = tag
            .attributes()
            .iter()
            .find(|a| a.name() == attribute_name)
            .unwrap();

        XmlLayoutError::EmptyAttribute {
            context: self.context(
                tag.span(),
                attribute.location(),
                attribute.location(),
                attribute_name.len() + 3,
            ),
            attribute: attribute_name,
        }
    }

    pub(super) fn err_missing_attribute(
        &self,
        tag: &XmlTag,
        attribute_name: &'static str,
    ) -> XmlLayoutError {
        XmlLayoutError::MissingAttribute {
            context: self.context(
                tag.span(),
                tag.location(),
                tag.location(),
                tag.identifier().len(),
            ),
            attribute: attribute_name,
        }
    }

    pub(super) fn err_missing_parameter(
        &self,
        value: &Value,
        binding_type: &str,
        parameter_name: &'static str,
    ) -> XmlLayoutError {
        XmlLayoutError::MissingParameter {
            context: self.context(
                value.span(),
                value.location(),
                value.location(),
                binding_type.len(),
            ),
            name: parameter_name.to_string(),
        }
    }

    pub(super) fn err_unknown_binding_type(
        &self,
        value: &Value,
        binding_name: &str,
    ) -> XmlLayoutError {
        XmlLayoutError::UnknownBindingType {
            context: self.context(
                value.span(),
                value.location(),
                value.location(),
                binding_name.len(),
            ),
            name: binding_name.to_string(),
        }
    }

    pub(super) fn err_duplicate_param(
        &self,
        source: &Value,
        duplicates: &[Value],
        parameter: &str,
    ) -> XmlLayoutError {
        XmlLayoutError::DuplicateParam {
            context: Duplicates::new(
                self.file().to_string(),
                source.location(),
                self.substring_other(&source.span()),
                self.attribute_error_list(source, duplicates),
            ),
            name: parameter.to_string(),
        }
    }

    fn make_error_span(&self, span: &Span, location: Location, length: usize) -> ErrorSpan {
        let inner = self.substring_other(span);
        let start = location.column() - (span.start() - location.position());
        ErrorSpan::new(inner, start, length)
    }

    fn attribute_error_span(&self, span: &Span, location: Location, length: usize) -> ErrorSpan {
        let inner = self.substring_other(span);
        let start = (location.column() + 1) - (span.start() - location.position()); //Skip '{'
        ErrorSpan::new(inner, start, length)
    }

    fn attribute_error_list(&self, source: &Value, values: &[Value]) -> Vec<SimpleErrorSpan> {
        let mut list = vec![];
        values.iter().for_each(|value| {
            let offset = value.location().column - source.location().column();
            let start =
                offset + value.location.column() - (value.span.start() - value.location.position());
            list.push(SimpleErrorSpan {
                start,
                length: value.span.len(),
            });
        });

        list
    }

    fn context(
        &self,
        span: Span,
        location: Location,
        error_location: Location,
        length: usize,
    ) -> ErrorContext {
        ErrorContext::new(
            self.file().to_string(),
            location,
            self.make_error_span(&span, error_location, length),
        )
    }
}
