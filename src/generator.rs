use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::parser::parse_i18n_file_with_record_name;
use crate::types::Translation;

pub fn add_translation_with_record_name(path: &Path, translation: &Translation, record_name: &str) -> Result<()> {
    // Create backup
    let backup_path = path.with_extension("elm.bak");
    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Parse the file to find insertion points
    let parse_result = parse_i18n_file_with_record_name(path, record_name)?;

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

    // Remove backup file after successful write
    let _ = fs::remove_file(&backup_path);

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

pub fn remove_translation_with_record_name(path: &Path, key: &str, record_name: &str) -> Result<()> {
    // Create backup
    let backup_path = path.with_extension("elm.bak");
    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Parse the file to find the translation
    let parse_result = parse_i18n_file_with_record_name(path, record_name)?;

    // Check if the key exists
    if !parse_result.translations.contains_key(key) {
        // Remove backup before returning error
        let _ = fs::remove_file(&backup_path);
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

    // Remove backup file after successful write
    let _ = fs::remove_file(&backup_path);

    Ok(())
}

fn remove_type_field(lines: &mut Vec<String>, key: &str) {
    // Find the line containing the type field
    let mut field_idx = None;
    let mut is_first_field = false;

    for (i, line) in lines.iter().enumerate() {
        if line.contains(&format!(" {} :", key)) {
            field_idx = Some(i);
            // Check if this is the first field (no leading comma)
            let trimmed = line.trim_start();
            is_first_field = !trimmed.starts_with(',');
            break;
        }
    }

    if let Some(idx) = field_idx {
        // Remove the field line
        lines.remove(idx);

        // If we removed the first field, we need to make the next field the first
        if is_first_field && idx < lines.len() {
            // Find the next field line (starts with comma)
            let mut next_field_idx = idx;
            while next_field_idx < lines.len() {
                let line = lines[next_field_idx].trim();
                if line.starts_with(',') {
                    // This is the next field - convert it to first field format
                    // Change ", fieldName : Type" to "  fieldName : Type"
                    let field_line = &lines[next_field_idx];
                    // Replace the leading ", " with "  " to maintain proper indentation
                    let new_line = field_line.replacen(", ", "  ", 1);
                    lines[next_field_idx] = new_line;
                    break;
                } else if line.starts_with('}') {
                    // No more fields
                    break;
                }
                // Skip comments and empty lines
                next_field_idx += 1;
            }
        }
    }
}

fn remove_record_field(lines: &mut Vec<String>, key: &str) {
    let mut field_start_idx = None;
    let mut comma_line_idx = None;
    let mut is_first_field = false;

    // Find the field - it might be preceded by a comma on the previous line
    for (i, line) in lines.iter().enumerate() {
        // Check if this line has a comma followed by our field on the next line
        if i + 1 < lines.len() && line.trim().ends_with(',') && lines[i + 1].contains(&format!("{} =", key)) {
            comma_line_idx = Some(i);
            field_start_idx = Some(i + 1);
            break;
        }
        // Check if this line starts with comma and our field
        if line.trim_start().starts_with(&format!(", {} =", key)) {
            field_start_idx = Some(i);
            break;
        }
        // Check if this line just has our field (first field in record)
        if line.contains(&format!("{} =", key)) && !line.trim_start().starts_with(',') {
            field_start_idx = Some(i);
            is_first_field = true;
            break;
        }
    }

    if let Some(start_idx) = field_start_idx {
        let mut lines_to_remove = vec![start_idx];

        // Check if it's a multi-line value (function or complex expression)
        let field_line = &lines[start_idx];
        let is_function = field_line.contains("\\") || field_line.contains("case") || field_line.contains("if ");
        let is_multiline = is_function || !field_line.trim().ends_with('"');

        if is_multiline {
            // Find the end of this field
            let mut j = start_idx + 1;
            let indent_level = count_leading_spaces(&lines[start_idx]);

            while j < lines.len() {
                let current_line = &lines[j];
                let current_indent = count_leading_spaces(current_line);
                let trimmed = current_line.trim();

                // Check if we've reached the next field at the same or lower indent level
                if !trimmed.is_empty() {
                    // Next field at same level (starts with comma or closing brace)
                    if current_indent <= indent_level && (trimmed.starts_with(',') || trimmed.starts_with('}')) {
                        break;
                    }
                    // For fields inside the record, check for field assignment at similar indent
                    if current_indent <= indent_level + 4 && trimmed.contains(" = ") && !trimmed.starts_with("case ") {
                        // This might be the next field if it's not inside a case expression
                        let before_eq = trimmed.split(" = ").next().unwrap_or("");
                        if before_eq.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            break;
                        }
                    }
                }

                lines_to_remove.push(j);
                j += 1;
            }
        }

        // Also remove the comma line if it exists and only contains a comma
        if let Some(comma_idx) = comma_line_idx {
            if lines[comma_idx].trim() == "," {
                lines_to_remove.insert(0, comma_idx);
            }
        }

        // Handle the case where we need to fix trailing commas
        // If we're removing the last field before }, we need to remove the comma from the previous field
        if start_idx > 0 && lines_to_remove.len() > 0 {
            let last_removed_idx = *lines_to_remove.last().unwrap();
            if last_removed_idx + 1 < lines.len() && lines[last_removed_idx + 1].trim().starts_with('}') {
                // Check if previous field ends with comma
                let prev_field_idx = start_idx - 1;
                if lines[prev_field_idx].trim().ends_with(',') {
                    // Remove the trailing comma
                    lines[prev_field_idx] = lines[prev_field_idx].trim_end().trim_end_matches(',').to_string();
                }
            }
        }

        // Remove lines in reverse order to maintain indices
        lines_to_remove.sort_by(|a, b| b.cmp(a));
        for &line_idx in lines_to_remove.iter() {
            lines.remove(line_idx);
        }

        // If we removed the first field, promote the next field to be first
        if is_first_field {
            // After removal, find the next field line (starts with comma)
            // The removed lines are gone, so we search from where the first field was
            let search_start = if start_idx >= lines_to_remove.len() {
                start_idx - lines_to_remove.len() + 1
            } else {
                0
            };

            for i in search_start..lines.len() {
                let line = lines[i].trim();
                if line.starts_with(',') {
                    // This is the next field - convert it to first field format
                    // Change ", fieldName = value" to "  fieldName = value"
                    let field_line = &lines[i];
                    let new_line = field_line.replacen(", ", "  ", 1);
                    lines[i] = new_line;
                    break;
                } else if line.starts_with('}') {
                    // No more fields
                    break;
                }
                // Skip comments and empty lines
            }
        }
    }
}

fn count_leading_spaces(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_remove_anonymous_function_field() {
        let temp_dir = TempDir::new().unwrap();
        let i18n_file = temp_dir.path().join("I18n.elm");
        
        // Create a test I18n file with anonymous functions
        let content = r#"module I18n exposing (..)

type alias Translations =
    { welcome : String
    , ticketPriority : Ticket.Priority -> String
    , ticketStatus : Ticket.Status -> String
    , goodbye : String
    }

translationsEn : Translations
translationsEn =
    { welcome = "Welcome"
    , ticketPriority =
        \priority ->
            case priority of
                Ticket.Low -> "Low"
                Ticket.Normal -> "Normal"
                Ticket.High -> "High"
                Ticket.Urgent -> "Urgent"
    , ticketStatus =
        \status ->
            case status of
                Ticket.Open -> "Open"
                Ticket.InProgress -> "In Progress"
                Ticket.Resolved -> "Resolved"
                Ticket.Closed -> "Closed"
    , goodbye = "Goodbye"
    }

translationsFr : Translations
translationsFr =
    { welcome = "Bienvenue"
    , ticketPriority =
        \priority ->
            case priority of
                Ticket.Low -> "Faible"
                Ticket.Normal -> "Normal"
                Ticket.High -> "Élevé"
                Ticket.Urgent -> "Urgent"
    , ticketStatus =
        \status ->
            case status of
                Ticket.Open -> "Ouvert"
                Ticket.InProgress -> "En cours"
                Ticket.Resolved -> "Résolu"
                Ticket.Closed -> "Fermé"
    , goodbye = "Au revoir"
    }
"#;
        
        fs::write(&i18n_file, content).unwrap();
        
        // Remove the ticketStatus field
        remove_translation(&i18n_file, "ticketStatus").unwrap();
        
        // Read the result
        let result = fs::read_to_string(&i18n_file).unwrap();
        
        // Verify ticketStatus is completely removed
        assert!(!result.contains("ticketStatus"));
        
        // Verify ticketPriority is intact and not corrupted
        assert!(result.contains("ticketPriority ="));
        assert!(result.contains(r#"Ticket.Low -> "Low""#));
        assert!(result.contains(r#"Ticket.Urgent -> "Urgent""#));
        
        // Verify the structure is still valid (no orphaned lambdas)
        assert!(!result.contains(r#"Ticket.Urgent -> "Urgent"
    \status ->"#));
        
        // Verify other fields are intact
        assert!(result.contains(r#"welcome = "Welcome""#));
        assert!(result.contains(r#"goodbye = "Goodbye""#));
    }
    
    #[test]
    fn test_remove_field_between_functions() {
        let temp_dir = TempDir::new().unwrap();
        let i18n_file = temp_dir.path().join("I18n.elm");
        
        // Create a test with a simple field between two function fields
        let content = r#"module I18n exposing (..)

type alias Translations =
    { funcA : Int -> String
    , simpleField : String
    , funcB : Bool -> String
    }

translationsEn : Translations
translationsEn =
    { funcA =
        \n ->
            if n > 0 then
                "Positive"
            else
                "Non-positive"
    , simpleField = "Simple"
    , funcB =
        \b ->
            if b then
                "True"
            else
                "False"
    }

translationsFr : Translations
translationsFr =
    { funcA =
        \n ->
            if n > 0 then
                "Positif"
            else
                "Non-positif"
    , simpleField = "Simple"
    , funcB =
        \b ->
            if b then
                "Vrai"
            else
                "Faux"
    }
"#;
        
        fs::write(&i18n_file, content).unwrap();
        
        // Remove the simple field
        remove_translation(&i18n_file, "simpleField").unwrap();
        
        let result = fs::read_to_string(&i18n_file).unwrap();
        
        // Verify simpleField is removed
        assert!(!result.contains("simpleField"));
        
        // Verify both functions are intact
        assert!(result.contains("funcA ="));
        assert!(result.contains(r#""Positive""#));
        assert!(result.contains("funcB ="));
        assert!(result.contains(r#""True""#));
    }
}