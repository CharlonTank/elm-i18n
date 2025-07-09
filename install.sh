#\!/bin/bash

# Build the project in release mode
cargo build --release

# Copy the binary to /usr/local/bin (no sudo needed on macOS with Homebrew)
cp target/release/elm-i18n /usr/local/bin/

# Make it executable
chmod +x /usr/local/bin/elm-i18n

echo "elm-i18n has been installed to /usr/local/bin/"
echo "You can now use it from anywhere by running:"
echo ""
echo "  elm-i18n --help"
echo "  elm-i18n add myKey --fr \"Ma clé\" --en \"My key\""
echo "  elm-i18n check myKey"
echo "  elm-i18n remove oldKey"
EOF < /dev/null