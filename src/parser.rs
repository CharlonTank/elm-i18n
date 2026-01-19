use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::{ParseResult, Translation, TypeField, RecordField};

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

    // Track brace depth to only capture top-level fields
    // Depth 0 = before first {, Depth 1 = inside top-level record, Depth 2+ = inside nested records
    let mut brace_depth = 0;

    for i in (start + 1)..end {
        let line = lines[i];

        // Update brace depth BEFORE checking for field
        // Count opening braces
        let open_braces = line.matches('{').count();
        let close_braces = line.matches('}').count();

        // Only capture fields at depth 1 (top level of the record)
        // We need to be at depth 1 before the line's braces are processed
        let current_depth = brace_depth;

        // Update depth after capturing current depth
        brace_depth += open_braces;
        brace_depth = brace_depth.saturating_sub(close_braces);

        // Only capture fields at the top level (depth 1)
        // Note: first line with { puts us at depth 1, so fields are at depth 1
        if current_depth == 1 || (current_depth == 0 && open_braces > 0) {
            if let Some(captures) = field_regex.captures(line) {
                fields.push(TypeField {
                    name: captures[1].to_string(),
                    type_annotation: captures[2].trim().to_string(),
                });
            }
        }
    }

    Ok(fields)
}

fn parse_record_fields(lines: &[&str], start: usize, end: usize) -> Result<Vec<RecordField>> {
    let mut fields = Vec::new();
    let field_regex = Regex::new(r"^\s*,?\s*(\w+)\s*=\s*(.*)$")?;
    // Regex to detect if a line starts a new field (starts with optional comma then identifier = ...)
    let new_field_regex = Regex::new(r"^\s*,?\s*\w+\s*=")?;

    let mut i = start + 1;
    while i < end {
        let line = lines[i];

        if let Some(captures) = field_regex.captures(line) {
            let name = captures[1].to_string();
            let mut value = captures[2].to_string();

            // Check if this is a multiline value (function or case expression)
            // Only treat as multiline if the next line doesn't start a new field
            if (value.starts_with('\\') || value.contains("case")) && i + 1 < end {
                // Check if next line is a continuation (not a new field)
                let next_line = lines[i + 1];
                if !new_field_regex.is_match(next_line) {
                    let mut j = i + 1;

                    // Collect all lines until we find a new field
                    while j < end {
                        let current = lines[j];

                        // Stop if this line starts a new field
                        if new_field_regex.is_match(current) {
                            break;
                        }

                        // Add this line to the value
                        value.push('\n');
                        value.push_str(&format!("        {}", current.trim_start()));

                        j += 1;
                    }

                    // Position i at the last line we consumed
                    i = j - 1;
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

pub fn check_key_exists_with_record_name(path: &Path, key: &str, record_name: &str) -> Result<Option<Translation>> {
    let result = parse_i18n_file_with_record_name(path, record_name)?;
    Ok(result.translations.get(key).cloned())
}