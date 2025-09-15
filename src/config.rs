use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use colored::*;

const CONFIG_FILE_NAME: &str = "elm-i18n.json";
const ELM_I18N_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum Config {
    #[serde(rename = "single-file")]
    SingleFile {
        #[serde(rename = "elm-i18n-version")]
        elm_i18n_version: String,
        languages: Vec<String>,
        #[serde(rename = "sourceDir")]
        source_dir: PathBuf,
        file: PathBuf,
        #[serde(rename = "recordName")]
        record_name: String,
    },
    #[serde(rename = "multi-file")]
    MultiFile {
        #[serde(rename = "elm-i18n-version")]
        elm_i18n_version: String,
        languages: Vec<String>,
        #[serde(rename = "sourceDir")]
        source_dir: PathBuf,
        files: HashMap<String, FileConfig>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub path: PathBuf,
    #[serde(rename = "recordName")]
    pub record_name: String,
}

impl Config {
    /// Load config from current directory
    pub fn load() -> Result<Option<Self>> {
        let config_path = Path::new(CONFIG_FILE_NAME);
        
        if !config_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {}", CONFIG_FILE_NAME))?;
            
        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}. Please check the JSON syntax.", CONFIG_FILE_NAME))?;
            
        config.validate()?;
        
        Ok(Some(config))
    }
    
    /// Save config to current directory
    pub fn save(&self) -> Result<()> {
        let config_path = Path::new(CONFIG_FILE_NAME);
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
            
        fs::write(config_path, content)
            .with_context(|| format!("Failed to write {}", CONFIG_FILE_NAME))?;
            
        Ok(())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        match self {
            Config::SingleFile { elm_i18n_version, languages, file, .. } => {
                // Just warn if version is different, don't fail
                let current_version = ELM_I18N_VERSION;
                if !elm_i18n_version.starts_with(&current_version[..3]) {
                    eprintln!("⚠ Config was created with elm-i18n v{}, current version is v{}", 
                        elm_i18n_version.yellow(), current_version.yellow());
                }
                
                if languages.is_empty() {
                    bail!("At least one language must be specified");
                }
                
                if file.to_str().unwrap_or("").is_empty() {
                    bail!("File path cannot be empty");
                }
            }
            Config::MultiFile { elm_i18n_version, languages, files, .. } => {
                // Just warn if version is different, don't fail
                let current_version = ELM_I18N_VERSION;
                if !elm_i18n_version.starts_with(&current_version[..3]) {
                    eprintln!("⚠ Config was created with elm-i18n v{}, current version is v{}", 
                        elm_i18n_version.yellow(), current_version.yellow());
                }
                
                if languages.is_empty() {
                    bail!("At least one language must be specified");
                }
                
                if files.is_empty() {
                    bail!("At least one file must be configured in multi-file mode");
                }
                
                // Validate shortcuts
                for (shortcut, file_config) in files {
                    if shortcut.is_empty() {
                        bail!("File shortcuts cannot be empty");
                    }
                    
                    if shortcut.contains('-') || shortcut.contains(' ') {
                        bail!("File shortcut '{}' contains invalid characters. Use only letters, numbers, and underscores.", shortcut);
                    }
                    
                    if file_config.path.to_str().unwrap_or("").is_empty() {
                        bail!("File path for shortcut '{}' cannot be empty", shortcut);
                    }
                    
                    if file_config.record_name.is_empty() {
                        bail!("Record name for shortcut '{}' cannot be empty", shortcut);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get languages configured
    pub fn languages(&self) -> &Vec<String> {
        match self {
            Config::SingleFile { languages, .. } => languages,
            Config::MultiFile { languages, .. } => languages,
        }
    }
    
    /// Get source directory
    pub fn source_dir(&self) -> &PathBuf {
        match self {
            Config::SingleFile { source_dir, .. } => source_dir,
            Config::MultiFile { source_dir, .. } => source_dir,
        }
    }
    
    /// Get file config for a given shortcut (multi-file mode)
    pub fn get_file_config(&self, shortcut: &str) -> Option<&FileConfig> {
        match self {
            Config::MultiFile { files, .. } => files.get(shortcut),
            _ => None,
        }
    }
    
    /// Get the single file path (single-file mode)
    pub fn get_single_file(&self) -> Option<(&PathBuf, &str)> {
        match self {
            Config::SingleFile { file, record_name, .. } => Some((file, record_name)),
            _ => None,
        }
    }
    
    /// Check if in multi-file mode
    pub fn is_multi_file(&self) -> bool {
        matches!(self, Config::MultiFile { .. })
    }
    
    /// Get all available shortcuts (for help text)
    pub fn get_shortcuts(&self) -> Vec<(String, PathBuf)> {
        match self {
            Config::MultiFile { files, .. } => {
                let mut shortcuts: Vec<_> = files.iter()
                    .map(|(k, v)| (k.clone(), v.path.clone()))
                    .collect();
                shortcuts.sort_by(|a, b| a.0.cmp(&b.0));
                shortcuts
            }
            _ => vec![],
        }
    }
    
    /// Print available shortcuts (for error messages)
    pub fn print_shortcuts(&self) {
        if let Config::MultiFile { files, .. } = self {
            println!("{} Multi-file mode requires a file shortcut.", "Error:".red());
            println!("Available shortcuts:");
            
            let mut shortcuts: Vec<_> = files.iter().collect();
            shortcuts.sort_by(|a, b| a.0.cmp(b.0));
            
            for (shortcut, config) in shortcuts {
                println!("  {} → {}", 
                    format!("--{}", shortcut).yellow(),
                    config.path.display()
                );
            }
            
            println!();
            println!("Example: elm-i18n {} add myKey --en \"...\" --fr \"...\"",
                "--<shortcut>".yellow());
        }
    }
}

/// Create a default single-file configuration
pub fn create_default_single_file_config() -> Config {
    Config::SingleFile {
        elm_i18n_version: ELM_I18N_VERSION.to_string(),
        languages: vec!["en".to_string(), "fr".to_string()],
        source_dir: PathBuf::from("src"),
        file: PathBuf::from("src/I18n.elm"),
        record_name: "Translations".to_string(),
    }
}

/// Create a sample multi-file configuration
pub fn create_sample_multi_file_config() -> Config {
    let mut files = HashMap::new();
    
    files.insert("app".to_string(), FileConfig {
        path: PathBuf::from("src/I18n/App.elm"),
        record_name: "AppTranslations".to_string(),
    });
    
    files.insert("landing".to_string(), FileConfig {
        path: PathBuf::from("src/I18n/LandingPage.elm"),
        record_name: "LandingPageTranslations".to_string(),
    });
    
    Config::MultiFile {
        elm_i18n_version: ELM_I18N_VERSION.to_string(),
        languages: vec!["en".to_string(), "fr".to_string()],
        source_dir: PathBuf::from("src"),
        files,
    }
}

/// Check if config exists
pub fn config_exists() -> bool {
    Path::new(CONFIG_FILE_NAME).exists()
}

/// Prompt user to create config
pub fn prompt_setup_message() {
    eprintln!("{} No elm-i18n.json configuration found.", "✗".red());
    eprintln!();
    eprintln!("Please run {} to create a configuration file.", "elm-i18n setup".yellow());
    eprintln!();
    eprintln!("This will guide you through setting up:");
    eprintln!("  • Single-file or multi-file translation mode");
    eprintln!("  • File paths and shortcuts");
    eprintln!("  • Supported languages");
    eprintln!("  • Source directory for your Elm code");
}