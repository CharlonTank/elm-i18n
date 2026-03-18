# elm-i18n CLI

A command-line tool for managing internationalization (i18n) translations in Elm applications.

## Features

- ✅ Add simple key-value translations
- ✅ Add function translations (with anonymous functions)
- ✅ Check if translations exist
- ✅ Remove translations
- ✅ Remove all unused translations
- ✅ List all translations with filtering
- ✅ Detect keys whose value is identical across multiple languages
- ✅ Initialize new I18n.elm files
- ✅ Shows existing translations when key already exists
- ✅ Maintains proper Elm formatting
- ✅ Creates backups before modifying files
- ✅ **NEW**: Automatically replace hardcoded strings in your codebase

## Installation

### Quick Install (Recommended)

```bash
cd elm-i18n-cli
./install.sh
```

This will:
- Build the tool in release mode
- Install it to `~/.local/bin` or `/usr/local/bin`
- Check if the installation directory is in your PATH
- Provide instructions if you need to update your PATH

### Manual Installation

```bash
# Build in release mode
cargo build --release

# Copy to a directory in your PATH
cp target/release/elm-i18n ~/.local/bin/
# OR
sudo cp target/release/elm-i18n /usr/local/bin/
```

### Using Cargo Install

```bash
cargo install --path .
```

### Uninstall

```bash
./uninstall.sh
```

## Usage

### Setup configuration (Required first step)

```bash
elm-i18n setup
```

This interactive command creates an `elm-i18n/config.json` configuration file. Choose between:
- **Single-file mode**: One I18n.elm file for all translations
- **Multi-file mode**: Separate files for different parts of your app

### Check your configuration

```bash
elm-i18n status
```

Shows your current configuration, available shortcuts, and usage examples.

### Initialize a new I18n module

```bash
elm-i18n init
# Creates src/I18n.elm with English and French support

elm-i18n init --languages en,fr,es
# Creates with custom language support
```

### Add a simple translation

```bash
elm-i18n add welcomeBack --fr "Bon retour" --en "Welcome back"
```

If the key already exists, it will show the current translations:
```
ℹ Translation 'welcome' already exists:
  EN: Welcome to your Lamdera application!
  FR: Bienvenue dans votre application Lamdera!
  
The existing translations might be sufficient. Consider using a different key.
```

### Add a translation and replace hardcoded strings

**NEW**: Use the `--replace` flag to automatically find and replace hardcoded strings in your codebase:

```bash
elm-i18n add youAreWelcome --fr="De rien" --en="You are welcome" --replace
```

This will:
1. Add the translation to your I18n.elm file
2. Search for all occurrences of "De rien" and "You are welcome" in your Elm files
3. Replace them with `t.youAreWelcome`

Note: You'll need to ensure `t` (translations) is passed as a parameter to your views. The compiler will guide you through any necessary changes.

Example output:
```
✓ Added translation 'youAreWelcome' to src/I18n.elm
  EN: You are welcome
  FR: De rien

🔍 Searching for hardcoded strings to replace...

✓ Found 2 occurrences of "You are welcome":
  src/Main.elm:10:
    , p [] [ text "You are welcome" ]
  src/Main.elm:13:
    [ text "You are welcome"

✓ Found 2 occurrences of "De rien":
  src/Main.elm:11:
    , p [] [ text "De rien" ]
  src/Main.elm:15:
    , text "De rien"

🔄 Replacing strings with t.youAreWelcome...
✓ Replaced 4 occurrences across 1 file(s)
```

Options for `--replace`:
- `--src-dir`: Directory to search for replacements (default: "src")

### Add a function translation

For translations that require parameters or complex logic:

