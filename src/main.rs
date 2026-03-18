use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

mod config;
mod generator;
mod parser;
mod replacer;
mod templates;
mod types;

use crate::config::{config_exists, config_file_path, prompt_setup_message, Config, FileConfig};
use crate::generator::{
    add_translation_with_record_name, create_i18n_file, remove_translation_with_record_name,
};
use crate::parser::{check_key_exists_with_record_name, parse_i18n_file_with_record_name};
use crate::replacer::{find_string_occurrences, find_unused_keys, replace_strings};
use crate::templates::get_i18n_template_with_record_name;
use crate::types::Translation;

// Elm reserved words
const ELM_RESERVED_WORDS: &[&str] = &[
    "if",
    "then",
    "else",
    "case",
    "of",
    "let",
    "in",
    "type",
    "module",
    "where",
    "import",
    "exposing",
    "as",
    "port",
    "effect",
    "command",
    "subscription",
    "alias",
    "infixl",
    "infixr",
    "infix",
];

const LOCAL_CONFIG_FILE: &str = "elm-i18n/config.json";
const LOCAL_SUPPRESSED_FILE: &str = "elm-i18n/suppressed.json";
const SHARED_VALUES_CHECK_NAME: &str = "shared-values";

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

        /// Translation value as LANG=VALUE (e.g., -t en="Hello" -t fr="Bonjour")
        #[arg(short = 't', long = "translation", required = true)]
        translations: Vec<String>,

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

        /// Translation value as LANG=VALUE (e.g., -t en="impl" -t fr="impl")
        #[arg(short = 't', long = "translation", required = true)]
        translations: Vec<String>,

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

    /// Find keys that have exactly the same translations
    #[command(name = "duplicate-keys", alias = "duplicates")]
    DuplicateKeys {
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },

    /// Find keys whose value is identical in multiple languages
    #[command(name = "shared-values")]
    SharedValues {
        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,

        /// Suppress current findings by storing them in ./elm-i18n/
        #[arg(long)]
        suppress: bool,
    },

    /// Modify an existing translation (update specific language values only)
    Modify {
        /// The translation key to modify
        key: String,

        /// Translation value as LANG=VALUE (e.g., -t es="Hola")
        #[arg(short = 't', long = "translation", required = true)]
        translations: Vec<String>,

        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },

    /// Bulk-modify translations for one language from a JSON file
    #[command(name = "modify-bulk")]
    ModifyBulk {
        /// Language code to modify (e.g., "es", "de")
        #[arg(long)]
        lang: String,

        /// Path to JSON file with key-value translations (e.g., {"loading": "Cargando...", ...})
        #[arg(long = "from")]
        json_file: PathBuf,

        /// Path to I18n.elm file (defaults to src/I18n.elm)
        #[arg(long, default_value = "src/I18n.elm")]
        file: PathBuf,
    },

    /// Add a new language by duplicating an existing one
    #[command(name = "add-language")]
    AddLanguage {
        /// New language code (e.g., "de", "es", "ja")
        new_lang: String,

        /// Existing language to copy values from (e.g., "en")
        #[arg(long, default_value = "en")]
        from: String,
    },

    /// Show version information
    Version,
}

/// Validates and cleans a translation key
fn validate_and_clean_key(key: &str) -> Result<String> {
    // Check for forbidden characters
    if key.contains('.') {
        eprintln!(
            "{} Error: Translation keys cannot contain dots (.)",
            "✗".red()
        );
        eprintln!(
            "{} The dot character is reserved for accessing nested translations (e.g., t.welcome)",
            "ℹ".blue()
        );
        eprintln!("{} Please use camelCase or underscores instead", "ℹ".blue());
        std::process::exit(1);
    }

    // Handle reserved words
    let mut cleaned_key = key.to_string();
    if ELM_RESERVED_WORDS.contains(&key) {
        cleaned_key = format!("{}_", key);
        println!(
            "{} Warning: '{}' is a reserved word in Elm, using '{}' instead",
            "⚠".yellow(),
            key.yellow(),
            cleaned_key.green()
        );
    }

    // Validate key format (alphanumeric + underscores, starting with letter)
    if !cleaned_key.chars().next().unwrap_or('0').is_alphabetic() {
        eprintln!(
            "{} Error: Translation keys must start with a letter",
            "✗".red()
        );
        std::process::exit(1);
    }

    if !cleaned_key.chars().all(|c| c.is_alphanumeric() || c == '_') {
        eprintln!(
            "{} Error: Translation keys can only contain letters, numbers, and underscores",
            "✗".red()
        );
        std::process::exit(1);
    }

    Ok(cleaned_key)
}

/// Parse translation CLI arguments in LANG=VALUE format
fn parse_translation_args(
    args: &[String],
    languages: &[String],
) -> Result<std::collections::HashMap<String, String>> {
    let mut values = std::collections::HashMap::new();

    for arg in args {
        let (lang, value) = arg.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid translation format: '{}'. Expected LANG=VALUE (e.g., en=\"Hello\")",
                arg
            )
        })?;
        let lang = lang.trim().to_lowercase();
        if !languages.contains(&lang) {
            eprintln!(
                "{} Warning: language '{}' is not in configured languages: {}",
                "⚠".yellow(),
                lang.yellow(),
                languages.join(", ")
            );
        }
        values.insert(lang, value.to_string());
    }

    // Check that all configured languages have values
    for lang in languages {
        if !values.contains_key(lang) {
            eprintln!(
                "{} Missing translation for language '{}'. Use -t {}=\"...\"",
                "✗".red(),
                lang.yellow(),
                lang
            );
            std::process::exit(1);
        }
    }

    Ok(values)
}

/// Parse translation args without requiring all languages (for modify command)
fn parse_partial_translation_args(
    args: &[String],
    languages: &[String],
) -> Result<std::collections::HashMap<String, String>> {
    let mut values = std::collections::HashMap::new();

    for arg in args {
        let (lang, value) = arg.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid translation format: '{}'. Expected LANG=VALUE (e.g., es=\"Hola\")",
                arg
            )
        })?;
        let lang = lang.trim().to_lowercase();
        if !languages.contains(&lang) {
            eprintln!(
                "{} Warning: language '{}' is not in configured languages: {}",
                "⚠".yellow(),
                lang.yellow(),
                languages.join(", ")
            );
        }
        values.insert(lang, value.to_string());
    }

    if values.is_empty() {
        anyhow::bail!("At least one translation must be provided");
    }

    Ok(values)
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

    let languages = config.languages();

    match cli.command {
        Commands::Setup => unreachable!(),

        Commands::Add {
            key,
            translations,
            file,
            replace,
            src_dir,
        } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let values = parse_translation_args(&translations, languages)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            let actual_src_dir = if src_dir.to_str() == Some("src") {
                config.source_dir().clone()
            } else {
                src_dir
            };
            handle_add(
                &actual_file,
                &cleaned_key,
                &values,
                false,
                None,
                replace,
                &actual_src_dir,
                &record_name,
                languages,
            )?;
        }

        Commands::AddFunction {
            key,
            type_sig,
            translations,
            file,
        } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let values = parse_translation_args(&translations, languages)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_add(
                &actual_file,
                &cleaned_key,
                &values,
                true,
                Some(type_sig),
                false,
                config.source_dir(),
                &record_name,
                languages,
            )?;
        }

        Commands::Check { key, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_check(&actual_file, &cleaned_key, &record_name, languages)?;
        }

        Commands::Init {
            languages: init_langs,
            file,
        } => {
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_init(&actual_file, &init_langs, &record_name)?;
        }

        Commands::Modify {
            key,
            translations,
            file,
        } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let values = parse_partial_translation_args(&translations, languages)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_modify(&actual_file, &cleaned_key, &values, &record_name, languages)?;
        }

        Commands::ModifyBulk {
            lang,
            json_file,
            file,
        } => {
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_modify_bulk(&actual_file, &lang, &json_file, &record_name, languages)?;
        }

        Commands::Remove { key, file } => {
            let cleaned_key = validate_and_clean_key(&key)?;
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_remove(&actual_file, &cleaned_key, &record_name, languages)?;
        }

        Commands::RemoveUnused {
            file,
            src_dir,
            confirm,
        } => {
            let actual_src_dir = if src_dir.to_str() == Some("src") {
                config.source_dir().clone()
            } else {
                src_dir
            };

            // In multi-file mode without a target, process all files
            if cli.target.is_none() {
                if let Config::MultiFile { files, .. } = &config {
                    println!(
                        "{} Running remove-unused on all translation files...\n",
                        "🔍".blue()
                    );
                    for (shortcut, file_config) in files {
                        if !file_config.path.exists() {
                            println!(
                                "  {} Skipping {} (file not found)\n",
                                "⚠".yellow(),
                                shortcut
                            );
                            continue;
                        }
                        println!(
                            "{} Processing {} ({})...",
                            "→".cyan(),
                            shortcut.yellow(),
                            file_config.path.display()
                        );
                        handle_remove_unused(
                            &file_config.path,
                            &actual_src_dir,
                            confirm,
                            &file_config.record_name,
                            languages,
                        )?;
                        println!();
                    }
                } else {
                    // Single file mode
                    handle_remove_unused(
                        &file_path,
                        &actual_src_dir,
                        confirm,
                        &record_name,
                        languages,
                    )?;
                }
            } else {
                // Target was specified, use the determined file
                let actual_file = if file.to_str() == Some("src/I18n.elm") {
                    file_path.clone()
                } else {
                    file
                };
                handle_remove_unused(
                    &actual_file,
                    &actual_src_dir,
                    confirm,
                    &record_name,
                    languages,
                )?;
            }
        }

        Commands::List {
            file,
            verbose,
            filter,
        } => {
            let actual_file = if file.to_str() == Some("src/I18n.elm") {
                file_path.clone()
            } else {
                file
            };
            handle_list(&actual_file, verbose, &filter, &record_name, languages)?
        }

        Commands::DuplicateKeys { file } => {
            // In multi-file mode without a target, find duplicates across all files
            if cli.target.is_none() {
                if let Config::MultiFile { files, .. } = &config {
                    handle_duplicates_cross_file(files, languages)?;
                } else {
                    // Single file mode
                    handle_duplicates(&file_path, &record_name, languages)?;
                }
            } else {
                // Target was specified, use the determined file
                let actual_file = if file.to_str() == Some("src/I18n.elm") {
                    file_path.clone()
                } else {
                    file
                };
                handle_duplicates(&actual_file, &record_name, languages)?;
            }
        }

        Commands::SharedValues { file, suppress } => {
            if cli.target.is_none() {
                if let Config::MultiFile { files, .. } = &config {
                    handle_shared_values_cross_file(files, languages, suppress)?;
                } else {
                    handle_shared_values(&file_path, &record_name, languages, suppress)?;
                }
            } else {
                let actual_file = if file.to_str() == Some("src/I18n.elm") {
                    file_path.clone()
                } else {
                    file
                };
                handle_shared_values(&actual_file, &record_name, languages, suppress)?;
            }
        }

        Commands::AddLanguage { new_lang, from } => {
            handle_add_language(&config, &new_lang, &from)?;
        }

        Commands::Version => unreachable!(),
        Commands::Status => unreachable!(),
        Commands::SetupClaude => unreachable!(),
    }

    Ok(())
}

