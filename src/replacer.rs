use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;
use crate::parser::parse_i18n_file_with_record_name;

#[derive(Debug, Clone)]
pub struct StringMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug)]
struct FunctionInfo {
    line_number: usize,
    has_translations_param: bool,
    type_signature_line: Option<usize>,
}

#[derive(Debug)]
struct FunctionCall {
    caller_function: String,
    called_function: String,
    line_number: usize,
}

/// Find all occurrences of exact strings in Elm files
pub fn find_string_occurrences(
    root_path: &Path,
    search_strings: &[&str],
) -> Result<Vec<StringMatch>> {
    let mut matches = Vec::new();
    
    
    // Create regex patterns for each string
    let patterns: Vec<Regex> = search_strings
        .iter()
        .map(|s| {
            // Escape special regex characters and create pattern for quoted strings
            let escaped = regex::escape(s);
            // Match the string when it appears as a complete string literal
            Regex::new(&format!(r#""{}""#, escaped)).unwrap()
        })
        .collect();
    
    // Walk through all Elm files
    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        let entry = entry?;
        let path = entry.path();
        
        
        // Only process .elm files
        if path.is_file() && path.extension().map_or(false, |ext| ext == "elm") {
            
            // Skip I18n.elm itself
            if path.file_name().map_or(false, |name| name == "I18n.elm") {
                continue;
            }
            
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            
            // Check each line
            for (line_idx, line) in content.lines().enumerate() {
                // Skip comments
                if line.trim_start().starts_with("--") {
                    continue;
                }
                
                // Check each pattern
                for (_pattern_idx, pattern) in patterns.iter().enumerate() {
                    
                    for mat in pattern.find_iter(line) {
                        matches.push(StringMatch {
                            file_path: path.to_path_buf(),
                            line_number: line_idx + 1,
                            line_content: line.to_string(),
                            start_col: mat.start(),
                            end_col: mat.end(),
                        });
                    }
                }
            }
        }
    }
    
    matches.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then(a.line_number.cmp(&b.line_number))
    });
    
    Ok(matches)
}

