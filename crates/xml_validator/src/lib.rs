pub mod r#type;
mod element;

use std::collections::HashMap;
use crate::{element::{Group, GroupDeclaration}, r#type::{TypeDeclaration, TypeReference}};

pub struct Schema {
    references: HashMap<String, TypeReference>, //Name, Type
    types: Vec<TypeDeclaration>,
    groups: HashMap<String, GroupDeclaration>,
    //elements: Vec<Element>
}
