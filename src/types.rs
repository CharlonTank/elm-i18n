use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a translation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub key: String,
    pub en: String,
    pub fr: String,
    pub is_function: bool,
    pub type_signature: Option<String>,
}

/// Represents a field in the Translations type
#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub type_annotation: String,
}

/// Represents a field in a record
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: String,
    pub value: String,
}

/// Result of parsing an I18n file
#[derive(Debug)]
pub struct ParseResult {
    pub type_start_line: usize,
    pub type_end_line: usize,
    pub en_start_line: usize,
    pub en_end_line: usize,
    pub fr_start_line: usize,
    pub fr_end_line: usize,
    pub translations: HashMap<String, Translation>,
}