/// Replace string occurrences with translation keys and handle Translations parameter propagation
pub fn replace_strings(
    matches: &[StringMatch],
    key: &str,
    _i18n_module: &str,
) -> Result<()> {
    // Group matches by file
    let mut files_to_update: HashMap<PathBuf, Vec<&StringMatch>> = HashMap::new();
    
    for mat in matches {
        files_to_update
            .entry(mat.file_path.clone())
            .or_insert_with(Vec::new)
            .push(mat);
    }
    
    // Process each file
    for (file_path, file_matches) in files_to_update {
        let content = fs::read_to_string(&file_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        // First, analyze the file to find functions that will use t.key
        let functions_using_t = find_functions_using_translations(&lines, &file_matches, key);
        
        // Then, find all function information and calls
        let (function_infos, function_calls) = analyze_elm_file(&lines);
        
        // Determine which functions need the Translations parameter
        let functions_needing_t = propagate_translations_requirement(
            &functions_using_t,
            &function_infos,
            &function_calls,
        );
        
        // Apply all modifications
        apply_modifications(
            &mut lines,
            &file_matches,
            key,
            &function_infos,
            &function_calls,
            &functions_needing_t,
        )?;
        
        // Write back to file
        let new_content = lines.join("\n");
        fs::write(&file_path, new_content)?;
    }
    
    Ok(())
}

fn replace_string_in_line(line: &str, mat: &StringMatch, key: &str) -> String {
    let mut result = String::new();
    result.push_str(&line[..mat.start_col]);
    result.push_str(&format!("t.{}", key));
    result.push_str(&line[mat.end_col..]);
    result
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    // Don't filter root directory itself
    if entry.depth() == 0 {
        return false;
    }
    
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

/// Find functions that will use t.key after replacement
fn find_functions_using_translations(
    lines: &[String],
    matches: &[&StringMatch],
    _key: &str,
) -> HashSet<String> {
    let mut functions_using_t = HashSet::new();
    
    for mat in matches {
        // Find the function containing this string
        if let Some(func_name) = find_containing_function(lines, mat.line_number) {
            functions_using_t.insert(func_name);
        }
    }
    
    functions_using_t
}

/// Find which function contains a given line
fn find_containing_function(lines: &[String], line_number: usize) -> Option<String> {
    // Search backwards from the line to find the function definition
    for i in (0..line_number.min(lines.len())).rev() {
        let line = &lines[i];
        
        // Check if this is a function definition
        if let Some(func_name) = extract_function_name(line) {
            // Make sure we're not in a type signature
            if !line.contains(" : ") {
                return Some(func_name);
            }
        }
    }
    
    None
}

/// Extract function name from a line
fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    
    // Skip type signatures and comments
    if trimmed.contains(" : ") || trimmed.starts_with("--") || trimmed.starts_with("{-") {
        return None;
    }
    
    // Match function definitions like "functionName param1 param2 ="
    let func_regex = Regex::new(r"^([a-z][a-zA-Z0-9_]*)\s+.*=").unwrap();
    if let Some(captures) = func_regex.captures(trimmed) {
        return Some(captures[1].to_string());
    }
    
    None
}

/// Analyze an Elm file to find all functions and their calls
fn analyze_elm_file(lines: &[String]) -> (HashMap<String, FunctionInfo>, Vec<FunctionCall>) {
    let mut function_infos = HashMap::new();
    let mut function_calls = Vec::new();
    
    // First pass: collect all function information
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with("--") || trimmed.starts_with("{-") {
            continue;
        }
        
        // Check for function type signature
        if let Some(func_name) = extract_function_from_type_signature(trimmed) {
            // The next non-empty, non-comment line should be the implementation
            for j in (i + 1)..lines.len() {
                let next_line = lines[j].trim();
                if !next_line.is_empty() && !next_line.starts_with("--") {
                    if let Some(impl_name) = extract_function_name(&lines[j]) {
                        if impl_name == func_name {
                            let has_t = trimmed.contains("Translations ->");
                            function_infos.insert(
                                func_name.clone(),
                                FunctionInfo {
                                    line_number: j + 1,
                                    has_translations_param: has_t,
                                    type_signature_line: Some(i + 1),
                                },
                            );
                        }
                    }
                    break;
                }
            }
        }
        
        // Check for function definition without type signature
        if let Some(func_name) = extract_function_name(line) {
            if !function_infos.contains_key(&func_name) {
                let has_t = line.contains(" t ") || line.starts_with(&format!("{} t ", func_name));
                function_infos.insert(
                    func_name.clone(),
                    FunctionInfo {
                        line_number: i + 1,
                        has_translations_param: has_t,
                        type_signature_line: None,
                    },
                );
            }
        }
    }
    
    // Second pass: find function calls now that we know all functions
    let mut current_function: Option<String> = None;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip comments and type signatures
        if trimmed.starts_with("--") || trimmed.starts_with("{-") || trimmed.contains(" : ") {
            continue;
        }
        
        // Update current function context
        if let Some(func_name) = extract_function_name(line) {
            current_function = Some(func_name);
            continue; // Skip the function definition line
        }
        
        // Find function calls
        if let Some(ref caller) = current_function {
            // Skip lines that are part of let bindings or conditionals
            if is_let_binding(trimmed) || is_conditional_line(trimmed) {
                continue;
            }
            
            // Look for any known function names in this line
            for (func_name, _) in &function_infos {
                if func_name != caller && line.contains(func_name) {
                    // Verify it's actually a function call
                    if is_function_call(trimmed, func_name) {
                        function_calls.push(FunctionCall {
                            caller_function: caller.clone(),
                            called_function: func_name.clone(),
                            line_number: i + 1,
                        });
                    }
                }
            }
        }
    }
    
    (function_infos, function_calls)
}

/// Extract function name from type signature
fn extract_function_from_type_signature(line: &str) -> Option<String> {
    let type_sig_regex = Regex::new(r"^([a-z][a-zA-Z0-9_]*)\s*:").unwrap();
    type_sig_regex.captures(line).map(|c| c[1].to_string())
}

