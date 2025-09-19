#!/bin/bash

# Script to notarize macOS artifacts from a GitHub release
# Usage: ./notarize-release.sh [release-tag]
# If no tag provided, uses the latest release

set -euo pipefail

# Configuration - Update these with your values
GITHUB_REPO="faiyaz26/velosi"
APPLE_ID="${APPLE_ID:-}"  # Set via environment variable
APPLE_PASSWORD="${APPLE_ID_PASSWORD:-}"  # Set via environment variable
APPLE_TEAM_ID="${APPLE_TEAM_ID:-}"  # Set via environment variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if required tools are installed
check_dependencies() {
    local missing_deps=()

    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi

    if ! command -v gh &> /dev/null; then
        missing_deps+=("gh (GitHub CLI)")
    fi

    if ! command -v xcrun &> /dev/null; then
        missing_deps+=("xcrun (Xcode command line tools)")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}Error: Missing required dependencies: ${missing_deps[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi
}

# Check if Apple credentials are provided
check_credentials() {
    if [ -z "$APPLE_ID" ] || [ -z "$APPLE_PASSWORD" ] || [ -z "$APPLE_TEAM_ID" ]; then
        echo -e "${RED}Error: Apple credentials not provided.${NC}"
        echo "Please set these environment variables:"
        echo "  APPLE_ID=your-apple-id@email.com"
        echo "  APPLE_ID_PASSWORD=your-app-specific-password"
        echo "  APPLE_TEAM_ID=your-team-id"
        echo ""
        echo "Or create a .env file with these variables."
        exit 1
    fi
}

# Load environment variables from .env file if it exists
load_env_file() {
    if [ -f ".env" ]; then
        echo -e "${YELLOW}Loading credentials from .env file...${NC}"
        set -a
        source .env
        set +a
    fi
}

# Get release information
get_release_info() {
    local tag="${1:-}"

    if [ -z "$tag" ]; then
        echo -e "${YELLOW}No tag specified, getting latest release...${NC}"
        RELEASE_INFO=$(gh release list --repo "$GITHUB_REPO" --json tagName,publishedAt --limit 1)
        TAG=$(echo "$RELEASE_INFO" | jq -r '.[0].tagName')
    else
        TAG="$tag"
        echo -e "${YELLOW}Using specified tag: $TAG${NC}"
    fi

    if [ "$TAG" = "null" ] || [ -z "$TAG" ]; then
        echo -e "${RED}Error: Could not find release${NC}"
        exit 1
    fi

    echo -e "${GREEN}Found release: $TAG${NC}"
}

# Download macOS artifacts from release
download_artifacts() {
    echo -e "${YELLOW}Downloading macOS artifacts from release $TAG...${NC}"

    # Create temp directory for downloads
    TEMP_DIR=$(mktemp -d)
    echo "Using temp directory: $TEMP_DIR"

    # Download all assets from the release
    gh release download "$TAG" --repo "$GITHUB_REPO" --dir "$TEMP_DIR"

    # Find DMG files
    DMG_FILES=($(find "$TEMP_DIR" -name "*.dmg" -type f))

    if [ ${#DMG_FILES[@]} -eq 0 ]; then
        echo -e "${RED}Error: No DMG files found in release${NC}"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    echo -e "${GREEN}Found ${#DMG_FILES[@]} DMG file(s):${NC}"
    for dmg in "${DMG_FILES[@]}"; do
        echo "  $(basename "$dmg")"
    done

    DOWNLOAD_DIR="$TEMP_DIR"
}

# Notarize a single DMG file
notarize_dmg() {
    local dmg_path="$1"
    local dmg_name=$(basename "$dmg_path")

    echo -e "${YELLOW}Notarizing $dmg_name...${NC}"

    # Submit for notarization
    echo "Submitting to Apple notary service..."
    SUBMISSION_RESULT=$(xcrun notarytool submit "$dmg_path" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --team-id "$APPLE_TEAM_ID" \
        --wait \
        --output-format json)

    # Check if submission was successful
    STATUS=$(echo "$SUBMISSION_RESULT" | jq -r '.status')

    if [ "$STATUS" != "Accepted" ]; then
        echo -e "${RED}Error: Notarization failed for $dmg_name${NC}"
        echo "Status: $STATUS"
        echo "Full response: $SUBMISSION_RESULT"
        return 1
    fi

    echo -e "${GREEN}Notarization successful for $dmg_name${NC}"

    # Staple the notarization ticket
    echo "Stapling notarization ticket..."
    xcrun stapler staple "$dmg_path"

    # Verify stapling
    echo "Verifying stapling..."
    xcrun stapler validate "$dmg_path"

    echo -e "${GREEN}Successfully notarized and stapled: $dmg_name${NC}"
}

# Upload notarized DMG back to release
upload_notarized_dmg() {
    local dmg_path="$1"
    local dmg_name=$(basename "$dmg_path")

    echo -e "${YELLOW}Uploading notarized $dmg_name to release...${NC}"

    # Upload the notarized file to the release
    gh release upload "$TAG" "$dmg_path" --repo "$GITHUB_REPO" --clobber

    echo -e "${GREEN}Successfully uploaded notarized $dmg_name${NC}"
}

# Main function
main() {
    echo -e "${GREEN}=== GitHub Release Notarization Script ===${NC}"

    # Load environment variables
    load_env_file

    # Check dependencies
    check_dependencies

    # Check credentials
    check_credentials

    # Get release info
    get_release_info "$1"

    # Download artifacts
    download_artifacts

    # Notarize each DMG
    NOTARIZED_FILES=()
    for dmg in "${DMG_FILES[@]}"; do
        if notarize_dmg "$dmg"; then
            NOTARIZED_FILES+=("$dmg")
        else
            echo -e "${RED}Failed to notarize $dmg, skipping upload${NC}"
        fi
    done

    # Upload notarized files back to release
    if [ ${#NOTARIZED_FILES[@]} -gt 0 ]; then
        echo -e "${YELLOW}Uploading ${#NOTARIZED_FILES[@]} notarized file(s)...${NC}"
        for dmg in "${NOTARIZED_FILES[@]}"; do
            upload_notarized_dmg "$dmg"
        done
    else
        echo -e "${RED}No files were successfully notarized${NC}"
        exit 1
    fi

    # Cleanup
    echo -e "${YELLOW}Cleaning up temporary files...${NC}"
    rm -rf "$DOWNLOAD_DIR"

    echo -e "${GREEN}=== Notarization complete! ===${NC}"
    echo "Release $TAG has been updated with notarized macOS artifacts."
}

# Show usage if requested
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [release-tag]"
    echo ""
    echo "Notarize macOS artifacts from a GitHub release."
    echo ""
    echo "Arguments:"
    echo "  release-tag    Optional: Specific release tag to process (default: latest)"
    echo ""
    echo "Environment Variables:"
    echo "  APPLE_ID          Your Apple ID email"
    echo "  APPLE_ID_PASSWORD Your app-specific password"
    echo "  APPLE_TEAM_ID     Your Apple Developer Team ID"
    echo ""
    echo "Or create a .env file with these variables."
    exit 0
fi

# Run main function
main "$@"