/// Determine which file to target based on config and shortcut
fn determine_target_file(
    config: &Config,
    shortcut: &Option<String>,
    command: &Commands,
) -> Result<(PathBuf, String)> {
    // For Init command, we might allow creation of new files
    let is_init = matches!(command, Commands::Init { .. });
    // These commands can work without a target (they process all files)
    let is_remove_unused = matches!(command, Commands::RemoveUnused { .. });
    let is_duplicates = matches!(command, Commands::DuplicateKeys { .. });
    let is_shared_values = matches!(command, Commands::SharedValues { .. });
    let is_add_language = matches!(command, Commands::AddLanguage { .. });

    match config {
        Config::SingleFile {
            file, record_name, ..
        } => {
            if shortcut.is_some() {
                eprintln!(
                    "{} Warning: File shortcuts are ignored in single-file mode",
                    "⚠".yellow()
                );
            }
            Ok((file.clone(), record_name.clone()))
        }
        Config::MultiFile { files, .. } => {
            match shortcut {
                Some(s) => match files.get(s) {
                    Some(file_config) => {
                        Ok((file_config.path.clone(), file_config.record_name.clone()))
                    }
                    None => {
                        eprintln!("{} Unknown file shortcut: {}", "✗".red(), s.yellow());
                        config.print_shortcuts();
                        std::process::exit(1);
                    }
                },
                None => {
                    // Some commands can run without a target - they process all files
                    if is_remove_unused || is_duplicates || is_shared_values || is_add_language {
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

    println!(
        "{} Setting up CLAUDE.md with elm-i18n instructions...",
        "🤖".blue()
    );
    println!();

    // Load configuration to understand project setup
    let config = match Config::load()? {
        Some(config) => config,
        None => {
            eprintln!(
                "{} No elm-i18n configuration found at {}!",
                "✗".red(),
                config_file_path()
            );
            eprintln!(
                "Run {} first to create a configuration.",
                "elm-i18n setup".green()
            );
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
            let before_section = existing
                .split("## elm-i18n Configuration")
                .next()
                .unwrap_or("");
            let after_section = existing
                .split("## elm-i18n Configuration")
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
        format!(
            "# Project-Specific Instructions for Claude\n\n{}",
            elm_i18n_section
        )
    };

    // Write the file
    fs::write(&claude_path, final_content)?;

    println!(
        "{} CLAUDE.md has been {}",
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
        Config::SingleFile {
            file,
            record_name,
            languages,
            ..
        } => {
            instructions.push_str(&format!("It's configured in **single-file mode**.\n\n"));
            instructions.push_str("### Configuration Details\n\n");
            instructions.push_str(&format!("- **Translation file**: `{}`\n", file.display()));
            instructions.push_str(&format!("- **Record type**: `{}`\n", record_name));
            instructions.push_str(&format!("- **Languages**: {}\n", languages.join(", ")));
            instructions.push_str("\n### Usage Examples\n\n");
            instructions.push_str("```bash\n");
            instructions.push_str("# Add a simple translation\n");
            instructions.push_str(&format!(
                "elm-i18n add myKey -t en=\"Hello\" -t fr=\"Bonjour\"\n\n"
            ));
            instructions.push_str("# Add a function translation\n");
            instructions.push_str("elm-i18n add-fn itemCount \\\n");
            instructions.push_str("  --type-sig \"Int -> String\" \\\n");
            instructions.push_str("  -t en=\"\\n -> if n == 1 then \\\"1 item\\\" else String.fromInt n ++ \\\" items\\\"\" \\\n");
            instructions.push_str("  -t fr=\"\\n -> if n == 1 then \\\"1 élément\\\" else String.fromInt n ++ \\\" éléments\\\"\"\n\n");
            instructions.push_str("# Check if a key exists\n");
            instructions.push_str("elm-i18n check myKey\n\n");
            instructions.push_str("# List all translations\n");
            instructions.push_str("elm-i18n list\n\n");
            instructions.push_str("# Remove a translation\n");
            instructions.push_str("elm-i18n remove myKey\n");
            instructions.push_str("```\n");
        }
        Config::MultiFile {
            files, languages, ..
        } => {
            instructions.push_str(&format!(
                "It's configured in **multi-file mode** with {} translation files.\n\n",
                files.len()
            ));
            instructions.push_str("### Configuration Details\n\n");
            instructions.push_str(&format!("- **Languages**: {}\n", languages.join(", ")));
            instructions.push_str("- **Translation files**:\n");

            for (shortcut, file_config) in files {
                instructions.push_str(&format!(
                    "  - `--target {}` → `{}` (Record: `{}`)\n",
                    shortcut,
                    file_config.path.display(),
                    file_config.record_name
                ));
            }

            instructions.push_str("\n### Usage Examples\n\n");
            instructions.push_str("```bash\n");

            if let Some((first_shortcut, _)) = files.iter().next() {
                instructions.push_str(&format!(
                    "# Add a translation to the {} file\n",
                    first_shortcut
                ));
                instructions.push_str(&format!(
                    "elm-i18n --target {} add myKey -t en=\"Hello\" -t fr=\"Bonjour\"\n\n",
                    first_shortcut
                ));

                instructions.push_str(&format!(
                    "# Add a function translation to the {} file\n",
                    first_shortcut
                ));
                instructions.push_str(&format!(
                    "elm-i18n --target {} add-fn itemCount \\\n",
                    first_shortcut
                ));
                instructions.push_str("  --type-sig \"Int -> String\" \\\n");
                instructions.push_str("  -t en=\"\\n -> if n == 1 then \\\"1 item\\\" else String.fromInt n ++ \\\" items\\\"\" \\\n");
                instructions.push_str("  -t fr=\"\\n -> if n == 1 then \\\"1 élément\\\" else String.fromInt n ++ \\\" éléments\\\"\"\n\n");

                instructions.push_str(&format!(
                    "# Check if a key exists in the {} file\n",
                    first_shortcut
                ));
                instructions.push_str(&format!(
                    "elm-i18n --target {} check myKey\n\n",
                    first_shortcut
                ));

                instructions.push_str(&format!(
                    "# List all translations in the {} file\n",
                    first_shortcut
                ));
                instructions.push_str(&format!("elm-i18n --target {} list\n\n", first_shortcut));

                instructions.push_str(&format!(
                    "# Remove a translation from the {} file\n",
                    first_shortcut
                ));
                instructions.push_str(&format!(
                    "elm-i18n --target {} remove myKey\n",
                    first_shortcut
                ));
            }

            instructions.push_str("```\n");

            instructions.push_str("\n### Important Notes\n\n");
            instructions.push_str(
                "- **Always specify `--target <shortcut>`** when working with translations\n",
            );
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
                instructions.push_str(&format!(
                    "elm-i18n --target {} remove-unused --confirm\n\n",
                    shortcut
                ));
            }
        }
    } else {
        instructions.push_str("elm-i18n remove-unused --confirm\n\n");
    }
    instructions.push_str("# Add translation and replace hardcoded strings\n");
    if config.is_multi_file() {
        if let Config::MultiFile { files, .. } = config {
            if let Some((shortcut, _)) = files.iter().next() {
                instructions.push_str(&format!(
                    "elm-i18n --target {} add myKey -t en=\"Hello\" -t fr=\"Bonjour\" --replace\n",
                    shortcut
                ));
            }
        }
    } else {
        instructions.push_str("elm-i18n add myKey -t en=\"Hello\" -t fr=\"Bonjour\" --replace\n");
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
        Some(config) => match &config {
            Config::SingleFile {
                file,
                record_name,
                languages,
                source_dir,
                ..
            } => {
                println!("Mode: {}", "Single-file".green());
                println!("File: {}", file.display());
                println!("Record Type: {}", record_name.yellow());
                println!("Languages: {}", languages.join(", "));
                println!("Source Directory: {}", source_dir.display());
                println!();
                println!("Usage example:");
                println!("  elm-i18n add myKey -t en=\"Hello\" -t fr=\"Bonjour\"");
            }
            Config::MultiFile {
                files,
                languages,
                source_dir,
                ..
            } => {
                println!("Mode: {}", "Multi-file".green());
                println!("Languages: {}", languages.join(", "));
                println!("Source Directory: {}", source_dir.display());
                println!();
                println!("Available shortcuts:");

                let shortcuts = config.get_shortcuts();
                for (shortcut, path) in &shortcuts {
                    if let Some(file_config) = files.get(shortcut) {
                        println!(
                            "  {} → {}",
                            format!("--target {}", shortcut).yellow(),
                            path.display()
                        );
                        println!("       Record Type: {}", file_config.record_name.cyan());
                    }
                }

                println!();
                println!("Usage example:");
                if let Some((shortcut, _)) = shortcuts.first() {
                    println!(
                        "  elm-i18n --target {} add myKey -t en=\"Hello\" -t fr=\"Bonjour\"",
                        shortcut
                    );
                }
            }
        },
        None => {
            println!("{} No configuration found!", "⚠".yellow());
            println!();
            println!(
                "Run {} to create a configuration file.",
                "elm-i18n setup".green()
            );
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
    println!("  • Configuration file support ({})", config_file_path());
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
        eprintln!(
            "{} Configuration file already exists: {}",
            "✗".red(),
            config_file_path()
        );
        eprintln!("Delete it first if you want to reconfigure.");
        std::process::exit(1);
    }

    println!("{} Welcome to elm-i18n setup!", "🎉".blue());
    println!();
    println!(
        "This will create a {} configuration file.",
        config_file_path()
    );
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
    println!(
        "{} Created {} configuration file",
        "✓".green(),
        config_file_path()
    );

    if config.is_multi_file() {
        println!();
        println!("Available shortcuts:");
        for (shortcut, path) in config.get_shortcuts() {
            println!(
                "  {} → {}",
                format!("--{}", shortcut).yellow(),
                path.display()
            );
        }
        println!();
        println!("Example usage:");
        if let Some((shortcut, _)) = config.get_shortcuts().first() {
            println!(
                "  elm-i18n --{} add myKey -t en=\"Hello\" -t fr=\"Bonjour\"",
                shortcut
            );
        }
    } else {
        println!();
        println!("Example usage:");
        println!("  elm-i18n add myKey -t en=\"Hello\" -t fr=\"Bonjour\"");
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
        input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
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
        input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
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

        files.insert(shortcut.clone(), FileConfig { path, record_name });

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
    values: &std::collections::HashMap<String, String>,
    is_function: bool,
    type_sig: Option<String>,
    replace: bool,
    src_dir: &PathBuf,
    record_name: &str,
    languages: &[String],
) -> Result<()> {
    // Check if file exists
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        eprintln!(
            "{} Run 'elm-i18n init' to create a new I18n.elm file",
            "ℹ".blue()
        );
        std::process::exit(1);
    }

    // Check if key already exists
    match check_key_exists_with_record_name(file, key, record_name, languages)? {
        Some(existing) => {
            println!(
                "{} Translation '{}' already exists:",
                "ℹ".blue(),
                key.yellow()
            );
            for lang in languages {
                if let Some(val) = existing.values.get(lang) {
                    println!("  {}: {}", lang.to_uppercase().green(), val);
                }
            }
            println!();
            println!(
                "The existing translations might be sufficient. Consider using a different key."
            );
        }
        None => {
            // Add the translation
            let translation = Translation {
                key: key.to_string(),
                values: values.clone(),
                is_function,
                type_signature: type_sig,
            };

            add_translation_with_record_name(file, &translation, record_name, languages)?;

            println!(
                "{} Added translation '{}' to {}",
                "✓".green(),
                key.yellow(),
                file.display()
            );

            if !is_function {
                for lang in languages {
                    if let Some(val) = values.get(lang) {
                        println!("  {}: {}", lang.to_uppercase().green(), val);
                    }
                }
            }

            // Handle string replacement if requested
            if replace && !is_function {
                println!();
                println!(
                    "{} Searching for hardcoded strings to replace...",
                    "🔍".blue()
                );

                let search_strings: Vec<&str> = values.values().map(|s| s.as_str()).collect();
                let matches = find_string_occurrences(src_dir, &search_strings)?;

                if matches.is_empty() {
                    println!("{} No hardcoded strings found to replace", "ℹ".blue());
                } else {
                    // Show what will be replaced for each language
                    for (lang, value) in values {
                        let lang_matches: Vec<_> = matches
                            .iter()
                            .filter(|m| m.line_content.contains(&format!(r#""{}""#, value)))
                            .collect();

                        if !lang_matches.is_empty() {
                            println!();
                            println!(
                                "{} Found {} occurrences of \"{}\" ({}):",
                                "✓".green(),
                                lang_matches.len(),
                                value,
                                lang.to_uppercase()
                            );
                            for mat in lang_matches.iter().take(3) {
                                println!("  {}:{}:", mat.file_path.display(), mat.line_number);
                                println!("    {}", mat.line_content.trim());
                            }
                            if lang_matches.len() > 3 {
                                println!("  ... and {} more", lang_matches.len() - 3);
                            }
                        }
                    }

                    // Perform replacements
                    println!();
                    println!("{} Replacing strings with t.{}...", "🔄".blue(), key);
                    replace_strings(&matches, key, "I18n")?;

                    println!(
                        "{} Replaced {} occurrences across {} file(s)",
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

fn handle_check(file: &PathBuf, key: &str, record_name: &str, languages: &[String]) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    match check_key_exists_with_record_name(file, key, record_name, languages)? {
        Some(translation) => {
            println!("{} Translation '{}' exists:", "✓".green(), key.yellow());
            for lang in languages {
                if let Some(val) = translation.values.get(lang) {
                    println!("  {}: {}", lang.to_uppercase().green(), val);
                }
            }

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

    println!(
        "{} Created {} with basic structure",
        "✓".green(),
        file.display()
    );
    println!("Languages: {}", langs.join(", "));

    Ok(())
}

fn handle_remove(file: &PathBuf, key: &str, record_name: &str, languages: &[String]) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    // Check if key exists first
    match check_key_exists_with_record_name(file, key, record_name, languages)? {
        Some(translation) => {
            // Show what will be removed
            println!("{} Removing translation '{}':", "ℹ".blue(), key.yellow());
            for lang in languages {
                if let Some(val) = translation.values.get(lang) {
                    println!("  {}: {}", lang.to_uppercase().green(), val);
                }
            }
            println!();

            // Remove the translation
            match remove_translation_with_record_name(file, key, record_name, languages) {
                Ok(_) => {
                    println!(
                        "{} Removed translation '{}' from {}",
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

fn handle_remove_unused(
    file: &PathBuf,
    src_dir: &PathBuf,
    confirm: bool,
    record_name: &str,
    languages: &[String],
) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    println!("{} Scanning for unused translation keys...", "🔍".blue());

    // Find all unused keys
    let unused_keys = find_unused_keys(file, src_dir, record_name, languages)?;

    if unused_keys.is_empty() {
        println!("{} All translation keys are in use!", "✓".green());
        return Ok(());
    }

    // Show unused keys
    println!();
    println!(
        "{} Found {} unused translation keys:",
        "⚠".yellow(),
        unused_keys.len()
    );
    for key in &unused_keys {
        println!("  • {}", key.yellow());
    }

    if !confirm {
        println!();
        println!(
            "{} To remove these keys, run with --confirm flag:",
            "ℹ".blue()
        );
        println!("  elm-i18n remove-unused --confirm");
        return Ok(());
    }

    // Remove the unused keys
    println!();
    println!("{} Removing unused keys...", "🗑".red());

    for key in &unused_keys {
        match remove_translation_with_record_name(file, key, record_name, languages) {
            Ok(_) => {
                println!("  {} Removed: {}", "✓".green(), key);
            }
            Err(e) => {
                eprintln!("  {} Failed to remove {}: {}", "✗".red(), key, e);
            }
        }
    }

    println!();
    println!(
        "{} Removed {} unused translation keys",
        "✓".green(),
        unused_keys.len()
    );

    Ok(())
}

fn handle_list(
    file: &PathBuf,
    verbose: bool,
    filter: &Option<String>,
    record_name: &str,
    languages: &[String],
) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    // Parse the I18n file
    let parse_result = parse_i18n_file_with_record_name(file, record_name, languages)?;
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
            println!(
                "{} No translations found matching '{}'",
                "✗".red(),
                filter.as_ref().unwrap().yellow()
            );
        } else {
            println!("{} No translations found", "✗".red());
        }
        return Ok(());
    }

    // Display results
    println!(
        "{} Found {} translation{}:",
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

            // Show translations for each language
            for lang in languages {
                if let Some(val) = translation.values.get(lang) {
                    println!(
                        "    {}: {}",
                        lang.to_uppercase().green(),
                        if val.contains('\n') {
                            format!(
                                "\n{}",
                                val.lines()
                                    .map(|line| format!("      {}", line))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )
                        } else {
                            val.clone()
                        }
                    );
                }
            }

            println!();
        }
    } else {
        // Simple list
        for (key, translation) in &translations {
            let type_info = if translation.is_function {
                format!(
                    " ({})",
                    translation
                        .type_signature
                        .as_ref()
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

fn handle_duplicates(file: &PathBuf, record_name: &str, languages: &[String]) -> Result<()> {
    use std::collections::HashMap;

    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    println!("{} Scanning for duplicate translations...", "🔍".blue());

    // Parse the I18n file
    let parse_result = parse_i18n_file_with_record_name(file, record_name, languages)?;

    // Build a map: sorted values -> Vec<key>
    let mut value_to_keys: HashMap<Vec<(String, String)>, Vec<String>> = HashMap::new();

    for (key, translation) in &parse_result.translations {
        if translation.is_function {
            continue;
        }

        let mut sorted_values: Vec<(String, String)> = translation
            .values
            .iter()
            .map(|(lang, value)| (lang.clone(), value.clone()))
            .collect();
        sorted_values.sort();
        value_to_keys
            .entry(sorted_values)
            .or_default()
            .push(key.clone());
    }

    // Filter to only entries with 2+ keys (actual duplicates)
    let mut duplicates: Vec<_> = value_to_keys
        .into_iter()
        .filter(|(_, keys)| keys.len() >= 2)
        .collect();

    if duplicates.is_empty() {
        println!();
        println!("{} No duplicate translations found", "✓".green());
        return Ok(());
    }

    duplicates.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

    let total_duplicate_keys: usize = duplicates.iter().map(|(_, keys)| keys.len()).sum();
    let potential_savings = total_duplicate_keys - duplicates.len();

    println!();
    println!(
        "{} Found {} duplicate group{}:",
        "📋".blue(),
        duplicates.len(),
        if duplicates.len() == 1 { "" } else { "s" }
    );
    println!();

    for (values, mut keys) in duplicates {
        keys.sort();

        let display: Vec<String> = values
            .iter()
            .map(|(_, value)| truncate_for_display(value, 40))
            .collect();

        println!("  {} {}:", "•".green(), display.join(" / "));
        for key in &keys {
            println!("    - {}", key.yellow());
        }
        println!();
    }

    println!(
        "{} {} keys could potentially be consolidated into {}",
        "✓".green(),
        total_duplicate_keys,
        total_duplicate_keys - potential_savings
    );

    Ok(())
}

fn handle_duplicates_cross_file(
    files: &std::collections::HashMap<String, FileConfig>,
    languages: &[String],
) -> Result<()> {
    use std::collections::HashMap;

    println!(
        "{} Scanning for duplicate translations across all files...",
        "🔍".blue()
    );
    println!();

    // Build a map: sorted values -> Vec<(file_shortcut, key)>
    let mut value_to_keys: HashMap<Vec<(String, String)>, Vec<(String, String)>> = HashMap::new();
    let mut files_processed = 0;
    let mut total_keys = 0;

    for (shortcut, file_config) in files {
        if !file_config.path.exists() {
            println!("  {} Skipping {} (file not found)", "⚠".yellow(), shortcut);
            continue;
        }

        let parse_result = parse_i18n_file_with_record_name(
            &file_config.path,
            &file_config.record_name,
            languages,
        )?;
        files_processed += 1;

        for (key, translation) in &parse_result.translations {
            if translation.is_function {
                continue;
            }

            total_keys += 1;
            let mut sorted_values: Vec<(String, String)> = translation
                .values
                .iter()
                .map(|(lang, value)| (lang.clone(), value.clone()))
                .collect();
            sorted_values.sort();
            value_to_keys
                .entry(sorted_values)
                .or_default()
                .push((shortcut.clone(), key.clone()));
        }
    }

    println!(
        "  Processed {} files with {} translation keys",
        files_processed, total_keys
    );
    println!();

    // Filter to entries that span multiple files
    let cross_file_duplicates: Vec<_> = value_to_keys
        .into_iter()
        .filter(|(_, keys)| {
            let unique_files: std::collections::HashSet<_> = keys.iter().map(|(f, _)| f).collect();
            unique_files.len() > 1
        })
        .collect();

    if cross_file_duplicates.is_empty() {
        println!("{} No cross-file duplicate translations found", "✓".green());
        return Ok(());
    }

    let mut duplicates = cross_file_duplicates;
    duplicates.sort_by(|a, b| {
        let a_files: std::collections::HashSet<_> = a.1.iter().map(|(f, _)| f).collect();
        let b_files: std::collections::HashSet<_> = b.1.iter().map(|(f, _)| f).collect();
        b_files
            .len()
            .cmp(&a_files.len())
            .then_with(|| b.1.len().cmp(&a.1.len()))
            .then_with(|| a.0.cmp(&b.0))
    });

    let total_duplicate_keys: usize = duplicates.iter().map(|(_, keys)| keys.len()).sum();

    println!(
        "{} Found {} cross-file duplicate group{}:",
        "📋".blue(),
        duplicates.len(),
        if duplicates.len() == 1 { "" } else { "s" }
    );
    println!();

    for (values, mut keys) in duplicates {
        keys.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let display: Vec<String> = values
            .iter()
            .map(|(_, value)| truncate_for_display(value, 40))
            .collect();

        let mut current_file = String::new();
        println!("  {} {}:", "•".green(), display.join(" / "));
        for (file_shortcut, key) in &keys {
            if file_shortcut != &current_file {
                current_file = file_shortcut.clone();
                println!("    [{}]", file_shortcut.cyan());
            }
            println!("      - {}", key.yellow());
        }
        println!();
    }

    println!(
        "{} {} keys across files share the same translations",
        "✓".green(),
        total_duplicate_keys
    );
    println!("   Consider consolidating into a shared I18n module");

    Ok(())
}

fn handle_shared_values(
    file: &PathBuf,
    record_name: &str,
    languages: &[String],
    suppress: bool,
) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    println!(
        "{} Scanning for values shared by multiple languages within the same key...",
        "🔍".blue()
    );

    let parse_result = parse_i18n_file_with_record_name(file, record_name, languages)?;
    let findings = find_keys_with_shared_language_values(&parse_result.translations, languages);
    let suppressed_path = suppressed_entries_path();
    let suppressions = load_suppressed_entries(&suppressed_path)?;
    let (visible_findings, suppressed_groups) =
        filter_suppressed_shared_values(file, findings, &suppressions);

    if suppress {
        suppress_shared_values(
            &suppressed_path,
            collect_shared_value_suppressions(file, &visible_findings),
            suppressed_groups,
        )?;
        return Ok(());
    }

    print_shared_value_findings(&visible_findings, suppressed_groups);

    Ok(())
}

fn handle_shared_values_cross_file(
    files: &std::collections::HashMap<String, FileConfig>,
    languages: &[String],
    suppress: bool,
) -> Result<()> {
    println!(
        "{} Scanning for values shared by multiple languages within the same key across all files...",
        "🔍".blue()
    );
    println!();

    let suppressed_path = suppressed_entries_path();
    let suppressions = load_suppressed_entries(&suppressed_path)?;
    let mut all_findings = Vec::new();
    let mut suppressed_groups = 0;
    let mut files_processed = 0;

    for (shortcut, file_config) in files {
        if !file_config.path.exists() {
            println!("  {} Skipping {} (file not found)", "⚠".yellow(), shortcut);
            continue;
        }

        let parse_result = parse_i18n_file_with_record_name(
            &file_config.path,
            &file_config.record_name,
            languages,
        )?;
        files_processed += 1;

        let findings = find_keys_with_shared_language_values(&parse_result.translations, languages);
        let (visible_findings, file_suppressed_groups) =
            filter_suppressed_shared_values(&file_config.path, findings, &suppressions);
        suppressed_groups += file_suppressed_groups;

        all_findings.extend(visible_findings.into_iter().map(|entry| {
            FileKeySharedLanguageValues {
                file_shortcut: shortcut.clone(),
                file_path: file_config.path.clone(),
                key: entry.key,
                groups: entry.groups,
            }
        }));
    }

    println!("  Processed {} files", files_processed);
    println!();

    all_findings.sort_by(|a, b| {
        a.file_shortcut
            .cmp(&b.file_shortcut)
            .then_with(|| a.key.cmp(&b.key))
    });

    if suppress {
        let entries = collect_cross_file_shared_value_suppressions(&all_findings);
        suppress_shared_values(&suppressed_path, entries, suppressed_groups)?;
        return Ok(());
    }

    print_cross_file_shared_value_findings(&all_findings, suppressed_groups);

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SharedLanguageValueGroup {
    value: String,
    languages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeySharedLanguageValues {
    key: String,
    groups: Vec<SharedLanguageValueGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileKeySharedLanguageValues {
    file_shortcut: String,
    file_path: PathBuf,
    key: String,
    groups: Vec<SharedLanguageValueGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct SuppressedEntry {
    check: String,
    file_path: String,
    key: String,
    languages: Vec<String>,
    value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct SuppressedStore {
    #[serde(default)]
    entries: Vec<SuppressedEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LocalStateConfig {
    #[serde(default = "default_local_state_version")]
    version: u32,
}

impl Default for LocalStateConfig {
    fn default() -> Self {
        Self {
            version: default_local_state_version(),
        }
    }
}

fn default_local_state_version() -> u32 {
    1
}

fn find_keys_with_shared_language_values(
    translations: &std::collections::HashMap<String, Translation>,
    languages: &[String],
) -> Vec<KeySharedLanguageValues> {
    let mut keys_with_shared_values = Vec::new();

    for (key, translation) in translations {
        let groups = find_shared_language_value_groups(&translation.values, languages);
        if !groups.is_empty() {
            keys_with_shared_values.push(KeySharedLanguageValues {
                key: key.clone(),
                groups,
            });
        }
    }

    keys_with_shared_values.sort_by(|a, b| a.key.cmp(&b.key));
    keys_with_shared_values
}

fn find_shared_language_value_groups(
    values: &std::collections::HashMap<String, String>,
    languages: &[String],
) -> Vec<SharedLanguageValueGroup> {
    let mut value_to_languages: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (lang, value) in values {
        if value.trim().is_empty() {
            continue;
        }

        value_to_languages
            .entry(value.clone())
            .or_default()
            .push(lang.clone());
    }

    let mut groups = Vec::new();

    for (value, mut langs) in value_to_languages {
        if langs.len() < 2 {
            continue;
        }

        langs.sort_by(|a, b| {
            language_sort_index(a, languages)
                .cmp(&language_sort_index(b, languages))
                .then_with(|| a.cmp(b))
        });

        groups.push(SharedLanguageValueGroup {
            value,
            languages: langs,
        });
    }

    groups.sort_by(|a, b| {
        b.languages
            .len()
            .cmp(&a.languages.len())
            .then_with(|| a.value.cmp(&b.value))
            .then_with(|| a.languages.cmp(&b.languages))
    });

    groups
}

fn language_sort_index(lang: &str, languages: &[String]) -> usize {
    languages
        .iter()
        .position(|configured| configured == lang)
        .unwrap_or(usize::MAX)
}

fn format_language_codes(languages: &[String]) -> String {
    languages
        .iter()
        .map(|lang| lang.to_uppercase())
        .collect::<Vec<_>>()
        .join(", ")
}

fn truncate_for_display(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let truncated: String = value.chars().take(max_chars - 3).collect();
    format!("{}...", truncated)
}

fn compact_value_for_display(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn shared_values_summary(total_visible_groups: usize) -> String {
    match total_visible_groups {
        0 => "I found no errors!".to_string(),
        1 => "I found 1 error!".to_string(),
        n => format!("I found {} errors!", n),
    }
}

fn suppressed_errors_summary(suppressed_groups: usize) -> String {
    match suppressed_groups {
        0 => String::new(),
        1 => "There is still 1 suppressed error.".to_string(),
        n => format!("There are still {} suppressed errors.", n),
    }
}

fn local_state_config_path() -> PathBuf {
    PathBuf::from(LOCAL_CONFIG_FILE)
}

fn suppressed_entries_path() -> PathBuf {
    PathBuf::from(LOCAL_SUPPRESSED_FILE)
}

fn format_local_path(path: &Path) -> String {
    format!("./{}", path.display())
}

fn ensure_local_state_config(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    if path.exists() {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let _: LocalStateConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        return Ok(());
    }

    let content = serde_json::to_string_pretty(&LocalStateConfig::default())
        .context("Failed to serialize local state config")?;
    std::fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn load_suppressed_entries(path: &Path) -> Result<SuppressedStore> {
    if !path.exists() {
        return Ok(SuppressedStore::default());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut store: SuppressedStore = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    normalize_suppressed_entries(&mut store);
    Ok(store)
}

fn save_suppressed_entries(path: &Path, store: &SuppressedStore) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let content =
        serde_json::to_string_pretty(store).context("Failed to serialize suppressed entries")?;
    std::fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn normalize_suppressed_entries(store: &mut SuppressedStore) {
    for entry in &mut store.entries {
        entry.languages.sort();
    }

    store.entries.sort_by(|a, b| {
        a.check
            .cmp(&b.check)
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.key.cmp(&b.key))
            .then_with(|| a.languages.cmp(&b.languages))
            .then_with(|| a.value.cmp(&b.value))
    });
    store.entries.dedup();
}

fn normalize_file_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_shared_values_suppressed(
    suppressions: &SuppressedStore,
    file_path: &str,
    key: &str,
    _group: &SharedLanguageValueGroup,
) -> bool {
    suppressions.entries.iter().any(|entry| {
        entry.check == SHARED_VALUES_CHECK_NAME && entry.file_path == file_path && entry.key == key
    })
}

fn filter_suppressed_shared_values(
    file: &Path,
    findings: Vec<KeySharedLanguageValues>,
    suppressions: &SuppressedStore,
) -> (Vec<KeySharedLanguageValues>, usize) {
    let file_path = normalize_file_path(file);
    let mut filtered_findings = Vec::new();
    let mut suppressed_groups = 0;

    for finding in findings {
        let KeySharedLanguageValues { key, groups } = finding;
        let mut visible_groups = Vec::new();

        for group in groups {
            if is_shared_values_suppressed(suppressions, &file_path, &key, &group) {
                suppressed_groups += 1;
            } else {
                visible_groups.push(group);
            }
        }

        if !visible_groups.is_empty() {
            filtered_findings.push(KeySharedLanguageValues {
                key,
                groups: visible_groups,
            });
        }
    }

    (filtered_findings, suppressed_groups)
}

fn collect_shared_value_suppressions(
    file: &Path,
    findings: &[KeySharedLanguageValues],
) -> Vec<SuppressedEntry> {
    let file_path = normalize_file_path(file);
    let mut entries = Vec::new();

    for finding in findings {
        for group in &finding.groups {
            entries.push(SuppressedEntry {
                check: SHARED_VALUES_CHECK_NAME.to_string(),
                file_path: file_path.clone(),
                key: finding.key.clone(),
                languages: group.languages.clone(),
                value: group.value.clone(),
            });
        }
    }

    entries
}

fn collect_cross_file_shared_value_suppressions(
    findings: &[FileKeySharedLanguageValues],
) -> Vec<SuppressedEntry> {
    let mut entries = Vec::new();

    for finding in findings {
        for group in &finding.groups {
            entries.push(SuppressedEntry {
                check: SHARED_VALUES_CHECK_NAME.to_string(),
                file_path: normalize_file_path(&finding.file_path),
                key: finding.key.clone(),
                languages: group.languages.clone(),
                value: group.value.clone(),
            });
        }
    }

    entries
}

fn suppress_shared_values(
    suppressed_path: &Path,
    new_entries: Vec<SuppressedEntry>,
    already_suppressed_groups: usize,
) -> Result<()> {
    let config_path = local_state_config_path();
    ensure_local_state_config(&config_path)?;

    if new_entries.is_empty() {
        println!("{} No new shared-value findings to suppress", "✓".green());
        if already_suppressed_groups > 0 {
            println!("{}", suppressed_errors_summary(already_suppressed_groups));
        }
        return Ok(());
    }

    let mut store = load_suppressed_entries(suppressed_path)?;
    let mut existing_entries: std::collections::HashSet<_> =
        store.entries.iter().cloned().collect();
    let mut added_entries = 0;

    for entry in new_entries {
        if existing_entries.insert(entry.clone()) {
            store.entries.push(entry);
            added_entries += 1;
        }
    }

    normalize_suppressed_entries(&mut store);
    save_suppressed_entries(suppressed_path, &store)?;

    println!(
        "{} Suppressed {} error{} in {}",
        "✓".green(),
        added_entries,
        if added_entries == 1 { "" } else { "s" },
        format_local_path(suppressed_path).cyan()
    );
    println!(
        "{} Local state config is stored in {}",
        "ℹ".blue(),
        format_local_path(&config_path).cyan()
    );
    if already_suppressed_groups > 0 {
        println!("{}", suppressed_errors_summary(already_suppressed_groups));
    }

    Ok(())
}

fn print_shared_value_findings(findings: &[KeySharedLanguageValues], suppressed_groups: usize) {
    let total_groups: usize = findings.iter().map(|entry| entry.groups.len()).sum();

    println!();
    println!("{}", shared_values_summary(total_groups));
    if suppressed_groups > 0 {
        println!();
        println!("{}", suppressed_errors_summary(suppressed_groups));
    }

    if findings.is_empty() {
        return;
    }

    println!();

    for entry in findings {
        println!("  {} {}:", "•".green(), entry.key.yellow());
        for group in &entry.groups {
            println!(
                "    - {}: {}",
                format_language_codes(&group.languages).cyan(),
                truncate_for_display(&compact_value_for_display(&group.value), 50)
            );
        }
        println!();
    }
}

fn print_cross_file_shared_value_findings(
    findings: &[FileKeySharedLanguageValues],
    suppressed_groups: usize,
) {
    let total_groups: usize = findings.iter().map(|entry| entry.groups.len()).sum();

    println!("{}", shared_values_summary(total_groups));
    if suppressed_groups > 0 {
        println!();
        println!("{}", suppressed_errors_summary(suppressed_groups));
    }

    if findings.is_empty() {
        return;
    }

    println!();

    for entry in findings {
        println!(
            "  {} [{}] {}:",
            "•".green(),
            entry.file_shortcut.cyan(),
            entry.key.yellow()
        );
        for group in &entry.groups {
            println!(
                "    - {}: {}",
                format_language_codes(&group.languages).cyan(),
                truncate_for_display(&compact_value_for_display(&group.value), 50)
            );
        }
        println!();
    }
}

/// Handle the modify command: update specific language values for an existing key
fn handle_modify(
    file: &PathBuf,
    key: &str,
    values: &std::collections::HashMap<String, String>,
    record_name: &str,
    languages: &[String],
) -> Result<()> {
    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    // Check if key exists
    match check_key_exists_with_record_name(file, key, record_name, languages)? {
        Some(existing) => {
            // Parse the file to find field locations
            let content = std::fs::read_to_string(file)?;
            let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let parse_result = parse_i18n_file_with_record_name(file, record_name, languages)?;

            // For each language we want to modify
            for (lang, new_value) in values {
                // Find the language record bounds
                if let Some((_, start, end)) =
                    parse_result.lang_bounds.iter().find(|(l, _, _)| l == lang)
                {
                    // Find the field within this language record
                    let is_function = existing.is_function;
                    let mut field_start = None;
                    let mut field_end = None;

                    let field_regex =
                        regex::Regex::new(&format!(r"^\s*,?\s*{}\s*=", regex::escape(key)))?;
                    let next_field_regex = regex::Regex::new(r"^\s*,?\s*\w+\s*=")?;

                    let mut i = *start + 1;
                    while i <= *end {
                        if field_regex.is_match(&lines[i]) {
                            field_start = Some(i);
                            // Find the end of this field
                            if is_function {
                                let mut j = i + 1;
                                while j <= *end {
                                    let line = &lines[j];
                                    let trimmed = line.trim();
                                    if trimmed.starts_with('}') || next_field_regex.is_match(line) {
                                        break;
                                    }
                                    j += 1;
                                }
                                field_end = Some(j - 1);
                            } else {
                                field_end = Some(i);
                            }
                            break;
                        }
                        i += 1;
                    }

                    if let (Some(fs), Some(fe)) = (field_start, field_end) {
                        // Detect if it's the first field (uses { key = instead of , key =)
                        let is_first = lines[fs].trim_start().starts_with('{');
                        let prefix = if is_first { "    { " } else { "    , " };

                        // Remove old field lines
                        for _ in fs..=fe {
                            lines.remove(fs);
                        }

                        // Insert new field
                        if is_function {
                            let new_lines: Vec<String> =
                                format!("{}{} = {}", prefix, key, new_value)
                                    .lines()
                                    .map(|l| l.to_string())
                                    .collect();
                            for (idx, line) in new_lines.iter().enumerate() {
                                lines.insert(fs + idx, line.clone());
                            }
                        } else {
                            let escaped = new_value
                                .replace('\\', "\\\\")
                                .replace('"', "\\\"")
                                .replace('\n', "\\n");
                            lines.insert(fs, format!("{}{}= \"{}\"", prefix, key, escaped));
                        }
                    }
                }
            }

            // Write back
            let new_content = lines.join("\n");
            std::fs::write(file, new_content)?;

            println!(
                "{} Modified translation '{}' in {}",
                "✓".green(),
                key.yellow(),
                file.display()
            );
            for (lang, val) in values {
                let display_val = if val.len() > 60 {
                    format!("{}...", &val[..57])
                } else {
                    val.clone()
                };
                println!("  {}: {}", lang.to_uppercase().green(), display_val);
            }
        }
        None => {
            eprintln!(
                "{} Translation '{}' not found in {}",
                "✗".red(),
                key.yellow(),
                file.display()
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle the modify-bulk command: update all translations for one language from a JSON file
fn handle_modify_bulk(
    file: &PathBuf,
    lang: &str,
    json_file: &PathBuf,
    record_name: &str,
    languages: &[String],
) -> Result<()> {
    use std::collections::HashMap;

    if !file.exists() {
        eprintln!("{} File not found: {}", "✗".red(), file.display());
        std::process::exit(1);
    }

    if !json_file.exists() {
        eprintln!("{} JSON file not found: {}", "✗".red(), json_file.display());
        std::process::exit(1);
    }

    let lang = lang.to_lowercase();
    if !languages.contains(&lang) {
        eprintln!(
            "{} Language '{}' is not in configured languages: {}",
            "✗".red(),
            lang.yellow(),
            languages.join(", ")
        );
        std::process::exit(1);
    }

    // Read the JSON translations
    let json_content = std::fs::read_to_string(json_file)?;
    let translations_map: HashMap<String, String> = serde_json::from_str(&json_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON file {}: {}", json_file.display(), e))?;

    if translations_map.is_empty() {
        println!("{} No translations in JSON file", "ℹ".blue());
        return Ok(());
    }

    println!(
        "{} Applying {} translations for '{}' to {}...",
        "→".cyan(),
        translations_map.len(),
        lang.to_uppercase().yellow(),
        file.display()
    );

    // Parse the file to find the language record
    let parse_result = parse_i18n_file_with_record_name(file, record_name, languages)?;
    let content = std::fs::read_to_string(file)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Find the target language record bounds
    let (_, lang_start, lang_end) = parse_result
        .lang_bounds
        .iter()
        .find(|(l, _, _)| *l == lang)
        .ok_or_else(|| anyhow::anyhow!("Language '{}' record not found in file", lang))?;

    let field_regex = regex::Regex::new(r"^\s*[,{]\s*(\w+)\s*=")?;
    let mut modified = 0;
    let mut skipped = 0;

    // Iterate through the language record and replace values
    let mut i = *lang_start + 1;
    while i < *lang_end {
        if let Some(captures) = field_regex.captures(&lines[i].clone()) {
            let key = captures[1].to_string();

            if let Some(new_value) = translations_map.get(&key) {
                // Check if this is a function (multiline) translation
                let is_function = parse_result
                    .translations
                    .get(&key)
                    .map(|t| t.is_function)
                    .unwrap_or(false);

                if is_function {
                    // Skip function translations in bulk mode (too complex for JSON)
                    skipped += 1;
                    i += 1;
                    continue;
                }

                // Detect prefix (first field uses "{ ", others use ", ")
                let line = &lines[i];
                let prefix = if line.trim_start().starts_with('{') {
                    "    { "
                } else {
                    "    , "
                };

                // Replace the line with the new value
                // Preserve Elm escape sequences (\n, \t, \r, \\) while escaping other chars
                let escaped = new_value
                    .replace("\\\\", "\x00BACKSLASH\x00") // Protect existing \\
                    .replace("\\n", "\x00NEWLINE\x00") // Protect \n
                    .replace("\\t", "\x00TAB\x00") // Protect \t
                    .replace("\\r", "\x00CR\x00") // Protect \r
                    .replace("\\\"", "\x00QUOTE\x00") // Protect \"
                    .replace('\\', "\\\\") // Escape remaining backslashes
                    .replace('"', "\\\"") // Escape quotes
                    .replace('\n', "\\n") // Escape actual newlines
                    .replace("\x00BACKSLASH\x00", "\\\\") // Restore \\
                    .replace("\x00NEWLINE\x00", "\\n") // Restore \n
                    .replace("\x00TAB\x00", "\\t") // Restore \t
                    .replace("\x00CR\x00", "\\r") // Restore \r
                    .replace("\x00QUOTE\x00", "\\\""); // Restore \"
                lines[i] = format!("{}{} = \"{}\"", prefix, key, escaped);
                modified += 1;
            }
        }
        i += 1;
    }

    // Write back
    let new_content = lines.join("\n");
    std::fs::write(file, new_content)?;

    println!(
        "{} Modified {} translations, skipped {} function translations",
        "✓".green(),
        modified.to_string().yellow(),
        skipped
    );

    Ok(())
}

/// Handle the add-language command: add a new language by duplicating an existing one
fn handle_add_language(config: &Config, new_lang: &str, from_lang: &str) -> Result<()> {
    use std::fs;

    let new_lang = new_lang.to_lowercase();
    let from_lang = from_lang.to_lowercase();
    let languages = config.languages();

    // Validate
    if !languages.contains(&from_lang) {
        eprintln!(
            "{} Source language '{}' is not configured. Available: {}",
            "✗".red(),
            from_lang.yellow(),
            languages.join(", ")
        );
        std::process::exit(1);
    }
    if languages.contains(&new_lang) {
        eprintln!(
            "{} Language '{}' already exists in configuration",
            "✗".red(),
            new_lang.yellow()
        );
        std::process::exit(1);
    }

    fn capitalize_first(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    // Get all translation files to process
    let files_to_process: Vec<(PathBuf, String)> = match config {
        Config::SingleFile {
            file, record_name, ..
        } => {
            vec![(file.clone(), record_name.clone())]
        }
        Config::MultiFile { files, .. } => files
            .values()
            .map(|fc| (fc.path.clone(), fc.record_name.clone()))
            .collect(),
    };

    // Process each file
    for (file_path, record_name) in &files_to_process {
        if !file_path.exists() {
            println!(
                "  {} Skipping {} (file not found)",
                "⚠".yellow(),
                file_path.display()
            );
            continue;
        }

        println!("{} Processing {}...", "→".cyan(), file_path.display());

        let content = fs::read_to_string(file_path)?;
        let mut new_content = content.clone();

        // 1. Add new variant to Language type
        let from_upper = from_lang.to_uppercase();
        let new_upper = new_lang.to_uppercase();
        // Find the last language variant and add after it
        if let Some(pos) = new_content.find(&format!("| {}\n", from_upper)) {
            let insert_pos = pos + format!("| {}\n", from_upper).len();
            new_content.insert_str(insert_pos, &format!("    | {}\n", new_upper));
        } else if let Some(pos) = new_content.find(&format!("= {}\n", from_upper)) {
            let insert_pos = pos + format!("= {}\n", from_upper).len();
            new_content.insert_str(insert_pos, &format!("    | {}\n", new_upper));
        } else {
            // Add after the last variant we can find
            let mut last_variant_end = None;
            for lang in languages {
                let upper = lang.to_uppercase();
                if let Some(pos) = new_content.find(&format!("| {}\n", upper)) {
                    let end = pos + format!("| {}\n", upper).len();
                    last_variant_end = Some(end);
                } else if let Some(pos) = new_content.find(&format!("= {}\n", upper)) {
                    let end = pos + format!("= {}\n", upper).len();
                    last_variant_end = Some(end);
                }
            }
            if let Some(pos) = last_variant_end {
                new_content.insert_str(pos, &format!("    | {}\n", new_upper));
            }
        }

        // 2. Duplicate the source language's translation record
        let from_cap = capitalize_first(&from_lang);
        let new_cap = capitalize_first(&new_lang);
        let from_fn_name = format!("translations{}", from_cap);
        let new_fn_name = format!("translations{}", new_cap);

        // Find the source translation record (type annotation + implementation)
        if let Some(type_start) = new_content.find(&format!("{} : {}", from_fn_name, record_name)) {
            // Find the end of the record (closing brace followed by blank line or next definition)
            let after_type = &new_content[type_start..];
            if let Some(brace_pos) = find_closing_brace(after_type) {
                let record_end = type_start + brace_pos + 1;
                let record_text = &new_content[type_start..record_end];

                // Create the new record by replacing the function name
                let new_record = record_text.replace(&from_fn_name, &new_fn_name);

                // Insert after the source record (with spacing)
                let insert_text = format!("\n\n{}", new_record);
                new_content.insert_str(record_end, &insert_text);
            }
        }

        // 3. Update languageToString: add new case
        let lang_to_str_case = format!("        {} ->\n            \"{}\"", new_upper, new_lang);
        // Try to insert after the last existing case before the function ends
        if let Some(pos) = new_content.find(&format!(
            "        {} ->\n            \"{}\"",
            from_upper, from_lang
        )) {
            let case_end =
                pos + format!("        {} ->\n            \"{}\"", from_upper, from_lang).len();
            new_content.insert_str(case_end, &format!("\n\n{}", lang_to_str_case));
        } else {
            // from_lang might not have an explicit case; find the last explicit case in languageToString
            // Insert before the closing of the function by finding the last case branch
            let mut last_case_end = None;
            for lang in languages {
                let upper = lang.to_uppercase();
                let pattern = format!("        {} ->\n            \"{}\"", upper, lang);
                if let Some(pos) = new_content.find(&pattern) {
                    let end = pos + pattern.len();
                    if last_case_end.map_or(true, |prev| end > prev) {
                        last_case_end = Some(end);
                    }
                }
            }
            if let Some(end) = last_case_end {
                new_content.insert_str(end, &format!("\n\n{}", lang_to_str_case));
            }
        }

        // 4. Update stringToLanguage: add new case before the default (_ ->) case
        let str_to_lang_case = format!("        \"{}\" ->\n            {}", new_lang, new_upper);
        if let Some(pos) = new_content.find(&format!(
            "        \"{}\" ->\n            {}",
            from_lang, from_upper
        )) {
            let case_end =
                pos + format!("        \"{}\" ->\n            {}", from_lang, from_upper).len();
            new_content.insert_str(case_end, &format!("\n\n{}", str_to_lang_case));
        } else {
            // from_lang is likely the default case (_ -> FROM_UPPER), insert before it
            if let Some(pos) = new_content.find("        _ ->\n") {
                // Find the stringToLanguage function context by checking we're in the right function
                new_content.insert_str(pos, &format!("{}\n\n", str_to_lang_case));
            }
        }

        // 5. Update translations function: add new case
        let translations_case = format!("        {} ->\n            {}", new_upper, new_fn_name);
        if let Some(pos) = new_content.find(&format!(
            "        {} ->\n            {}",
            from_upper, from_fn_name
        )) {
            let case_end =
                pos + format!("        {} ->\n            {}", from_upper, from_fn_name).len();
            new_content.insert_str(case_end, &format!("\n\n{}", translations_case));
        } else {
            // from_lang is the default; find the last explicit case in translations function
            let mut last_case_end = None;
            for lang in languages {
                let upper = lang.to_uppercase();
                let cap = capitalize_first(lang);
                let fn_name = format!("translations{}", cap);
                let pattern = format!("        {} ->\n            {}", upper, fn_name);
                if let Some(pos) = new_content.find(&pattern) {
                    let end = pos + pattern.len();
                    if last_case_end.map_or(true, |prev| end > prev) {
                        last_case_end = Some(end);
                    }
                }
            }
            if let Some(end) = last_case_end {
                new_content.insert_str(end, &format!("\n\n{}", translations_case));
            }
        }

        fs::write(file_path, new_content)?;
        println!(
            "  {} Added language '{}' (copied from '{}')",
            "✓".green(),
            new_lang.yellow(),
            from_lang
        );
    }

    // Update the config
    let mut updated_config = config.clone();
    match &mut updated_config {
        Config::SingleFile { languages, .. } => languages.push(new_lang.clone()),
        Config::MultiFile { languages, .. } => languages.push(new_lang.clone()),
    }
    updated_config.save()?;

    println!();
    println!(
        "{} Language '{}' added successfully!",
        "✓".green(),
        new_lang.yellow()
    );
    println!(
        "{} All values are duplicated from '{}' — update them with the actual translations.",
        "ℹ".blue(),
        from_lang
    );

    Ok(())
}

/// Find the position of the closing brace that ends a record definition
fn find_closing_brace(text: &str) -> Option<usize> {
    let mut brace_count = 0;
    let mut found_open = false;
    for (i, c) in text.char_indices() {
        if c == '{' {
            brace_count += 1;
            found_open = true;
        } else if c == '}' {
            brace_count -= 1;
            if found_open && brace_count == 0 {
                return Some(i);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn finds_shared_language_value_groups() {
        let languages = vec![
            "en".to_string(),
            "fr".to_string(),
            "es".to_string(),
            "de".to_string(),
        ];

        let values = HashMap::from([
            ("fr".to_string(), "\"Brand\"".to_string()),
            ("de".to_string(), "\"Hola\"".to_string()),
            ("en".to_string(), "\"Brand\"".to_string()),
            ("es".to_string(), "\"Hola\"".to_string()),
        ]);

        let groups = find_shared_language_value_groups(&values, &languages);

        assert_eq!(
            groups,
            vec![
                SharedLanguageValueGroup {
                    value: "\"Brand\"".to_string(),
                    languages: vec!["en".to_string(), "fr".to_string()],
                },
                SharedLanguageValueGroup {
                    value: "\"Hola\"".to_string(),
                    languages: vec!["es".to_string(), "de".to_string()],
                },
            ]
        );
    }

    #[test]
    fn finds_shared_values_for_functions_and_ignores_missing_values() {
        let languages = vec!["en".to_string(), "fr".to_string(), "es".to_string()];
        let mut translations = HashMap::new();

        translations.insert(
            "brandName".to_string(),
            Translation {
                key: "brandName".to_string(),
                values: HashMap::from([
                    ("en".to_string(), "\"Cleemo\"".to_string()),
                    ("fr".to_string(), "\"Cleemo\"".to_string()),
                    ("es".to_string(), "\"Cleemo ES\"".to_string()),
                ]),
                is_function: false,
                type_signature: None,
            },
        );
        translations.insert(
            "welcome".to_string(),
            Translation {
                key: "welcome".to_string(),
                values: HashMap::from([
                    ("en".to_string(), "\"Welcome\"".to_string()),
                    ("fr".to_string(), "\"Bienvenue\"".to_string()),
                    ("es".to_string(), "\"Hola\"".to_string()),
                ]),
                is_function: false,
                type_signature: None,
            },
        );
        translations.insert(
            "formatDate".to_string(),
            Translation {
                key: "formatDate".to_string(),
                values: HashMap::from([
                    ("en".to_string(), "\\\\d -> format d".to_string()),
                    ("fr".to_string(), "\\\\d -> format d".to_string()),
                    ("es".to_string(), "\\\\d -> format d".to_string()),
                ]),
                is_function: true,
                type_signature: Some("Date -> String".to_string()),
            },
        );
        translations.insert(
            "missing".to_string(),
            Translation {
                key: "missing".to_string(),
                values: HashMap::from([
                    ("en".to_string(), "".to_string()),
                    ("fr".to_string(), "".to_string()),
                    ("es".to_string(), "\"Disponible\"".to_string()),
                ]),
                is_function: false,
                type_signature: None,
            },
        );

        let keys = find_keys_with_shared_language_values(&translations, &languages);

        assert_eq!(
            keys,
            vec![
                KeySharedLanguageValues {
                    key: "brandName".to_string(),
                    groups: vec![SharedLanguageValueGroup {
                        value: "\"Cleemo\"".to_string(),
                        languages: vec!["en".to_string(), "fr".to_string()],
                    }],
                },
                KeySharedLanguageValues {
                    key: "formatDate".to_string(),
                    groups: vec![SharedLanguageValueGroup {
                        value: "\\\\d -> format d".to_string(),
                        languages: vec!["en".to_string(), "fr".to_string(), "es".to_string()],
                    }],
                },
            ]
        );
    }

    #[test]
    fn finds_shared_values_for_anonymous_functions_from_parsed_file() {
        let temp_dir = TempDir::new().unwrap();
        let i18n_file = temp_dir.path().join("I18n.elm");
        let languages = vec!["en".to_string(), "fr".to_string(), "es".to_string()];

        std::fs::write(
            &i18n_file,
            r#"module I18n exposing (..)

type Language
    = EN
    | FR
    | ES

type Status
    = Active
    | Inactive

type alias Translations =
    { statusMessage : Status -> String
    }

translationsEn : Translations
translationsEn =
    { statusMessage = \status -> case status of
            Active -> "Active"
            Inactive -> "Inactive"
    }

translationsFr : Translations
translationsFr =
    { statusMessage = \status -> case status of
            Active -> "Active"
            Inactive -> "Inactive"
    }

translationsEs : Translations
translationsEs =
    { statusMessage = \status -> case status of
            Active -> "Activo"
            Inactive -> "Inactivo"
    }
"#,
        )
        .unwrap();

        let parse_result =
            parse_i18n_file_with_record_name(&i18n_file, "Translations", &languages).unwrap();
        let keys = find_keys_with_shared_language_values(&parse_result.translations, &languages);

        assert_eq!(
            keys,
            vec![KeySharedLanguageValues {
                key: "statusMessage".to_string(),
                groups: vec![SharedLanguageValueGroup {
                    value: r#"\status -> case status of
        Active -> "Active"
        Inactive -> "Inactive""#
                        .to_string(),
                    languages: vec!["en".to_string(), "fr".to_string()],
                }],
            }]
        );
    }

    #[test]
    fn suppresses_all_groups_for_a_suppressed_key() {
        let findings = vec![KeySharedLanguageValues {
            key: "brandName".to_string(),
            groups: vec![
                SharedLanguageValueGroup {
                    value: "\"Cleemo\"".to_string(),
                    languages: vec!["en".to_string(), "fr".to_string()],
                },
                SharedLanguageValueGroup {
                    value: "\"Brand\"".to_string(),
                    languages: vec!["es".to_string(), "pt".to_string()],
                },
            ],
        }];
        let suppressions = SuppressedStore {
            entries: vec![SuppressedEntry {
                check: SHARED_VALUES_CHECK_NAME.to_string(),
                file_path: "src/I18n.elm".to_string(),
                key: "brandName".to_string(),
                languages: vec!["en".to_string(), "fr".to_string()],
                value: "\"Cleemo\"".to_string(),
            }],
        };

        let (filtered, suppressed_groups) =
            filter_suppressed_shared_values(Path::new("src/I18n.elm"), findings, &suppressions);

        // Both groups suppressed because suppress matches by key, not by exact languages/value
        assert_eq!(suppressed_groups, 2);
        assert!(filtered.is_empty());
    }

    #[test]
    fn does_not_suppress_different_key() {
        let findings = vec![KeySharedLanguageValues {
            key: "otherKey".to_string(),
            groups: vec![SharedLanguageValueGroup {
                value: "\"Same\"".to_string(),
                languages: vec!["en".to_string(), "fr".to_string()],
            }],
        }];
        let suppressions = SuppressedStore {
            entries: vec![SuppressedEntry {
                check: SHARED_VALUES_CHECK_NAME.to_string(),
                file_path: "src/I18n.elm".to_string(),
                key: "brandName".to_string(),
                languages: vec!["en".to_string(), "fr".to_string()],
                value: "\"Cleemo\"".to_string(),
            }],
        };

        let (filtered, suppressed_groups) =
            filter_suppressed_shared_values(Path::new("src/I18n.elm"), findings, &suppressions);

        assert_eq!(suppressed_groups, 0);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn saves_and_loads_suppressed_entries() {
        let temp_dir = TempDir::new().unwrap();
        let suppressed_path = temp_dir.path().join("elm-i18n").join("suppressed.json");
        let store = SuppressedStore {
            entries: vec![SuppressedEntry {
                check: SHARED_VALUES_CHECK_NAME.to_string(),
                file_path: "src/I18n.elm".to_string(),
                key: "brandName".to_string(),
                languages: vec!["en".to_string(), "fr".to_string()],
                value: "\"Cleemo\"".to_string(),
            }],
        };

        save_suppressed_entries(&suppressed_path, &store).unwrap();
        let loaded = load_suppressed_entries(&suppressed_path).unwrap();

        assert_eq!(loaded, store);
        assert!(suppressed_path.exists());
    }
}
