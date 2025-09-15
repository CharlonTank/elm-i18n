use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

mod generator;
mod parser;
mod replacer;
mod templates;
mod types;

use crate::generator::{add_translation, create_i18n_file, remove_translation};
use crate::parser::{check_key_exists, parse_i18n_file};
use crate::replacer::{find_string_occurrences, replace_strings, find_unused_keys};
use crate::templates::get_i18n_template;
use crate::types::Translation;

// Elm reserved words
const ELM_RESERVED_WORDS: &[&str] = &[
    "if", "then", "else", "case", "of", "let", "in", "type", "module", "where",
    "import", "exposing", "as", "port", "effect", "command", "subscription",
    "alias", "infixl", "infixr", "infix"
];

#[derive(Parser)]
#[command(name = "elm-i18n")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "CLI tool for managing Elm I18n translations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a simple translation
    Add {
        /// The translation key
        key: String,
        
        /// French translation
        #[arg(long)]
        fr: String,
        
        /// English translation
        #[arg(long)]
        en: String,
        
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
        
        /// Replace hardcoded strings in source files
        #[arg(long)]
        replace: bool,
        
        /// Root directory to search for replacements (defaults to src/)
        #[arg(long, default_value = "src")]
        src_dir: PathBuf,
        
    },
    
    /// Add a function translation
    #[command(name = "add-fn")]
    AddFunction {
        /// The function key
        key: String,
        
        /// Type signature (e.g., "Int -> String")
        #[arg(long)]
        type_sig: String,
        
        /// English implementation
        #[arg(long)]
        en: String,
        
        /// French implementation
        #[arg(long)]
        fr: String,
        
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },
    
    /// Check if a translation key exists
    Check {
        /// The translation key to check
        key: String,
        
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },
    
    /// Initialize a new I18n.elm file
    Init {
        /// Languages to support (comma-separated, defaults to "en,fr")
        #[arg(long, default_value = "en,fr")]
        languages: String,
        
        /// Path where to create I18n.elm (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },
    
    /// Remove a translation
    Remove {
        /// The translation key to remove
        key: String,
        
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },
    
    /// Remove all unused translations
    RemoveUnused {
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
        
        /// Root directory to search for usage (defaults to src/)
        #[arg(long, default_value = "src")]
        src_dir: PathBuf,
        
        /// Actually remove the unused keys (without this flag, just shows what would be removed)
        #[arg(long)]
        confirm: bool,
    },
    
    /// List all translations
    List {
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
        
        /// Show full translation values
        #[arg(long)]
        verbose: bool,
        
        /// Filter keys by pattern
        #[arg(long)]
        filter: Option<String>,
    },
    
    /// Show version information
    Version,
}

