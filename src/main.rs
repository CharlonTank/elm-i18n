use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

mod generator;
mod parser;
mod templates;
mod types;

use crate::generator::{add_translation, create_i18n_file, remove_translation};
use crate::parser::check_key_exists;
use crate::templates::get_i18n_template;
use crate::types::Translation;

#[derive(Parser)]
#[command(name = "elm-i18n")]
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Add { key, fr, en, file } => {
            handle_add(&file, &key, &fr, &en, false, None)?;
        }
        
        Commands::AddFunction { key, type_sig, en, fr, file } => {
            handle_add(&file, &key, &fr, &en, true, Some(type_sig))?;
        }
        
        Commands::Check { key, file } => {
            handle_check(&file, &key)?;
        }
        
        Commands::Init { languages, file } => {
            handle_init(&file, &languages)?;
        }
        
        Commands::Remove { key, file } => {
            handle_remove(&file, &key)?;
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