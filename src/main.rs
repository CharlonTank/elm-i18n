use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::io::{self, Write};

mod config;
mod generator;
mod parser;
mod replacer;
mod templates;
mod types;

use crate::config::{Config, FileConfig, config_exists, prompt_setup_message};
use crate::generator::{add_translation_with_record_name, create_i18n_file, remove_translation_with_record_name};
use crate::parser::{check_key_exists_with_record_name, parse_i18n_file_with_record_name};
use crate::replacer::{find_string_occurrences, replace_strings, find_unused_keys};
use crate::templates::get_i18n_template_with_record_name;
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
    /// File shortcut for multi-file mode (e.g., --app, --landing)
    #[arg(long, global = true)]
    target: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup elm-i18n configuration
    Setup,
    
    /// Show current configuration status
    Status,
    
    /// Setup or update CLAUDE.md with elm-i18n instructions
    SetupClaude,
    
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
    
    // Handle commands that don't need config
    match &cli.command {
        Commands::Setup => return handle_setup(),
        Commands::Version => return handle_version(),
        Commands::Status => return handle_status(),
        Commands::SetupClaude => return handle_setup_claude(),
        _ => {}
    }
    
    // Load config for all other commands
    let config = match Config::load()? {
        Some(config) => config,
        None => {
            prompt_setup_message();
            std::process::exit(1);
        }
    };
    
    // Determine target file based on config and shortcut
    let (file_path, record_name) = determine_target_file(&config, &cli.target, &cli.command)?;
    
    match cli.command {
        Commands::Setup => unreachable!(),
        
        Commands::Add { key, fr, en, file, replace, src_dir } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                // Use config-determined file if default was not overridden
                file_path.clone()
            } else {
                file
            };
            let actual_src_dir = if src_dir.to_str() == Some("src") {
                config.source_dir().clone()
            } else {
                src_dir
            };
            handle_add(&actual_file, &cleaned_key, &fr, &en, false, None, replace, &actual_src_dir, &record_name)?;
        }
        
        Commands::AddFunction { key, type_sig, en, fr, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_add(&actual_file, &cleaned_key, &fr, &en, true, Some(type_sig), false, config.source_dir(), &record_name)?;
        }
        
        Commands::Check { key, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_check(&actual_file, &cleaned_key, &record_name)?;
        }
        
        Commands::Init { languages, file } => {
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_init(&actual_file, &languages, &record_name)?;
        }
        
        Commands::Remove { key, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_remove(&actual_file, &cleaned_key, &record_name)?;
        }
        
        Commands::RemoveUnused { file, src_dir, confirm } => {
            let actual_src_dir = if src_dir.to_str() == Some("src") {
                config.source_dir().clone()
            } else {
                src_dir
            };

            // In multi-file mode without a target, process all files
            if cli.target.is_none() {
                if let Config::MultiFile { files, .. } = &config {
                    println!("{} Running remove-unused on all translation files...\n", "🔍".blue());
                    for (shortcut, file_config) in files {
                        println!("{} Processing {} ({})...", "→".cyan(), shortcut.yellow(), file_config.path.display());
                        handle_remove_unused(&file_config.path, &actual_src_dir, confirm, &file_config.record_name)?;
                        println!();
                    }
                } else {
                    // Single file mode
                    handle_remove_unused(&file_path, &actual_src_dir, confirm, &record_name)?;
                }
            } else {
                // Target was specified, use the determined file
                let actual_file = if file.to_str() == Some("src/I18n.elm") {
                    file_path.clone()
                } else {
                    file
                };
                handle_remove_unused(&actual_file, &actual_src_dir, confirm, &record_name)?;
            }
        }
        
        Commands::List { file, verbose, filter } => {
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_list(&actual_file, verbose, &filter, &record_name)?
        }
        
        Commands::Version => unreachable!(),
        Commands::Status => unreachable!(),
        Commands::SetupClaude => unreachable!()
    }
    
    Ok(())
}

