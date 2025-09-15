use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::{ParseResult, Translation, TypeField, RecordField};

pub fn parse_i18n_file(path: &Path) -> Result<ParseResult> {
    parse_i18n_file_with_record_name(path, "Translations")
}

pub fn parse_i18n_file_with_record_name(path: &Path, record_name: &str) -> Result<ParseResult> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    
    // Find the type definition with custom record name
    let type_bounds = find_type_definition_with_name(&lines, record_name)?;
    
    // Try different naming conventions for language functions
    // First try the standard translationsEn/translationsFr
    let en_bounds = find_translation_record_with_type(&lines, "translationsEn", record_name)
        .or_else(|_| find_translation_record_with_type(&lines, "en", record_name))?;
    let fr_bounds = find_translation_record_with_type(&lines, "translationsFr", record_name)
        .or_else(|_| find_translation_record_with_type(&lines, "fr", record_name))?;
    
    // Parse all translations
    let type_fields = parse_type_fields(&lines, type_bounds.0, type_bounds.1)?;
    let en_fields = parse_record_fields(&lines, en_bounds.0, en_bounds.1)?;
    let fr_fields = parse_record_fields(&lines, fr_bounds.0, fr_bounds.1)?;
    
    // Build translation map
    let mut translations = HashMap::new();
    
    for type_field in &type_fields {
        let en_value = en_fields.iter()
            .find(|f| f.name == type_field.name)
            .map(|f| f.value.clone())
            .unwrap_or_default();
            
        let fr_value = fr_fields.iter()
            .find(|f| f.name == type_field.name)
            .map(|f| f.value.clone())
            .unwrap_or_default();
            
        let is_function = type_field.type_annotation.contains("->");
        
        translations.insert(
            type_field.name.clone(),
            Translation {
                key: type_field.name.clone(),
                en: en_value,
                fr: fr_value,
                is_function,
                type_signature: if is_function {
                    Some(type_field.type_annotation.clone())
                } else {
                    None
                },
            },
        );
    }
    
    Ok(ParseResult {
        type_start_line: type_bounds.0,
        type_end_line: type_bounds.1,
        en_start_line: en_bounds.0,
        en_end_line: en_bounds.1,
        fr_start_line: fr_bounds.0,
        fr_end_line: fr_bounds.1,
        translations,
    })
}

fn find_type_definition(lines: &[&str]) -> Result<(usize, usize)> {
    find_type_definition_with_name(lines, "Translations")
}

fn find_type_definition_with_name(lines: &[&str], record_name: &str) -> Result<(usize, usize)> {
    let mut start = None;
    let mut brace_count = 0;
    
    for (i, line) in lines.iter().enumerate() {
        if line.contains(&format!("type alias {}", record_name)) {
            start = Some(i);
            continue;
        }
        
        if let Some(_) = start {
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();
            
            if brace_count == 0 && line.contains('}') {
                return Ok((start.unwrap(), i));
            }
        }
    }
    
    anyhow::bail!("Could not find {} type definition", record_name)
}

fn find_translation_record(lines: &[&str], name: &str) -> Result<(usize, usize)> {
    find_translation_record_with_type(lines, name, "Translations")
}

fn find_translation_record_with_type(lines: &[&str], name: &str, record_type: &str) -> Result<(usize, usize)> {
    let mut start = None;
    let mut brace_count = 0;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(name) && line.contains(record_type) {
            start = Some(i);
            continue;
        }
        
        if let Some(_) = start {
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();
            
            if brace_count == 0 && line.trim().starts_with('}') {
                return Ok((start.unwrap(), i));
            }
        }
    }
    
    anyhow::bail!("Could not find {} definition", name)
}

fn parse_type_fields(lines: &[&str], start: usize, end: usize) -> Result<Vec<TypeField>> {
    let mut fields = Vec::new();
    let field_regex = Regex::new(r"^\s*,?\s*(\w+)\s*:\s*(.+)$")?;
    
    for i in (start + 1)..end {
        let line = lines[i];
        if let Some(captures) = field_regex.captures(line) {
            fields.push(TypeField {
                name: captures[1].to_string(),
                type_annotation: captures[2].trim().to_string(),
            });
        }
    }
    
    Ok(fields)
}

fn parse_record_fields(lines: &[&str], start: usize, end: usize) -> Result<Vec<RecordField>> {
    let mut fields = Vec::new();
    let field_regex = Regex::new(r"^\s*,?\s*(\w+)\s*=\s*(.*)$")?;
    
    let mut i = start + 1;
    while i < end {
        let line = lines[i];
        
        if let Some(captures) = field_regex.captures(line) {
            let name = captures[1].to_string();
            let mut value = captures[2].to_string();
            
            // Check if this is a multiline value (function)
            if value.starts_with('\\') || value.contains("case") {
                let mut j = i;
                
                // Collect all lines until we find the end of the function
                while j < end {
                    if j > i {
                        value.push('\n');
                        value.push_str(&format!("        {}", lines[j].trim_start()));
                    }
                    
                    // Simple heuristic: look for a line that doesn't start with whitespace
                    // after we've started collecting
                    if j > i && !lines[j + 1].starts_with("        ") {
                        i = j;
                        break;
                    }
                    
                    j += 1;
                }
            }
            
            fields.push(RecordField {
                name,
                value: value.trim().to_string(),
            });
        }
        
        i += 1;
    }
    
    Ok(fields)
}

pub fn check_key_exists(path: &Path, key: &str) -> Result<Option<Translation>> {
    check_key_exists_with_record_name(path, key, "Translations")
}

pub fn check_key_exists_with_record_name(path: &Path, key: &str, record_name: &str) -> Result<Option<Translation>> {
    let result = parse_i18n_file_with_record_name(path, record_name)?;
    Ok(result.translations.get(key).cloned())
}