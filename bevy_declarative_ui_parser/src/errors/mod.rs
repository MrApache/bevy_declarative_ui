mod error_context;
mod duplicates;
mod io_error;

pub use duplicates::Duplicates;
pub use error_context::ErrorContext;
use io_error::IoError;

use crate::position::Location;

#[non_exhaustive]
#[derive(PartialEq, Debug)]
pub enum XmlLayoutError {
    Io(IoError),
    Utf8Error(std::str::Utf8Error),

    UnexpectedChar {
        context: ErrorContext,
        expected: char,
        found: char,
    },
    ExpectedIdentifier {
        context: ErrorContext,
        found: char,
    },
    InvalidChar {
        context: ErrorContext,
        char: char
    },

    EndOfFile {
        file: String,
        location: Location,
    },

    MissingLayout {
        file: String,
    },

    MissingAttribute {
        context: ErrorContext,
        attribute: &'static str,
    },

    EmptyAttribute {
        context: ErrorContext,
        attribute: &'static str,
    },

    UnexpectedTag {
        context: ErrorContext,
        current: String,
        expected: Vec<&'static str>,
    },

    MismatchedEndTag {
        context: ErrorContext,
        current: String,
        expected: &'static str,
    },

    ExceptedValue {
        context: ErrorContext,
    },

    MissingParameter {
        context: ErrorContext,
        name: String,
    },

    UnknownBindingType {
        context: ErrorContext,
        name: String,
    },

    DuplicateParam {
        context: Duplicates,
        name: String,
    }
}

impl From<std::io::Error> for XmlLayoutError {
    fn from(error: std::io::Error) -> Self {
        XmlLayoutError::Io(IoError(error))
    }
}

impl std::error::Error for XmlLayoutError {}

impl std::fmt::Display for XmlLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlLayoutError::Io(error) => write!(f, "Could not load file: {error}"),
            XmlLayoutError::Utf8Error(error) => write!(f, "Could not read file: {error}"),
            XmlLayoutError::UnexpectedChar { context, expected, found, }
                => write_single_error(format!("Unexpected char: Expected '{expected}', but found '{found}'"), context, f),

            XmlLayoutError::ExpectedIdentifier { context, found, }
                => write_single_error(format!("Expected identifier, but found '{found}'"), context, f),

            XmlLayoutError::InvalidChar { context, char }
                => write_single_error(format!("Invalid character: '{char}'"), context, f),

            XmlLayoutError::EndOfFile { file, location } => write!(f, "[{file}:{location}] Unexpected end of file"),
            XmlLayoutError::MissingLayout { file } => write!(f, "[{file}:0:0] Missing <Layout> tag"),

            XmlLayoutError::MissingAttribute { context, attribute }
                => write_single_error(format!("Missing attribute: {attribute}"), context, f),

            XmlLayoutError::EmptyAttribute { context, attribute }
                => write_single_error(format!("Empty attribute: {attribute}"), context, f),

            XmlLayoutError::UnexpectedTag { context, current, expected }
                => write_single_error(format!("Unexpected tag. Expected: {expected:?}, but found {current}"), context, f),

            XmlLayoutError::MismatchedEndTag { context, current, expected }
                => write_single_error(format!("Mismatched end tag. Expected {expected}, but found {current}"), context, f),

            XmlLayoutError::ExceptedValue { context }
                => write_single_error("Expected value", context, f),

            XmlLayoutError::MissingParameter { context, name }
                => write_single_error(format!("Missing parameter: {name}"), context, f),

            XmlLayoutError::UnknownBindingType { context, name }
                => write_single_error(format!("Unknown binding type: {name}"), context, f),

            XmlLayoutError::DuplicateParam { context, name }
                => write_multiple_error(format!("Parameter '{name}' specified more than once"), context, f)
        }
    }
}

fn write_single_error(message: impl Into<String>, context: &ErrorContext, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let file = &context.file;
    let location = &context.location;
    let error = &context.error;

    writeln!(f, "error: {}", message.into())?;
    writeln!(f, "  --> {file}:{}:{}", location.line(), location.column())?;
    writeln!(f, "   |")?;
    writeln!(f, "   |      {}", error.source())?;
    writeln!(
        f,
        "   |      {}{}",
        " ".repeat(error.start()),
        "^".repeat(error.length())
    )?;
    Ok(())
}

fn write_multiple_error(message: impl Into<String>, context: &Duplicates, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let file = &context.file;
    let location = &context.location;

    writeln!(f, "error: {}", message.into())?;
    writeln!(f, "  --> {file}:{}:{}", location.line(), location.column())?;
    writeln!(f, "   |")?;
    writeln!(f, "   |      {}", context.source)?;

    let mut marker_line = vec![' '; context.source.len()];

    for span in context.errors.iter() {
        let start = span.start.min(context.source.len());
        let end = (span.start + span.length).min(context.source.len());

        for i in start..end {
            marker_line[i] = '^';
        }
    }

    writeln!(f, "   |      {}", marker_line.iter().collect::<String>())?;
    Ok(())
}