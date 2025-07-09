#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}elm-i18n CLI Uninstaller${NC}"
echo "=========================="

# Common installation paths
PATHS=(
    "$HOME/.local/bin/elm-i18n"
    "/usr/local/bin/elm-i18n"
    "$HOME/.cargo/bin/elm-i18n"
)

FOUND=false

# Check each path
for path in "${PATHS[@]}"; do
    if [ -f "$path" ]; then
        echo -e "\n${BLUE}Found elm-i18n at: $path${NC}"
        read -p "Remove this installation? (y/N) " -n 1 -r
        echo
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if rm "$path"; then
                echo -e "${GREEN}✓ Removed $path${NC}"
                FOUND=true
            else
                echo -e "${RED}✗ Failed to remove $path. You may need sudo permissions.${NC}"
                echo -e "Try running: ${BLUE}sudo $0${NC}"
            fi
        else
            echo "Skipped $path"
        fi
    fi
done

if [ "$FOUND" = false ]; then
    echo -e "\n${RED}No elm-i18n installations found in common locations.${NC}"
    echo "You can manually check with: which elm-i18n"
fi

echo -e "\n${GREEN}Uninstall complete!${NC}"