/// Check if a line is a let binding
fn is_let_binding(line: &str) -> bool {
    let trimmed = line.trim();
    // Match patterns like "name =", "name arg =", etc.
    // but not "in" or function calls
    if trimmed.starts_with("let ") || trimmed == "let" {
        return true;
    }
    
    // Look for simple variable bindings like "isSelected = ..."
    // This regex matches "identifier = " (with optional whitespace)
    let binding_regex = Regex::new(r"^\s*[a-z][a-zA-Z0-9_]*\s*=\s*").unwrap();
    if binding_regex.is_match(line) {
        // Make sure it's not a record update
        if !line.contains(" | ") && !line.contains("->") {
            return true;
        }
    }
    
    false
}

/// Check if a line contains conditional expressions
fn is_conditional_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("if ") || 
    trimmed.starts_with("then ") || 
    trimmed.starts_with("else ") ||
    trimmed.contains(" then") ||
    trimmed.contains(" else")
}

/// Check if an identifier in a line is actually a function call
fn is_function_call(line: &str, func_name: &str) -> bool {
    // Skip if it's in a let binding position
    if line.contains(&format!("{} =", func_name)) || line.contains(&format!("{} ", func_name)) && line.contains(" = ") {
        return false;
    }
    
    // Skip if it's in a conditional test
    if line.contains(&format!("if {}", func_name)) || line.contains(&format!("if {} ", func_name)) {
        return false;
    }
    
    // Skip if it's after "in" keyword
    if line.contains(&format!("in {}", func_name)) || line.contains(&format!("in {} ", func_name)) {
        return false;
    }
    
    // It's likely a function call if it appears in these contexts
    line.contains(&format!("[ {}", func_name)) ||      // In a list
    line.contains(&format!(", {}", func_name)) ||      // After a comma
    line.contains(&format!("( {}", func_name)) ||      // In parentheses
    line.contains(&format!(" {}", func_name)) ||       // After a space (general case)
    line.trim().starts_with(func_name)                 // At the start of a line
}

/// Propagate translations requirement through function call graph
fn propagate_translations_requirement(
    functions_directly_using_t: &HashSet<String>,
    function_infos: &HashMap<String, FunctionInfo>,
    function_calls: &[FunctionCall],
) -> HashSet<String> {
    let mut functions_needing_t = functions_directly_using_t.clone();
    let mut changed = true;
    
    // Keep propagating until no more changes
    while changed {
        changed = false;
        let current_set = functions_needing_t.clone();
        
        for call in function_calls {
            // If the called function needs t, the caller needs t too
            if current_set.contains(&call.called_function) && !current_set.contains(&call.caller_function) {
                // Only add if the function exists and doesn't already have t
                if let Some(info) = function_infos.get(&call.caller_function) {
                    if !info.has_translations_param {
                        functions_needing_t.insert(call.caller_function.clone());
                        changed = true;
                    }
                }
            }
        }
    }
    
    functions_needing_t
}

/// Apply all modifications to the file
fn apply_modifications(
    lines: &mut Vec<String>,
    matches: &[&StringMatch],
    key: &str,
    function_infos: &HashMap<String, FunctionInfo>,
    function_calls: &[FunctionCall],
    functions_needing_t: &HashSet<String>,
) -> Result<()> {
    // First, add Translations parameter to functions that need it
    let mut lines_to_modify: Vec<(usize, String)> = Vec::new();
    
    for func_name in functions_needing_t {
        if let Some(info) = function_infos.get(func_name) {
            if !info.has_translations_param {
                // Modify type signature if it exists
                if let Some(sig_line) = info.type_signature_line {
                    let sig_idx = sig_line - 1;
                    if sig_idx < lines.len() {
                        let new_sig = add_translations_to_signature(&lines[sig_idx]);
                        lines_to_modify.push((sig_idx, new_sig));
                    }
                }
                
                // Modify function implementation
                let impl_idx = info.line_number - 1;
                if impl_idx < lines.len() {
                    let new_impl = add_translations_to_implementation(&lines[impl_idx], func_name);
                    lines_to_modify.push((impl_idx, new_impl));
                }
            }
        }
    }
    
    // Then, update function calls to pass t parameter
    for call in function_calls {
        if functions_needing_t.contains(&call.called_function) {
            if let Some(called_info) = function_infos.get(&call.called_function) {
                if !called_info.has_translations_param {
                    let line_idx = call.line_number - 1;
                    if line_idx < lines.len() {
                        let new_line = add_t_to_function_call(&lines[line_idx], &call.called_function);
                        lines_to_modify.push((line_idx, new_line));
                    }
                }
            }
        }
    }
    
    // Sort modifications by line number in reverse order
    lines_to_modify.sort_by(|a, b| b.0.cmp(&a.0));
    
    // Apply line modifications
    for (idx, new_content) in lines_to_modify {
        if idx < lines.len() {
            lines[idx] = new_content;
        }
    }
    
    // Finally, replace the strings with t.key
    let mut sorted_matches = matches.to_vec();
    sorted_matches.sort_by(|a, b| {
        b.line_number
            .cmp(&a.line_number)
            .then(b.start_col.cmp(&a.start_col))
    });
    
    for mat in sorted_matches {
        let line_idx = mat.line_number - 1;
        if line_idx < lines.len() {
            let line = lines[line_idx].clone();
            let new_line = replace_string_in_line(&line, mat, key);
            lines[line_idx] = new_line;
        }
    }
    
    Ok(())
}

