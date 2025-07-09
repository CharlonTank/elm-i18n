use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::parser::parse_i18n_file;
use crate::types::Translation;

pub fn add_translation(path: &Path, translation: &Translation) -> Result<()> {
    // Create backup
    let backup_path = path.with_extension("elm.bak");
    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    
    // Parse the file to find insertion points
    let parse_result = parse_i18n_file(path)?;
    
    // Add to Translations type
    let type_insertion_line = find_last_field_line(&lines, parse_result.type_start_line, parse_result.type_end_line);
    insert_type_field(&mut lines, type_insertion_line, &translation.key, &translation.type_signature);
    
    // Add to translationsEn
    let en_insertion_line = find_last_field_line(&lines, parse_result.en_start_line, parse_result.en_end_line);
    insert_record_field(&mut lines, en_insertion_line, &translation.key, &translation.en, translation.is_function);
    
    // Add to translationsFr
    let fr_insertion_line = find_last_field_line(&lines, parse_result.fr_start_line, parse_result.fr_end_line);
    insert_record_field(&mut lines, fr_insertion_line, &translation.key, &translation.fr, translation.is_function);
    
    // Write the modified content
    let new_content = lines.join("\n");
    fs::write(path, new_content)
        .with_context(|| format!("Failed to write to {}", path.display()))?;
    
    Ok(())
}

fn find_last_field_line(lines: &[String], start: usize, end: usize) -> usize {
    // Find the last line with a field definition before the closing brace
    for i in (start..end).rev() {
        let line = &lines[i];
        if line.contains(" = ") || line.contains(" : ") {
            return i;
        }
    }
    // If no fields found, insert after the opening brace
    start
}

fn insert_type_field(lines: &mut Vec<String>, after_line: usize, key: &str, type_sig: &Option<String>) {
    let type_annotation = type_sig.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("String");
    let new_line = format!("    , {} : {}", key, type_annotation);
    lines.insert(after_line + 1, new_line);
}

fn insert_record_field(lines: &mut Vec<String>, after_line: usize, key: &str, value: &str, is_function: bool) {
    if is_function {
        // Handle multiline function definitions
        let indented_value = value.lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    line.to_string()
                } else {
                    format!("        {}", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let new_line = format!("    , {} = {}", key, indented_value);
        lines.insert(after_line + 1, new_line);
    } else {
        // Simple string value
        let escaped_value = escape_elm_string(value);
        let new_line = format!("    , {} = \"{}\"", key, escaped_value);
        lines.insert(after_line + 1, new_line);
    }
}

fn escape_elm_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub fn create_i18n_file(path: &Path, template: &str) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    fs::write(path, template)
        .with_context(|| format!("Failed to write I18n.elm to {}", path.display()))?;
    
    Ok(())
}

pub fn remove_translation(path: &Path, key: &str) -> Result<()> {
    // Create backup
    let backup_path = path.with_extension("elm.bak");
    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    
    // Parse the file to find the translation
    let parse_result = parse_i18n_file(path)?;
    
    // Check if the key exists
    if !parse_result.translations.contains_key(key) {
        anyhow::bail!("Translation '{}' not found", key);
    }
    
    // Remove from Translations type
    remove_type_field(&mut lines, key);
    
    // Remove from translationsEn
    remove_record_field(&mut lines, key);
    
    // Remove from translationsFr  
    remove_record_field(&mut lines, key);
    
    // Write the modified content
    let new_content = lines.join("\n");
    fs::write(path, new_content)
        .with_context(|| format!("Failed to write to {}", path.display()))?;
    
    Ok(())
}

fn remove_type_field(lines: &mut Vec<String>, key: &str) {
    // Find and remove the line containing the type field
    lines.retain(|line| {
        !line.contains(&format!("{} :", key)) || !line.trim().starts_with(',') && !line.trim().starts_with('}')
    });
}

fn remove_record_field(lines: &mut Vec<String>, key: &str) {
    let mut key_line_idx = None;
    let mut is_multiline = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.contains(&format!("{} =", key)) {
            key_line_idx = Some(i);
            // Check if it's a multiline value (function)
            is_multiline = line.contains('\\') || line.contains("case");
            break;
        }
    }
    
    if let Some(idx) = key_line_idx {
        if is_multiline {
            // Remove the key line and all continuation lines
            let mut lines_to_remove = vec![idx];
            let mut j = idx + 1;
            
            while j < lines.len() {
                // Stop when we hit a line that starts a new field
                if lines[j].trim_start().starts_with(',') || 
                   lines[j].trim_start().starts_with('}') ||
                   (lines[j].contains(" = ") && !lines[j].trim_start().starts_with("        ")) {
                    break;
                }
                lines_to_remove.push(j);
                j += 1;
            }
            
            // Remove lines in reverse order to maintain indices
            for &line_idx in lines_to_remove.iter().rev() {
                lines.remove(line_idx);
            }
        } else {
            // Simple single-line removal
            lines.remove(idx);
        }
    }
}