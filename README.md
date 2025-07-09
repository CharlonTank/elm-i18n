# elm-i18n CLI

A command-line tool for managing internationalization (i18n) translations in Elm applications.

## Features

- ✅ Add simple key-value translations
- ✅ Add function translations (with anonymous functions)
- ✅ Check if translations exist
- ✅ Remove translations
- ✅ Initialize new I18n.elm files
- ✅ Shows existing translations when key already exists
- ✅ Maintains proper Elm formatting
- ✅ Creates backups before modifying files

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

## License

MIT License - Copyright (c) 2025 Charles-André Assus