```bash
# Simple function with parameter
elm-i18n add-fn selected \
  --type-sig "Int -> String" \
  --en "\count -> String.fromInt count ++ \" selected\"" \
  --fr "\count -> String.fromInt count ++ \" sélectionné(s)\""

# Function with case expression (for months, categories, etc.)
elm-i18n add-fn getMonthName \
  --type-sig "Int -> String" \
  --en "\month -> case month of\n    1 -> \"January\"\n    2 -> \"February\"\n    3 -> \"March\"\n    _ -> \"Unknown\"" \
  --fr "\month -> case month of\n    1 -> \"Janvier\"\n    2 -> \"Février\"\n    3 -> \"Mars\"\n    _ -> \"Inconnu\""

# Function for ticket categories
elm-i18n add-fn ticketCategory \
  --type-sig "Ticket.TicketCategory -> String" \
  --en "\category -> case category of\n    Ticket.Maintenance -> \"Maintenance\"\n    Ticket.Cleaning -> \"Cleaning\"\n    Ticket.Other _ -> \"Other\"" \
  --fr "\category -> case category of\n    Ticket.Maintenance -> \"Maintenance\"\n    Ticket.Cleaning -> \"Nettoyage\"\n    Ticket.Other _ -> \"Autre\""
```

### Check if a translation exists

```bash
elm-i18n check welcomeBack
# ✓ Translation 'welcomeBack' exists:
#   EN: Welcome back
#   FR: Bon retour

elm-i18n check nonExistentKey
# ✗ Translation 'nonExistentKey' not found
```

### Remove a translation

```bash
elm-i18n remove oldKey
# ℹ Removing translation 'oldKey':
#   EN: Old text
#   FR: Ancien texte
# 
# ✓ Removed translation 'oldKey' from src/I18n.elm
```

### List all translations

```bash
elm-i18n list
# 📋 Found 6 translations:
#   • cancel (String)
#   • itemCount (Int -> String)
#   • loading (String)
#   • save (String)
#   • ticketStatus (Ticket.Status -> String)
#   • welcome (String)

elm-i18n list --verbose
# Shows full translation values for each key

elm-i18n list --filter "ticket"
# 📋 Found 1 translation:
#   • ticketStatus (Ticket.Status -> String)
```

### Find duplicate translations

```bash
elm-i18n duplicate-keys
# Reports only groups of keys that have the exact same translations
```

### Find values shared by multiple languages within the same key

```bash
elm-i18n shared-values
# Reports keys where multiple languages currently share the same value
# or the same function implementation

elm-i18n shared-values --suppress
# Stores the current findings in ./elm-i18n/suppressed.json
# and keeps local state in ./elm-i18n/config.json
```

### Specify a custom file location

By default, the tool looks for `src/I18n.elm`. You can specify a different path:

```bash
elm-i18n add myKey --fr "Ma clé" --en "My key" --file path/to/I18n.elm
```

## How it Works

The tool:
1. Parses your existing `I18n.elm` file
2. Checks if the key already exists
3. If new, adds the translation to:
   - The `Translations` type definition
   - The `translationsEn` record
   - The `translationsFr` record
4. Maintains proper indentation and formatting
5. Creates a backup file before making changes

## Safety Features

- **Backup**: Creates `.bak` files before modifications
- **Validation**: Checks if files exist before attempting operations
- **Duplicate Detection**: Warns when keys already exist
- **Clear Error Messages**: Provides helpful guidance when things go wrong

## Examples

### Complete workflow

```bash
# Start a new project
elm-i18n init

# Add some basic translations
elm-i18n add appName --fr "Mon Application" --en "My Application"
elm-i18n add loading --fr "Chargement..." --en "Loading..."
elm-i18n add save --fr "Sauvegarder" --en "Save"

# Add a function for pluralization
elm-i18n add-fn itemCount \
  --type-sig "Int -> String" \
  --en "\n -> if n == 1 then \"1 item\" else String.fromInt n ++ \" items\"" \
  --fr "\n -> if n == 1 then \"1 élément\" else String.fromInt n ++ \" éléments\""

# Check what you've added
elm-i18n check appName
```

## Future Enhancements

- [ ] Support for more languages dynamically
- [ ] Import/export from JSON or CSV
- [ ] Integration with elm-review
- [ ] Batch operations from a file
- [ ] Alphabetical sorting option
- [x] Remove translation command ✅
- [x] Automatic string replacement in codebase ✅

## License

MIT License - Copyright (c) 2025 Charles-André Assus