/// Validates and cleans a translation key
fn validate_and_clean_key(key: &str) -> Result<String> {
    // Check for forbidden characters
    if key.contains('.') {
        eprintln!("{} Error: Translation keys cannot contain dots (.)", "✗".red());
        eprintln!("{} The dot character is reserved for accessing nested translations (e.g., t.welcome)", "ℹ".blue());
        eprintln!("{} Please use camelCase or underscores instead", "ℹ".blue());
        std::process::exit(1);
    }
    
    // Handle reserved words
    let mut cleaned_key = key.to_string();
    if ELM_RESERVED_WORDS.contains(&key) {
        cleaned_key = format!("{}_", key);
        println!("{} Warning: '{}' is a reserved word in Elm, using '{}' instead", 
            "⚠".yellow(), 
            key.yellow(), 
            cleaned_key.green()
        );
    }
    
    // Validate key format (alphanumeric + underscores, starting with letter)
    if !cleaned_key.chars().next().unwrap_or('0').is_alphabetic() {
        eprintln!("{} Error: Translation keys must start with a letter", "✗".red());
        std::process::exit(1);
    }
    
    if !cleaned_key.chars().all(|c| c.is_alphanumeric() || c == '_') {
        eprintln!("{} Error: Translation keys can only contain letters, numbers, and underscores", "✗".red());
        std::process::exit(1);
    }
    
    Ok(cleaned_key)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Add { key, fr, en, file, replace, src_dir } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            handle_add(&file, &cleaned_key, &fr, &en, false, None, replace, &src_dir)?;
        }
        
        Commands::AddFunction { key, type_sig, en, fr, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            handle_add(&file, &cleaned_key, &fr, &en, true, Some(type_sig), false, &PathBuf::from("src"))?;
        }
        
        Commands::Check { key, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            handle_check(&file, &cleaned_key)?;
        }
        
        Commands::Init { languages, file } => {
            handle_init(&file, &languages)?;
        }
        
        Commands::Remove { key, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            handle_remove(&file, &cleaned_key)?;
        }
        
        Commands::RemoveUnused { file, src_dir, confirm } => {
            handle_remove_unused(&file, &src_dir, confirm)?;
        }
        
        Commands::List { file, verbose, filter } => {
            handle_list(&file, verbose, &filter)?;
        }
        
        Commands::Version => {
            println!("elm-i18n v{}", env!("CARGO_PKG_VERSION"));
            println!("CLI tool for managing Elm I18n translations");
            println!();
            println!("New in v0.4.0:");
            println!("  • Added 'list' command to view all translations");
            println!("  • Support for --verbose to see full translation values");
            println!("  • Filter translations with --filter option");
            println!();
            println!("New in v0.3.2:");
            println!("  • Fixed bug in remove_type_field that was corrupting field names");
            println!("  • Corrected the retain logic to properly remove type annotations");
            println!();
            println!("New in v0.3.1:");
            println!("  • Fixed critical bug in remove-unused where anonymous function fields");
            println!("    were not properly removed, causing syntax errors");
            println!("  • Improved multi-line field detection and removal");
            println!();
            println!("New in v0.3.0:");
            println!("  • Added 'remove-unused' command to find and remove unused translation keys");
            println!("  • Use --confirm flag to actually remove the keys");
            println!();
            println!("New in v0.2.1:");
            println!("  • Fixed bug where let bindings and conditionals were incorrectly modified");
            println!("  • Improved context-aware string replacement");
            println!("  • Better detection of actual function calls vs other identifiers");
            println!();
            println!("New in v0.2.0:");
            println!("  • Validation for forbidden characters (dots) in translation keys");
            println!("  • Automatic handling of Elm reserved words (e.g., 'type' → 'type_')");
            println!("  • Smart Translations parameter propagation when replacing strings");
        }
    }
    
    Ok(())
}