/// Add Translations parameter to type signature
fn add_translations_to_signature(signature: &str) -> String {
    if signature.contains(" : ") {
        let parts: Vec<&str> = signature.split(" : ").collect();
        if parts.len() == 2 {
            let func_name = parts[0];
            let type_part = parts[1];
            
            // Add Translations as the first parameter
            if type_part.contains(" -> ") {
                format!("{} : Translations -> {}", func_name, type_part)
            } else {
                // Single return type
                format!("{} : Translations -> {}", func_name, type_part)
            }
        } else {
            signature.to_string()
        }
    } else {
        signature.to_string()
    }
}

/// Add Translations parameter to function implementation
fn add_translations_to_implementation(implementation: &str, func_name: &str) -> String {
    // Find where the function name ends and parameters begin
    if let Some(name_end) = implementation.find(func_name) {
        let after_name = name_end + func_name.len();
        let before_name = &implementation[..name_end];
        let after_name_str = &implementation[after_name..];
        
        // Insert 't' as the first parameter
        if after_name_str.trim_start().starts_with('=') {
            // No parameters
            format!("{}{} t{}", before_name, func_name, after_name_str)
        } else {
            // Has parameters
            format!("{}{} t{}", before_name, func_name, after_name_str)
        }
    } else {
        implementation.to_string()
    }
}

/// Add t parameter to function calls
fn add_t_to_function_call(line: &str, func_name: &str) -> String {
    // Don't modify let bindings, conditionals, or in expressions
    if is_let_binding(line) || is_conditional_line(line) {
        return line.to_string();
    }
    
    // Skip if it's after "in" keyword
    if line.contains(&format!("in {}", func_name)) || line.contains(&format!("in {} ", func_name)) {
        return line.to_string();
    }
    
    // Skip if it's a let binding
    if line.contains(&format!("{} =", func_name)) {
        return line.to_string();
    }
    
    // Skip if it's in a conditional test
    if line.contains(&format!("if {}", func_name)) || line.contains(&format!("if {} ", func_name)) {
        return line.to_string();
    }
    
    // Only process actual function calls
    if !is_function_call(line, func_name) {
        return line.to_string();
    }
    
    // Handle function calls with existing arguments
    let pattern_with_args = format!(r"\b({})\s+([a-zA-Z])", regex::escape(func_name));
    if let Ok(re) = Regex::new(&pattern_with_args) {
        if re.is_match(line) {
            // Check if 't' is already the first argument
            if !line.contains(&format!("{} t ", func_name)) && !line.contains(&format!("{} t)", func_name)) {
                return re.replace(line, format!("$1 t $2")).to_string();
            }
        }
    }
    
    // Handle function calls without arguments
    // Be more specific about where we add 't'
    let patterns = vec![
        // Pattern for [ functionName at end of line (no closing bracket on same line)
        (format!(r"\[\s*{}\s*$", regex::escape(func_name)), format!("[ {} t", func_name)),
        // Pattern for , functionName ] 
        (format!(r",\s*{}\s*\]", regex::escape(func_name)), format!(", {} t ]", func_name)),
        // Other patterns
        (format!(r"\[\s*({})\s*\]", regex::escape(func_name)), format!("[ {} t ]", func_name)),
        (format!(r"\[\s*({})\s*,", regex::escape(func_name)), format!("[ {} t,", func_name)),
        (format!(r",\s*({})\s*,", regex::escape(func_name)), format!(", {} t,", func_name)),
        (format!(r"\(\s*({})\s*\)", regex::escape(func_name)), format!("( {} t )", func_name)),
    ];
    
    for (pattern, replacement) in patterns {
        if let Ok(re) = Regex::new(&pattern) {
            if re.is_match(line) {
                return re.replace(line, replacement.as_str()).to_string();
            }
        }
    }
    
    // If it's a standalone function call at the start or elsewhere
    let standalone_pattern = format!(r"^(\s*)({})\s*$", regex::escape(func_name));
    if let Ok(re) = Regex::new(&standalone_pattern) {
        if re.is_match(line) {
            return re.replace(line, format!("$1$2 t")).to_string();
        }
    }
    
    line.to_string()
}

