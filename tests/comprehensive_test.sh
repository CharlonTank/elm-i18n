#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ELM_I18N="/Users/charles-andreassus/projects/elm-i18n/target/release/elm-i18n"

echo -e "${BLUE}=== ELM-I18N COMPREHENSIVE TEST SUITE ===${NC}"
echo

# Function to print test headers
test_header() {
    echo -e "${YELLOW}TEST: $1${NC}"
}

# Function to print success
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print failure
failure() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# Test 1: Help and Version (don't need config)
test_header "Help and Version Commands"
cd fulltest/single
$ELM_I18N --help | grep -q "CLI tool for managing Elm I18n translations" && success "Help command works" || failure "Help command failed"
$ELM_I18N version | grep -q "elm-i18n v0.5.0" && success "Version command works" || failure "Version command failed"
echo

# Test 2: Setup prompt when no config
test_header "No Config Error"
$ELM_I18N add test --en "Test" --fr "Test" 2>&1 | grep -q "No elm-i18n.json configuration found" && success "Prompts for setup when no config" || failure "Should prompt for setup"
echo

# Test 3: Single-file configuration
test_header "Single-File Mode Setup"
cat > elm-i18n.json << 'EOF'
{
  "version": "1.0",
  "mode": "single-file",
  "languages": ["en", "fr"],
  "sourceDir": "src",
  "file": "src/I18n.elm",
  "recordName": "Translations"
}
EOF
success "Created single-file config"
mkdir -p src
echo

# Test 4: Init in single-file mode
test_header "Initialize Single-File"
$ELM_I18N init
[ -f "src/I18n.elm" ] && success "Created I18n.elm" || failure "Failed to create I18n.elm"
grep -q "type alias Translations" src/I18n.elm && success "Has Translations type" || failure "Missing Translations type"
grep -q "FR" src/I18n.elm && success "Has FR language" || failure "Missing FR language"
echo

# Test 5: Add simple translation
test_header "Add Simple Translation"
$ELM_I18N add greeting --en "Hello" --fr "Bonjour"
grep -q "greeting : String" src/I18n.elm && success "Added type field" || failure "Failed to add type field"
grep -q 'greeting = "Hello"' src/I18n.elm && success "Added EN translation" || failure "Failed to add EN translation"
grep -q 'greeting = "Bonjour"' src/I18n.elm && success "Added FR translation" || failure "Failed to add FR translation"
# ES translation removed from test
echo

# Test 6: Check translation
test_header "Check Translation"
$ELM_I18N check greeting | grep -q "Translation 'greeting' exists" && success "Check finds existing key" || failure "Check failed"
$ELM_I18N check nonexistent 2>&1 | grep -q "not found" && success "Check reports missing key" || failure "Check should report missing"
echo

# Test 7: List translations
test_header "List Translations"
$ELM_I18N list | grep -q "greeting" && success "List shows translations" || failure "List failed"
$ELM_I18N list --verbose | grep -q "Bonjour" && success "Verbose list shows values" || failure "Verbose list failed"
$ELM_I18N list --filter "greet" | grep -q "greeting" && success "Filter works" || failure "Filter failed"
echo

# Test 8: Remove translation
test_header "Remove Translation"
$ELM_I18N add temporary --en "Temp" --fr "Temp"
$ELM_I18N remove temporary
! grep -q "temporary" src/I18n.elm && success "Removed translation" || failure "Failed to remove"
echo

# Test 9: Add function translation
test_header "Add Function Translation"
$ELM_I18N add-fn itemCount --type-sig "Int -> String" --en '\\n -> String.fromInt n ++ " items"' --fr '\\n -> String.fromInt n ++ " articles"'
grep -q "itemCount : Int -> String" src/I18n.elm && success "Added function type" || failure "Failed to add function type"
grep -q '\\n ->' src/I18n.elm && success "Added lambda function" || failure "Failed to add lambda"
echo

