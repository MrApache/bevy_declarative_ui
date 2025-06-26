use xml_parser::error::XmlReaderError;
use xml_parser::span::{ErrorSpan, Location};

#[derive(Debug)]
pub struct IoError(std::io::Error);

impl From<std::io::Error> for XmlLayoutError {
    fn from(error: std::io::Error) -> Self {
        XmlLayoutError::Io(IoError(error))
    }
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

#[non_exhaustive]
#[derive(PartialEq, Debug)]
pub enum XmlLayoutError {
    Io(IoError),
    Utf8Error(std::str::Utf8Error),
    Reader(XmlReaderError),

    EndOfFile {
        file: String,
        location: Location,
    },

    MissingLayout {
        file: String,
    },

    MultipleLayouts {
        file: String,
        location: Location,
        error: ErrorSpan,
    },

    MissingAttribute {
        file: String,
        location: Location,
        error: ErrorSpan,
        attribute: &'static str,
    },

    EmptyAttribute {
        file: String,
        location: Location,
        error: ErrorSpan,
        attribute: &'static str,
    },

    UnexpectedTag {
        file: String,
        location: Location,
        error: ErrorSpan,
        current: String,
        expected: Vec<&'static str>,
    },

    MismatchedEndTag {
        file: String,
        location: Location,
        error: ErrorSpan,
        current: String,
        expected: &'static str,
    },

    ExceptedValue {
        file: String,
        location: Location,
        error: ErrorSpan,
    },

    //TODO remove this
    EmptyLayout, //Warning
    EmptyGlobalResources, //Warning
    EmptyLocalResources,  //Warning
    EmptyTemplate, //Warning
}

impl std::error::Error for XmlLayoutError {}

impl std::fmt::Display for XmlLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (message, file, location, error) = match self {
            XmlLayoutError::Io(error) => {
                write!(f, "Could not load file: {error}")?;
                return Ok(());
            }
            XmlLayoutError::Utf8Error(error) => {
                write!(f, "Could not read file: {error}")?;
                return Ok(());
            }

            XmlLayoutError::Reader(error) => {
                write!(f, "{}", error)?;
                return Ok(());
            }

            XmlLayoutError::EndOfFile {
                file, location
            } => {
                write!(f, "[{file}:{location}] Unexpected end of file")?;
                return Ok(());
            }

            XmlLayoutError::MissingLayout {
                file
            } =>  {
                write!(f, "[{file}:0:0] Missing <Layout> tag")?;
                return Ok(());
            },

            XmlLayoutError::MultipleLayouts {
                file, location, error
            } => ("Multiple <Layout> tag".to_string(), file, location, error),

            XmlLayoutError::MissingAttribute {
                file, location, error, attribute
            } => (format!("Missing attribute: {attribute}"), file, location, error),

            XmlLayoutError::EmptyAttribute {
                file, location, error, attribute
            } => (format!("Empty attribute: {attribute}"), file, location ,error),

            XmlLayoutError::UnexpectedTag {
                file, location, error, current, expected
            } => (format!("Unexpected tag. Expected {expected:?}, but found {current}"), file, location, error),

            XmlLayoutError::MismatchedEndTag {
                file, location, error, current, expected
            } => (format!("Mismatched end tag. Expected {expected}, but found {current}"), file, location, error),

            XmlLayoutError::ExceptedValue {
                file, location, error
            } => ("Expected value".to_string(), file, location, error),

            XmlLayoutError::EmptyLayout => {
                write!(f, "[Warning] Empty layout")?;
                return Ok(());
            }
            XmlLayoutError::EmptyGlobalResources => {
                write!(f, "[Warning] Empty global resources")?;
                return Ok(());
            }
            XmlLayoutError::EmptyLocalResources => {
                write!(f, "[Warning] Empty local resources")?;
                return Ok(());
            }
            XmlLayoutError::EmptyTemplate => {
                write!(f, "[Warning] Empty template")?;
                return Ok(());
            }
        };
        writeln!(f, "error: {message}")?;
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
}

impl From<XmlReaderError> for XmlLayoutError {
    fn from(error: XmlReaderError) -> Self {
        XmlLayoutError::Reader(error)
    }
}