/// Determine which file to target based on config and shortcut
fn determine_target_file(config: &Config, shortcut: &Option<String>, command: &Commands) -> Result<(PathBuf, String)> {
    // For Init command, we might allow creation of new files
    let is_init = matches!(command, Commands::Init { .. });
    // RemoveUnused can work without a target (processes all files)
    let is_remove_unused = matches!(command, Commands::RemoveUnused { .. });

    match config {
        Config::SingleFile { file, record_name, .. } => {
            if shortcut.is_some() {
                eprintln!("{} Warning: File shortcuts are ignored in single-file mode", "⚠".yellow());
            }
            Ok((file.clone(), record_name.clone()))
        }
        Config::MultiFile { files, .. } => {
            match shortcut {
                Some(s) => {
                    match files.get(s) {
                        Some(file_config) => Ok((file_config.path.clone(), file_config.record_name.clone())),
                        None => {
                            eprintln!("{} Unknown file shortcut: {}", "✗".red(), s.yellow());
                            config.print_shortcuts();
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // RemoveUnused can run without a target - it will process all files
                    if is_remove_unused {
                        // Return dummy values - the command handler will iterate all files
                        Ok((PathBuf::from(""), String::new()))
                    } else if !is_init {
                        config.print_shortcuts();
                        std::process::exit(1);
                    } else {
                        // For init, we might allow specifying a new file
                        eprintln!("{} Multi-file mode requires a file shortcut", "✗".red());
                        config.print_shortcuts();
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

/// Handle the setup-claude command
fn handle_setup_claude() -> Result<()> {
    use std::fs;
    
    println!("{} Setting up CLAUDE.md with elm-i18n instructions...", "🤖".blue());
    println!();
    
    // Load configuration to understand project setup
    let config = match Config::load()? {
        Some(config) => config,
        None => {
            eprintln!("{} No elm-i18n.json configuration found!", "✗".red());
            eprintln!("Run {} first to create a configuration.", "elm-i18n setup".green());
            std::process::exit(1);
        }
    };
    
    // Check if CLAUDE.md already exists
    let claude_path = PathBuf::from("CLAUDE.md");
    let existing_content = if claude_path.exists() {
        fs::read_to_string(&claude_path).ok()
    } else {
        None
    };
    
    // Generate elm-i18n specific instructions
    let elm_i18n_section = generate_claude_instructions(&config);
    
    // Track if we're updating or creating
    let is_update = existing_content.is_some();
    
    // Merge or create CLAUDE.md
    let final_content = if let Some(existing) = existing_content {
        // Check if elm-i18n section already exists
        if existing.contains("## elm-i18n Configuration") {
            // Replace existing elm-i18n section
            let before_section = existing.split("## elm-i18n Configuration").next().unwrap_or("");
            let after_section = existing.split("## elm-i18n Configuration")
                .nth(1)
                .and_then(|s| s.split("\n## ").nth(1))
                .map(|s| format!("\n## {}", s))
                .unwrap_or_default();
            
            format!("{}{}{}", before_section, elm_i18n_section, after_section)
        } else {
            // Append elm-i18n section
            format!("{}\n\n{}", existing.trim(), elm_i18n_section)
        }
    } else {
        // Create new CLAUDE.md with elm-i18n instructions
        format!("# Project-Specific Instructions for Claude\n\n{}", elm_i18n_section)
    };
    
    // Write the file
    fs::write(&claude_path, final_content)?;
    
    println!("{} CLAUDE.md has been {}", 
        "✓".green(),
        if is_update { "updated" } else { "created" }
    );
    
    println!();
    println!("The file contains:");
    println!("  • elm-i18n configuration details");
    println!("  • Available translation files and shortcuts");
    println!("  • Example commands for your specific setup");
    println!();
    println!("Claude will use these instructions to help with translations.");
    
    Ok(())
}

fn generate_claude_instructions(config: &Config) -> String {
    let mut instructions = String::from("## elm-i18n Configuration\n\n");
    instructions.push_str("This project uses elm-i18n for managing translations. ");
    
    match config {
        Config::SingleFile { file, record_name, languages, .. } => {
            instructions.push_str(&format!("It's configured in **single-file mode**.\n\n"));
            instructions.push_str("### Configuration Details\n\n");
            instructions.push_str(&format!("- **Translation file**: `{}`\n", file.display()));
            instructions.push_str(&format!("- **Record type**: `{}`\n", record_name));
            instructions.push_str(&format!("- **Languages**: {}\n", languages.join(", ")));
            instructions.push_str("\n### Usage Examples\n\n");
            instructions.push_str("```bash\n");
            instructions.push_str("# Add a simple translation\n");
            instructions.push_str(&format!("elm-i18n add myKey --en \"Hello\" --fr \"Bonjour\"\n\n"));
            instructions.push_str("# Add a function translation\n");
            instructions.push_str("elm-i18n add-fn itemCount \\\n");
            instructions.push_str("  --type-sig \"Int -> String\" \\\n");
            instructions.push_str("  --en \"\\n -> if n == 1 then \\\"1 item\\\" else String.fromInt n ++ \\\" items\\\"\" \\\n");
            instructions.push_str("  --fr \"\\n -> if n == 1 then \\\"1 élément\\\" else String.fromInt n ++ \\\" éléments\\\"\"\n\n");
            instructions.push_str("# Check if a key exists\n");
            instructions.push_str("elm-i18n check myKey\n\n");
            instructions.push_str("# List all translations\n");
            instructions.push_str("elm-i18n list\n\n");
            instructions.push_str("# Remove a translation\n");
            instructions.push_str("elm-i18n remove myKey\n");
            instructions.push_str("```\n");
        }
        Config::MultiFile { files, languages, .. } => {
            instructions.push_str(&format!("It's configured in **multi-file mode** with {} translation files.\n\n", files.len()));
            instructions.push_str("### Configuration Details\n\n");
            instructions.push_str(&format!("- **Languages**: {}\n", languages.join(", ")));
            instructions.push_str("- **Translation files**:\n");
            
            for (shortcut, file_config) in files {
                instructions.push_str(&format!("  - `--target {}` → `{}` (Record: `{}`)\n", 
                    shortcut, 
                    file_config.path.display(),
                    file_config.record_name
                ));
            }
            
            instructions.push_str("\n### Usage Examples\n\n");
            instructions.push_str("```bash\n");
            
            if let Some((first_shortcut, _)) = files.iter().next() {
                instructions.push_str(&format!("# Add a translation to the {} file\n", first_shortcut));
                instructions.push_str(&format!("elm-i18n --target {} add myKey --en \"Hello\" --fr \"Bonjour\"\n\n", first_shortcut));
                
                instructions.push_str(&format!("# Add a function translation to the {} file\n", first_shortcut));
                instructions.push_str(&format!("elm-i18n --target {} add-fn itemCount \\\n", first_shortcut));
                instructions.push_str("  --type-sig \"Int -> String\" \\\n");
                instructions.push_str("  --en \"\\n -> if n == 1 then \\\"1 item\\\" else String.fromInt n ++ \\\" items\\\"\" \\\n");
                instructions.push_str("  --fr \"\\n -> if n == 1 then \\\"1 élément\\\" else String.fromInt n ++ \\\" éléments\\\"\"\n\n");
                
                instructions.push_str(&format!("# Check if a key exists in the {} file\n", first_shortcut));
                instructions.push_str(&format!("elm-i18n --target {} check myKey\n\n", first_shortcut));
                
                instructions.push_str(&format!("# List all translations in the {} file\n", first_shortcut));
                instructions.push_str(&format!("elm-i18n --target {} list\n\n", first_shortcut));
                
                instructions.push_str(&format!("# Remove a translation from the {} file\n", first_shortcut));
                instructions.push_str(&format!("elm-i18n --target {} remove myKey\n", first_shortcut));
            }
            
            instructions.push_str("```\n");
            
            instructions.push_str("\n### Important Notes\n\n");
            instructions.push_str("- **Always specify `--target <shortcut>`** when working with translations\n");
            instructions.push_str("- Each file has its own record type and translation set\n");
            instructions.push_str("- Use `elm-i18n status` to see all available shortcuts\n");
        }
    }
    
    instructions.push_str("\n### Additional Commands\n\n");
    instructions.push_str("```bash\n");
    instructions.push_str("# Show current configuration\n");
    instructions.push_str("elm-i18n status\n\n");
    instructions.push_str("# Find and remove unused translations\n");
    if config.is_multi_file() {
        if let Config::MultiFile { files, .. } = config {
            if let Some((shortcut, _)) = files.iter().next() {
                instructions.push_str(&format!("elm-i18n --target {} remove-unused --confirm\n\n", shortcut));
            }
        }
    } else {
        instructions.push_str("elm-i18n remove-unused --confirm\n\n");
    }
    instructions.push_str("# Add translation and replace hardcoded strings\n");
    if config.is_multi_file() {
        if let Config::MultiFile { files, .. } = config {
            if let Some((shortcut, _)) = files.iter().next() {
                instructions.push_str(&format!("elm-i18n --target {} add myKey --en \"Hello\" --fr \"Bonjour\" --replace\n", shortcut));
            }
        }
    } else {
        instructions.push_str("elm-i18n add myKey --en \"Hello\" --fr \"Bonjour\" --replace\n");
    }
    instructions.push_str("```\n");
    
    instructions.push_str("\n### Key Naming Conventions\n\n");
    instructions.push_str("- Use camelCase for keys (e.g., `welcomeMessage`, `userProfile`)\n");
    instructions.push_str("- Keys cannot contain dots (.) as they're reserved for access syntax\n");
    instructions.push_str("- Elm reserved words will automatically get an underscore suffix\n");
    
    instructions
}

/// Handle the status command
fn handle_status() -> Result<()> {
    println!("{} Configuration Status", "🔧".blue());
    println!();
    
    match Config::load()? {
        Some(config) => {
            match &config {
                Config::SingleFile { file, record_name, languages, source_dir, .. } => {
                    println!("Mode: {}", "Single-file".green());
                    println!("File: {}", file.display());
                    println!("Record Type: {}", record_name.yellow());
                    println!("Languages: {}", languages.join(", "));
                    println!("Source Directory: {}", source_dir.display());
                    println!();
                    println!("Usage example:");
                    println!("  elm-i18n add myKey --en \"Hello\" --fr \"Bonjour\"");
                }
                Config::MultiFile { files, languages, source_dir, .. } => {
                    println!("Mode: {}", "Multi-file".green());
                    println!("Languages: {}", languages.join(", "));
                    println!("Source Directory: {}", source_dir.display());
                    println!();
                    println!("Available shortcuts:");
                    
                    let shortcuts = config.get_shortcuts();
                    for (shortcut, path) in &shortcuts {
                        if let Some(file_config) = files.get(shortcut) {
                            println!("  {} → {}", 
                                format!("--target {}", shortcut).yellow(),
                                path.display()
                            );
                            println!("       Record Type: {}", file_config.record_name.cyan());
                        }
                    }
                    
                    println!();
                    println!("Usage example:");
                    if let Some((shortcut, _)) = shortcuts.first() {
                        println!("  elm-i18n --target {} add myKey --en \"Hello\" --fr \"Bonjour\"", shortcut);
                    }
                }
            }
        }
        None => {
            println!("{} No configuration found!", "⚠".yellow());
            println!();
            println!("Run {} to create a configuration file.", "elm-i18n setup".green());
        }
    }
    
    Ok(())
}

/// Handle the version command
fn handle_version() -> Result<()> {
    println!("elm-i18n v{}", env!("CARGO_PKG_VERSION"));
    println!("CLI tool for managing Elm I18n translations");
    println!();
    println!("New in v0.5.0:");
    println!("  • Configuration file support (elm-i18n.json)");
    println!("  • Multi-file translation management");
    println!("  • Custom shortcuts for quick file access");
    println!("  • Run 'elm-i18n setup' to create configuration");
    println!();
    println!("New in v0.4.0:");
    println!("  • Added 'list' command to view all translations");
    println!("  • Support for --verbose to see full translation values");
    println!("  • Filter translations with --filter option");
    Ok(())
}

/// Handle the setup command
fn handle_setup() -> Result<()> {
    if config_exists() {
        eprintln!("{} Configuration file already exists: elm-i18n.json", "✗".red());
        eprintln!("Delete it first if you want to reconfigure.");
        std::process::exit(1);
    }
    
    println!("{} Welcome to elm-i18n setup!", "🎉".blue());
    println!();
    println!("This will create an elm-i18n.json configuration file.");
    println!();
    
    // Ask for mode
    print!("Choose translation mode:\n");
    print!("  1) Single-file mode (one I18n.elm file)\n");
    print!("  2) Multi-file mode (separate files for different parts)\n");
    print!("\nSelect mode [1-2]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let mode_choice = input.trim();
    
    let config = if mode_choice == "2" {
        setup_multi_file_config()?
    } else {
        setup_single_file_config()?
    };
    
    config.save()?;
    
    println!();
    println!("{} Created elm-i18n.json configuration file", "✓".green());
    
    if config.is_multi_file() {
        println!();
        println!("Available shortcuts:");
        for (shortcut, path) in config.get_shortcuts() {
            println!("  {} → {}", 
                format!("--{}", shortcut).yellow(),
                path.display()
            );
        }
        println!();
        println!("Example usage:");
        if let Some((shortcut, _)) = config.get_shortcuts().first() {
            println!("  elm-i18n --{} add myKey --en \"Hello\" --fr \"Bonjour\"", shortcut);
        }
    } else {
        println!();
        println!("Example usage:");
        println!("  elm-i18n add myKey --en \"Hello\" --fr \"Bonjour\"");
    }
    
    Ok(())
}

/// Setup single-file configuration
fn setup_single_file_config() -> Result<Config> {
    println!();
    print!("Path to I18n.elm file [src/I18n.elm]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let file_path = if input.trim().is_empty() {
        PathBuf::from("src/I18n.elm")
    } else {
        PathBuf::from(input.trim())
    };
    
    print!("Record name [Translations]: ");
    io::stdout().flush()?;
    
    input.clear();
    io::stdin().read_line(&mut input)?;
    let record_name = if input.trim().is_empty() {
        "Translations".to_string()
    } else {
        input.trim().to_string()
    };
    
    print!("Source directory [src]: ");
    io::stdout().flush()?;
    
    input.clear();
    io::stdin().read_line(&mut input)?;
    let source_dir = if input.trim().is_empty() {
        PathBuf::from("src")
    } else {
        PathBuf::from(input.trim())
    };
    
    print!("Languages (comma-separated) [en,fr]: ");
    io::stdout().flush()?;
    
    input.clear();
    io::stdin().read_line(&mut input)?;
    let languages = if input.trim().is_empty() {
        vec!["en".to_string(), "fr".to_string()]
    } else {
        input.trim().split(',').map(|s| s.trim().to_string()).collect()
    };
    
    Ok(Config::SingleFile {
        elm_i18n_version: env!("CARGO_PKG_VERSION").to_string(),
        languages,
        source_dir,
        file: file_path,
        record_name,
    })
}

/// Setup multi-file configuration
fn setup_multi_file_config() -> Result<Config> {
    use std::collections::HashMap;
    
    println!();
    print!("Source directory [src]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let source_dir = if input.trim().is_empty() {
        PathBuf::from("src")
    } else {
        PathBuf::from(input.trim())
    };
    
    print!("Languages (comma-separated) [en,fr]: ");
    io::stdout().flush()?;
    
    input.clear();
    io::stdin().read_line(&mut input)?;
    let languages = if input.trim().is_empty() {
        vec!["en".to_string(), "fr".to_string()]
    } else {
        input.trim().split(',').map(|s| s.trim().to_string()).collect()
    };
    
    let mut files = HashMap::new();
    
    println!();
    println!("Now let's configure your translation files.");
    println!("Enter shortcuts and file paths (empty shortcut to finish):");
    
    loop {
        println!();
        print!("Shortcut (e.g., 'app', 'landing', 'admin'): ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        let shortcut = input.trim().to_string();
        
        if shortcut.is_empty() {
            if files.is_empty() {
                println!("{} At least one file must be configured", "⚠".yellow());
                continue;
            }
            break;
        }
        
        print!("File path (e.g., 'src/I18n/App.elm'): ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        let path = PathBuf::from(input.trim());
        
        print!("Record name (e.g., 'AppTranslations'): ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        let record_name = input.trim().to_string();
        
        files.insert(shortcut.clone(), FileConfig {
            path,
            record_name,
        });
        
        println!("{} Added: --{}", "✓".green(), shortcut);
    }
    
    Ok(Config::MultiFile {
        elm_i18n_version: env!("CARGO_PKG_VERSION").to_string(),
        languages,
        source_dir,
        files,
    })
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
    record_name: &str,
) -> Result<()> {
    // Check if file exists
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        eprintln!("{} Run 'elm-i18n init' to create a new I18n.elm file", "ℹ".blue());
        std::process::exit(1);
    }
    
    // Check if key already exists
    match check_key_exists_with_record_name(file, key, record_name)? {
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
            
            add_translation_with_record_name(file, &translation, record_name)?;
            
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

fn handle_check(file: &PathBuf, key: &str, record_name: &str) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    match check_key_exists_with_record_name(file, key, record_name)? {
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

fn handle_init(file: &PathBuf, languages: &str, record_name: &str) -> Result<()> {
    if file.exists() {
        eprintln!("{} File already exists: {}", "✗".red(), file.display());
        eprintln!("Remove it first if you want to reinitialize.");
        std::process::exit(1);
    }
    
    let langs: Vec<String> = languages
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
    
    let template = get_i18n_template_with_record_name(&langs, record_name);
    create_i18n_file(file, &template)?;
    
    println!("{} Created {} with basic structure", "✓".green(), file.display());
    println!("Languages: {}", langs.join(", "));
    
    Ok(())
}

fn handle_remove(file: &PathBuf, key: &str, record_name: &str) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    // Check if key exists first
    match check_key_exists_with_record_name(file, key, record_name)? {
        Some(translation) => {
            // Show what will be removed
            println!("{} Removing translation '{}':", "ℹ".blue(), key.yellow());
            println!("  {}: {}", "EN".green(), translation.en);
            println!("  {}: {}", "FR".green(), translation.fr);
            println!();
            
            // Remove the translation
            match remove_translation_with_record_name(file, key, record_name) {
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

fn handle_remove_unused(file: &PathBuf, src_dir: &PathBuf, confirm: bool, record_name: &str) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    println!("{} Scanning for unused translation keys...", "🔍".blue());

    // Find all unused keys
    let unused_keys = find_unused_keys(file, src_dir, record_name)?;
    
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
        match remove_translation_with_record_name(file, key, record_name) {
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

fn handle_list(file: &PathBuf, verbose: bool, filter: &Option<String>, record_name: &str) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }
    
    // Parse the I18n file
    let parse_result = parse_i18n_file_with_record_name(file, record_name)?;
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