# Test 10: Multi-file configuration
test_header "Multi-File Mode Setup"
cd ../multi
cat > elm-i18n.json << 'EOF'
{
  "version": "1.0",
  "mode": "multi-file",
  "languages": ["en", "fr"],
  "sourceDir": "src",
  "files": {
    "app": {
      "path": "src/I18n/App.elm",
      "recordName": "AppTranslations"
    },
    "landing": {
      "path": "src/I18n/Landing.elm",
      "recordName": "LandingTranslations"
    },
    "admin": {
      "path": "src/I18n/Admin.elm",
      "recordName": "AdminTranslations"
    }
  }
}
EOF
success "Created multi-file config"
mkdir -p src/I18n
echo

# Test 11: Multi-file requires shortcuts
test_header "Multi-File Shortcut Requirement"
$ELM_I18N add test --en "Test" --fr "Test" 2>&1 | grep -q "Multi-file mode requires a file shortcut" && success "Requires shortcut" || failure "Should require shortcut"
echo

# Test 12: Multi-file init with shortcuts
test_header "Multi-File Initialize"
$ELM_I18N --target app init
[ -f "src/I18n/App.elm" ] && success "Created App.elm" || failure "Failed to create App.elm"
grep -q "type alias AppTranslations" src/I18n/App.elm && success "Has AppTranslations type" || failure "Wrong type name"

$ELM_I18N --target landing init
grep -q "type alias LandingTranslations" src/I18n/Landing.elm && success "Has LandingTranslations type" || failure "Wrong type name in Landing"
echo

# Test 13: Add to specific files
test_header "Multi-File Add Translations"
$ELM_I18N --target app add userProfile --en "User Profile" --fr "Profil Utilisateur"
grep -q "userProfile" src/I18n/App.elm && success "Added to App.elm" || failure "Failed to add to App"
! grep -q "userProfile" src/I18n/Landing.elm && success "Not in Landing.elm" || failure "Leaked to Landing"

$ELM_I18N --target landing add heroTitle --en "Welcome!" --fr "Bienvenue!"
grep -q "heroTitle" src/I18n/Landing.elm && success "Added to Landing.elm" || failure "Failed to add to Landing"
! grep -q "heroTitle" src/I18n/App.elm && success "Not in App.elm" || failure "Leaked to App"
echo

# Test 14: List in multi-file mode
test_header "Multi-File List"
$ELM_I18N --target app list | grep -q "userProfile" && success "Lists App translations" || failure "Failed to list App"
$ELM_I18N --target landing list | grep -q "heroTitle" && success "Lists Landing translations" || failure "Failed to list Landing"
echo

# Test 15: Check in multi-file mode
test_header "Multi-File Check"
$ELM_I18N --target app check userProfile | grep -q "exists" && success "Finds in App" || failure "Failed to find in App"
$ELM_I18N --target landing check heroTitle | grep -q "exists" && success "Finds in Landing" || failure "Failed to find in Landing"
$ELM_I18N --target app check heroTitle 2>&1 | grep -q "not found" && success "Not found cross-file" || failure "Should not find cross-file"
echo

# Test 16: Remove in multi-file mode
test_header "Multi-File Remove"
$ELM_I18N --target app add tempKey --en "Temp" --fr "Temp"
$ELM_I18N --target app remove tempKey
! grep -q "tempKey" src/I18n/App.elm && success "Removed from App" || failure "Failed to remove from App"
echo

# Test 17: Function translations in multi-file
test_header "Multi-File Functions"
$ELM_I18N --target app add-fn formatDate --type-sig "Date -> String" --en '\\d -> "Date: " ++ dateToString d' --fr '\\d -> "Date: " ++ dateToString d'
grep -q "formatDate : Date -> String" src/I18n/App.elm && success "Added function to App" || failure "Failed to add function"
echo

# Test 18: Test with replacement
test_header "String Replacement"
cd ../replace
cat > elm-i18n.json << 'EOF'
{
  "version": "1.0",
  "mode": "single-file",
  "languages": ["en", "fr"],
  "sourceDir": "src",
  "file": "src/I18n.elm",
  "recordName": "Translations"
}
EOF
mkdir -p src
$ELM_I18N init

# Create a test Elm file with hardcoded strings
cat > src/Main.elm << 'EOF'
module Main exposing (..)

view model =
    div []
        [ h1 [] [ text "Welcome to our app" ]
        , p [] [ text "Bienvenue dans notre application" ]
        , button [] [ text "Click me" ]
        ]