fn handle_add(
    file: &PathBuf,
    key: &str,
    fr: &str,
    en: &str,
    is_function: bool,
    type_sig: Option<String>,
    replace: bool,
    src_dir: &PathBuf,
) -> Result<()> {
    // Check if file exists
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        eprintln!("{} Run 'elm-i18n init' to create a new I18n.elm file", "ℹ".blue());
        std::process::exit(1);
    }
    
    // Check if key already exists
    match check_key_exists(file, key)? {
        Some(existing) => {
            println!("{} Translation '{}' already exists:", "ℹ".blue(), key.yellow());
            println!("  {}: {}", "EN".green(), existing.en);
            println!("  {}: {}", "FR".green(), existing.fr);
            println!();
            println!("The existing translations might be sufficient. Consider using a different key.");
        }
        None => {
            // Add the translation
            let translation = Translation {
                key: key.to_string(),
                en: en.to_string(),
                fr: fr.to_string(),
                is_function,
                type_signature: type_sig,
            };
            
            add_translation(file, &translation)?;
            
            println!("{} Added translation '{}' to {}", 
                "✓".green(), 
                key.yellow(), 
                file.display()
            );
            
            if !is_function {
                println!("  {}: {}", "EN".green(), en);
                println!("  {}: {}", "FR".green(), fr);
            }
            
            // Handle string replacement if requested
            if replace && !is_function {
                println!();
                println!("{} Searching for hardcoded strings to replace...", "🔍".blue());
                
                let search_strings = vec![en, fr];
                let matches = find_string_occurrences(src_dir, &search_strings)?;
                
                if matches.is_empty() {
                    println!("{} No hardcoded strings found to replace", "ℹ".blue());
                } else {
                    // Group matches by string value
                    let en_matches: Vec<_> = matches.iter()
                        .filter(|m| m.line_content.contains(&format!(r#""{}""#, en)))
                        .collect();
                    let fr_matches: Vec<_> = matches.iter()
                        .filter(|m| m.line_content.contains(&format!(r#""{}""#, fr)))
                        .collect();
                    
                    // Show what will be replaced
                    if !en_matches.is_empty() {
                        println!();
                        println!("{} Found {} occurrences of \"{}\":", "✓".green(), en_matches.len(), en);
                        for (_i, mat) in en_matches.iter().take(3).enumerate() {
                            println!("  {}:{}:", 
                                mat.file_path.display(), 
                                mat.line_number
                            );
                            println!("    {}", mat.line_content.trim());
                        }
                        if en_matches.len() > 3 {
                            println!("  ... and {} more", en_matches.len() - 3);
                        }
                    }
                    
                    if !fr_matches.is_empty() {
                        println!();
                        println!("{} Found {} occurrences of \"{}\":", "✓".green(), fr_matches.len(), fr);
                        for (_i, mat) in fr_matches.iter().take(3).enumerate() {
                            println!("  {}:{}:", 
                                mat.file_path.display(), 
                                mat.line_number
                            );
                            println!("    {}", mat.line_content.trim());
                        }
                        if fr_matches.len() > 3 {
                            println!("  ... and {} more", fr_matches.len() - 3);
                        }
                    }
                    
                    // Perform replacements
                    println!();
                    println!("{} Replacing strings with t.{}...", "🔄".blue(), key);
                    replace_strings(&matches, key, "I18n")?;
                    
                    println!("{} Replaced {} occurrences across {} file(s)", 
                        "✓".green(), 
                        matches.len(),
                        {
                            let unique_files: std::collections::HashSet<_> = 
                                matches.iter().map(|m| &m.file_path).collect();
                            unique_files.len()
                        }
                    );
                }
            }
        }
    }
    
    Ok(())
}

fn handle_check(file: &PathBuf, key: &str) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    match check_key_exists(file, key)? {
        Some(translation) => {
            println!("{} Translation '{}' exists:", "✓".green(), key.yellow());
            println!("  {}: {}", "EN".green(), translation.en);
            println!("  {}: {}", "FR".green(), translation.fr);
            
            if translation.is_function {
                if let Some(type_sig) = translation.type_signature {
                    println!("  {}: {}", "Type".cyan(), type_sig);
                }
            }
        }
        None => {
            println!("{} Translation '{}' not found", "✗".red(), key.yellow());
        }
    }
    
    Ok(())
}

fn handle_init(file: &PathBuf, languages: &str) -> Result<()> {
    if file.exists() {
        eprintln!("{} File already exists: {}", "✗".red(), file.display());
        eprintln!("Remove it first if you want to reinitialize.");
        std::process::exit(1);
    }
    
    let langs: Vec<String> = languages
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
    
    let template = get_i18n_template(&langs);
    create_i18n_file(file, &template)?;
    
    println!("{} Created {} with basic structure", "✓".green(), file.display());
    println!("Languages: {}", langs.join(", "));
    
    Ok(())
}

fn handle_remove(file: &PathBuf, key: &str) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    // Check if key exists first
    match check_key_exists(file, key)? {
        Some(translation) => {
            // Show what will be removed
            println!("{} Removing translation '{}':", "ℹ".blue(), key.yellow());
            println!("  {}: {}", "EN".green(), translation.en);
            println!("  {}: {}", "FR".green(), translation.fr);
            println!();
            
            // Remove the translation
            match remove_translation(file, key) {
                Ok(_) => {
                    println!("{} Removed translation '{}' from {}", 
                        "✓".green(), 
                        key.yellow(), 
                        file.display()
                    );
                }
                Err(e) => {
                    eprintln!("{} Failed to remove translation: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            println!("{} Translation '{}' not found", "✗".red(), key.yellow());
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn handle_remove_unused(file: &PathBuf, src_dir: &PathBuf, confirm: bool) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    println!("{} Scanning for unused translation keys...", "🔍".blue());
    
    // Find all unused keys
    let unused_keys = find_unused_keys(file, src_dir)?;
    
    if unused_keys.is_empty() {
        println!("{} All translation keys are in use!", "✓".green());
        return Ok(());
    }
    
    // Show unused keys
    println!();
    println!("{} Found {} unused translation keys:", "⚠".yellow(), unused_keys.len());
    for key in &unused_keys {
        println!("  • {}", key.yellow());
    }
    
    if !confirm {
        println!();
        println!("{} To remove these keys, run with --confirm flag:", "ℹ".blue());
        println!("  elm-i18n remove-unused --confirm");
        return Ok(());
    }
    
    // Remove the unused keys
    println!();
    println!("{} Removing unused keys...", "🗑".red());
    
    for key in &unused_keys {
        match remove_translation(file, key) {
            Ok(_) => {
                println!("  {} Removed: {}", "✓".green(), key);
            }
            Err(e) => {
                eprintln!("  {} Failed to remove {}: {}", "✗".red(), key, e);
            }
        }
    }
    
    println!();
    println!("{} Removed {} unused translation keys", "✓".green(), unused_keys.len());
    
    Ok(())
}

fn handle_list(file: &PathBuf, verbose: bool, filter: &Option<String>) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    // Parse the I18n file
    let parse_result = parse_i18n_file(file)?;
    let mut translations: Vec<_> = parse_result.translations.into_iter().collect();
    
    // Apply filter if provided
    if let Some(pattern) = filter {
        let pattern_lower = pattern.to_lowercase();
        translations.retain(|(key, _)| key.to_lowercase().contains(&pattern_lower));
    }
    
    // Sort by key
    translations.sort_by(|a, b| a.0.cmp(&b.0));
    
    if translations.is_empty() {
        if filter.is_some() {
            println!("{} No translations found matching '{}'", "✗".red(), filter.as_ref().unwrap().yellow());
        } else {
            println!("{} No translations found", "✗".red());
        }
        return Ok(());
    }
    
    // Display results
    println!("{} Found {} translation{}:", 
        "📋".blue(), 
        translations.len(),
        if translations.len() == 1 { "" } else { "s" }
    );
    
    if verbose {
        println!();
        for (key, translation) in &translations {
            println!("  {} {}", "•".green(), key.yellow());
            
            // Show type if it's a function
            if translation.is_function {
                if let Some(ref type_sig) = translation.type_signature {
                    println!("    {}: {}", "Type".cyan(), type_sig);
                }
            }
            
            // Show translations
            println!("    {}: {}", "EN".green(), 
                if translation.en.contains('\n') {
                    format!("\n{}", translation.en.lines()
                        .map(|line| format!("      {}", line))
                        .collect::<Vec<_>>()
                        .join("\n"))
                } else {
                    translation.en.clone()
                }
            );
            
            println!("    {}: {}", "FR".green(),
                if translation.fr.contains('\n') {
                    format!("\n{}", translation.fr.lines()
                        .map(|line| format!("      {}", line))
                        .collect::<Vec<_>>()
                        .join("\n"))
                } else {
                    translation.fr.clone()
                }
            );
            
            println!();
        }
    } else {
        // Simple list
        for (key, translation) in &translations {
            let type_info = if translation.is_function {
                format!(" ({})", 
                    translation.type_signature.as_ref()
                        .unwrap_or(&"Function".to_string())
                        .cyan()
                )
            } else {
                " (String)".cyan().to_string()
            };
            
            println!("  {} {}{}", "•".green(), key.yellow(), type_info);
        }
    }
    
    Ok(())
}