/// Find all translation keys that are not used in the codebase
pub fn find_unused_keys(i18n_file: &Path, src_dir: &Path, record_name: &str) -> Result<Vec<String>> {
    // Parse the I18n file to get all translation keys
    let parse_result = parse_i18n_file_with_record_name(i18n_file, record_name)?;
    let all_keys: HashSet<String> = parse_result.translations.keys().cloned().collect();

    // Find all uses of translation keys in the codebase
    let mut used_keys = HashSet::new();

    // Walk through all Elm files
    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        let entry = entry?;
        let path = entry.path();

        // Only process .elm files
        if path.is_file() && path.extension().map_or(false, |ext| ext == "elm") {
            // Skip I18n files themselves
            if path.file_name().map_or(false, |name| {
                let name_str = name.to_string_lossy();
                name_str == "I18n.elm"
                    || name_str == "App.elm"
                    || name_str == "LandingPage.elm"
                    || name_str == "ComingSoon.elm"
                    || name_str == "Email.elm"
                    || name_str == "Errors.elm"
            }) && path.parent().map_or(false, |p| {
                p.file_name().map_or(false, |pname| pname == "I18n")
            }) {
                continue;
            }

            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            // Look for patterns like <var>.<key> where <var> is a short identifier
            // This catches t.key, tlp.key, tcs.key, translations.key, etc.
            // Pattern: word boundary, 1-12 char identifier, dot, then the field name
            let field_access_pattern = Regex::new(r"\b[a-zA-Z][a-zA-Z0-9_]{0,11}\.([a-zA-Z][a-zA-Z0-9_]*)\b").unwrap();

            // Find all matches
            for captures in field_access_pattern.captures_iter(&content) {
                if let Some(key) = captures.get(1) {
                    used_keys.insert(key.as_str().to_string());
                }
            }
        }
    }

    // Find unused keys
    let unused_keys: Vec<String> = all_keys
        .difference(&used_keys)
        .cloned()
        .collect::<Vec<_>>();

    // Sort for consistent output
    let mut unused_keys = unused_keys;
    unused_keys.sort();

    Ok(unused_keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_find_string_occurrences() {
        let temp_dir = TempDir::new().unwrap();
        let elm_file = temp_dir.path().join("Test.elm");
        
        fs::write(
            &elm_file,
            r#"module Test exposing (..)

view model =
    div []
        [ text "You are welcome"
        , text "Hello"
        , text "You are welcome"
        ]
"#,
        )
        .unwrap();
        
        // Verify file was created correctly
        assert!(elm_file.exists(), "Test file should exist");
        let content = fs::read_to_string(&elm_file).unwrap();
        assert!(content.contains("You are welcome"), "File should contain search string");
        
        
        // Now test the function
        let matches = find_string_occurrences(temp_dir.path(), &["You are welcome"]).unwrap();
        
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_number, 5);
        assert_eq!(matches[1].line_number, 7);
    }
    
    #[test]
    fn test_replace_string_in_line() {
        let line = r#"        [ text "You are welcome""#;
        let mat = StringMatch {
            file_path: PathBuf::from("test.elm"),
            line_number: 1,
            line_content: line.to_string(),
            start_col: 15,
            end_col: 32,  // Include the closing quote
        };
        
        let result = replace_string_in_line(line, &mat, "youAreWelcome");
        assert_eq!(result, r#"        [ text t.youAreWelcome"#);
    }
    
    #[test]
    fn test_is_let_binding() {
        assert!(is_let_binding("let"));
        assert!(is_let_binding("let "));
        assert!(is_let_binding("let x = 5"));
        assert!(is_let_binding("    isSelected = form.category == Just category"));
        assert!(is_let_binding("selectedClass = \"some-class\""));
        
        assert!(!is_let_binding("if isSelected then"));
        assert!(!is_let_binding("in t"));
        assert!(!is_let_binding("{ model | field = value }"));
        assert!(!is_let_binding(", field = value"));
    }
    
    #[test]
    fn test_is_conditional_line() {
        assert!(is_conditional_line("if isSelected then"));
        assert!(is_conditional_line("    if form.isValid then"));
        assert!(is_conditional_line("else if x > 0 then"));
        assert!(is_conditional_line("    then doSomething"));
        assert!(is_conditional_line("    else doSomethingElse"));
        
        assert!(!is_conditional_line("isSelected = True"));
        assert!(!is_conditional_line("[ header"));
    }
    
    #[test]
    fn test_is_function_call() {
        assert!(is_function_call("[ welcomeMessage", "welcomeMessage"));
        assert!(is_function_call(", userInfo model", "userInfo"));
        assert!(is_function_call("div [] [ header", "header"));
        assert!(is_function_call("    mainContent model", "mainContent"));
        
        assert!(!is_function_call("isSelected = form.category", "isSelected"));
        assert!(!is_function_call("if isSelected then", "isSelected"));
        assert!(!is_function_call("in t", "t"));
        assert!(!is_function_call("let isSelected = True", "isSelected"));
    }
    
    #[test]
    fn test_add_t_to_function_call_avoids_let_bindings() {
        // Should not modify let bindings
        let line = "    isSelected = form.category == Just category";
        let result = add_t_to_function_call(line, "isSelected");
        assert_eq!(result, line);
        
        // Should not modify after in keyword
        let line = "in t";
        let result = add_t_to_function_call(line, "t");
        assert_eq!(result, line);
    }
    
    #[test]
    fn test_add_t_to_function_call_avoids_conditionals() {
        // Should not modify conditional tests
        let line = "if isSelected then";
        let result = add_t_to_function_call(line, "isSelected");
        assert_eq!(result, line);
        
        let line = "    if isSelected form then";
        let result = add_t_to_function_call(line, "isSelected");
        assert_eq!(result, line);
    }
    
    #[test]
    fn test_add_t_to_function_call_handles_lists() {
        // Should add t to function calls in lists
        let line = "        [ welcomeMessage";
        let result = add_t_to_function_call(line, "welcomeMessage");
        assert_eq!(result, "        [ welcomeMessage t");
        
        let line = "        , header ]";
        let result = add_t_to_function_call(line, "header");
        assert_eq!(result, "        , header t ]");
    }
    
    #[test]
    fn test_find_unused_keys() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test I18n.elm file with some keys
        let i18n_file = temp_dir.path().join("I18n.elm");
        fs::write(
            &i18n_file,
            r#"module I18n exposing (..)

type alias Translations =
    { welcome : String
    , goodbye : String
    , unused : String
    , alsoUnused : String
    }

translationsEn : Translations
translationsEn =
    { welcome = "Welcome"
    , goodbye = "Goodbye"
    , unused = "Not used"
    , alsoUnused = "Also not used"
    }

translationsFr : Translations
translationsFr =
    { welcome = "Bienvenue"
    , goodbye = "Au revoir"
    , unused = "Pas utilisé"
    , alsoUnused = "Aussi pas utilisé"
    }
"#,
        )
        .unwrap();
        
        // Create a test Elm file that uses some keys
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        
        let elm_file = src_dir.join("Main.elm");
        fs::write(
            &elm_file,
            r#"module Main exposing (..)

import I18n exposing (Translations)

view : Translations -> Html msg
view t =
    div []
        [ h1 [] [ text t.welcome ]
        , p [] [ text t.goodbye ]
        ]
"#,
        )
        .unwrap();
        
        // Find unused keys
        let unused = find_unused_keys(&i18n_file, &src_dir, "Translations").unwrap();
        
        assert_eq!(unused.len(), 2);
        assert!(unused.contains(&"alsoUnused".to_string()));
        assert!(unused.contains(&"unused".to_string()));
    }
}