EOF

$ELM_I18N add appWelcome --en "Welcome to our app" --fr "Bienvenue dans notre application" --replace
grep -q "t.appWelcome" src/Main.elm && success "Replaced strings" || failure "Failed to replace strings"
echo

# Test 19: Error cases
test_header "Error Cases"
cd ../error-cases

# No config
! $ELM_I18N list 2>/dev/null && success "Fails without config" || failure "Should fail without config"

# Invalid key names
cat > elm-i18n.json << 'EOF'
{
  "version": "1.0",
  "mode": "single-file",
  "languages": ["en", "fr"],
  "sourceDir": "src",
  "file": "src/I18n.elm",
  "recordName": "Translations"
}
EOF
mkdir -p src
$ELM_I18N init

$ELM_I18N add "invalid.key" --en "Test" --fr "Test" 2>&1 | grep -q "cannot contain dots" && success "Rejects dots in keys" || failure "Should reject dots"
$ELM_I18N add "123invalid" --en "Test" --fr "Test" 2>&1 | grep -q "must start with a letter" && success "Rejects non-letter start" || failure "Should reject number start"
echo

# Test 20: Reserved words
test_header "Reserved Words Handling"
$ELM_I18N add type --en "Type" --fr "Type" 2>&1 | grep -q "reserved word" && success "Warns about reserved words" || failure "Should warn about reserved"
grep -q "type_ :" src/I18n.elm && success "Adds underscore to reserved" || failure "Should add underscore"
echo

# Test 21: Duplicate keys
test_header "Duplicate Key Handling"
$ELM_I18N add existingKey --en "First" --fr "Premier"
$ELM_I18N add existingKey --en "Second" --fr "Deuxieme" | grep -q "already exists" && success "Warns about duplicates" || failure "Should warn about duplicates"
echo

# Test 22: Remove unused (needs actual unused keys)
test_header "Remove Unused Keys"
cd ../unused
cat > elm-i18n.json << 'EOF'
{
  "version": "1.0",
  "mode": "single-file",
  "languages": ["en", "fr"],
  "sourceDir": "src",
  "file": "src/I18n.elm",
  "recordName": "Translations"
}
EOF
mkdir -p src
$ELM_I18N init
$ELM_I18N add used --en "Used" --fr "Utilisé"
$ELM_I18N add unused1 --en "Unused1" --fr "Inutilisé1"
$ELM_I18N add unused2 --en "Unused2" --fr "Inutilisé2"

# Create a file that uses only one key
cat > src/Main.elm << 'EOF'
module Main exposing (..)
import I18n

view t model =
    div [] [ text t.used ]
EOF

$ELM_I18N remove-unused | grep -q "unused1" && success "Finds unused keys" || failure "Should find unused keys"
$ELM_I18N remove-unused --confirm
! grep -q "unused1" src/I18n.elm && success "Removed unused keys" || failure "Failed to remove unused"
echo

# Test 23: Complex function translations
test_header "Complex Function Translations"
cd ../functions
cat > elm-i18n.json << 'EOF'
{
  "version": "1.0",
  "mode": "single-file",
  "languages": ["en", "fr"],
  "sourceDir": "src",
  "file": "src/I18n.elm",
  "recordName": "Translations"
}
EOF
mkdir -p src
$ELM_I18N init

# Add complex function with case statement
$ELM_I18N add-fn statusMessage --type-sig "Status -> String" --en '\\s -> case s of
    Active -> "Active"
    Inactive -> "Inactive"
    Pending -> "Pending"' --fr '\\s -> case s of
    Active -> "Actif"
    Inactive -> "Inactif"
    Pending -> "En attente"'

grep -q "statusMessage : Status -> String" src/I18n.elm && success "Added complex function type" || failure "Failed complex function"
grep -q 'Active -> "Active"' src/I18n.elm && success "Has case branches" || failure "Missing case branches"
echo

# Final summary
echo -e "${GREEN}====================================${NC}"
echo -e "${GREEN}ALL COMPREHENSIVE TESTS COMPLETED!${NC}"
echo -e "${GREEN}====================================${NC}"