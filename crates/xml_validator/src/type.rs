use std::sync::atomic::AtomicU16;

use enum_dispatch::enum_dispatch;
use regex::Regex;

static REFERENCE_ID: AtomicU16 = AtomicU16::new(0);

pub struct TypeReference(u16);

#[derive(Default)]
pub struct TypeMetadata {
    path: String,
}

pub struct TypeDeclaration {
    reference: TypeReference,
    metadata: TypeMetadata,
    restriction: BaseType,
    values: Vec<ValueType>
}

impl TypeDeclaration {
    pub fn new(path: impl Into<String>, restriction: BaseType, values: Vec<ValueType>) -> Self {
        Self {
            reference: TypeReference(REFERENCE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)),
            metadata: TypeMetadata {
                path: path.into() 
            },
            restriction,
            values,
        }
    }

    pub fn validate_value(&self, declarations: &[TypeDeclaration], value: &str) -> bool {
        self.restriction.validate_value(value)
            && self.values
                .iter()
                .any(|allowed_value| allowed_value.validate_value(declarations, value))
    }
}

#[derive(Default)]
pub enum BaseType {
    Integer,
    Float,
    Double,
    Boolean,
    #[default]
    String,
}

impl BaseType {
    pub fn validate_value(&self, value: &str) -> bool {
        match self {
            BaseType::Integer => value.parse::<i32>().is_ok(),
            BaseType::Float   => value.parse::<f32>().is_ok(),
            BaseType::Double  => value.parse::<f64>().is_ok(),
            BaseType::Boolean => value.parse::<bool>().is_ok(),
            BaseType::String  => true,
        }
    }
}

#[enum_dispatch]
pub trait Value {
    fn validate_value(&self, declarations: &[TypeDeclaration], value: &str) -> bool;
}

impl Value for Regex {
    fn validate_value(&self, _: &[TypeDeclaration], value: &str) -> bool {
        self.is_match(value)
    }
}

impl Value for String {
    fn validate_value(&self, _: &[TypeDeclaration], value: &str) -> bool {
        self.eq(value)
    }
}

impl Value for Vec<TypeReference> {
    fn validate_value(&self, declarations: &[TypeDeclaration], value: &str) -> bool {
        self.iter().any(|ty| declarations[ty.0 as usize].validate_value(declarations, value))
    }
}

#[enum_dispatch(Value)]
pub enum ValueType {
    Pattern(Regex),
    Enumeration(String),
    Union(Vec<TypeReference>)
}

fn test() {
    //Box sizing
    TypeDeclaration {
        reference: TypeReference(0),
        restriction: BaseType::String,
        metadata: TypeMetadata {
            path: "bevy::prelude::BoxSizing".into(),
        },
        values: vec![
            ValueType::Enumeration("BorderBox".into()),
            ValueType::Enumeration("ContentBox".into()),
        ],
    };

    //Expr string
    TypeDeclaration {
        reference: TypeReference(1),
        restriction: BaseType::String,
        metadata: TypeMetadata::default(),
        values: vec![
            ValueType::Pattern(Regex::new(r#"\{.*\}"#).unwrap()) //TODO Return error
        ],
    };

    //Box sizing union
    TypeDeclaration {
        reference: TypeReference(2),
        restriction: BaseType::String,
        metadata: TypeMetadata::default(),
        values: vec![ 
            ValueType::Union(vec![
                TypeReference(0),
                TypeReference(1),
            ])
        ],
    };
}
