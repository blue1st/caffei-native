#!/bin/bash

# This script updates the Homebrew Cask definition in the tap repository.
# Expected environment variables:
# HOMEBREW_TAP_TOKEN: GitHub Personal Access Token with repo scope

set -e

TAP_REPO="blue1st/homebrew-taps"
CASK_NAME="caffei-native"
PACKAGE_JSON="package.json"

# Get version from package.json
VERSION=$(node -p "require('./$PACKAGE_JSON').version")
echo "Updating Homebrew Cask to version $VERSION"

# We expect DMGs to be downloaded into a directory or current dir
# Filenames are expected to be like Caffei.Native_0.1.8_aarch64.dmg and Caffei.Native_0.1.8_x64.dmg
DMG_ARM=$(find . -name "*_aarch64.dmg" | head -n 1)
DMG_X64=$(find . -name "*_x64.dmg" | head -n 1)

if [ -z "$DMG_ARM" ] || [ -z "$DMG_X64" ]; then
  echo "Error: Could not find both arm64 and x64 DMG files"
  echo "ARM: $DMG_ARM"
  echo "X64: $DMG_X64"
  exit 1
fi

SHA256_ARM=$(shasum -a 256 "$DMG_ARM" | awk '{print $1}')
SHA256_X64=$(shasum -a 256 "$DMG_X64" | awk '{print $1}')

echo "ARM SHA256: $SHA256_ARM"
echo "X64 SHA256: $SHA256_X64"

# Clone the tap repository
TMP_DIR=$(mktemp -d)
git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/${TAP_REPO}.git" "$TMP_DIR"

# Ensure Casks directory exists
mkdir -p "$TMP_DIR/Casks"

CASK_FILE="$TMP_DIR/Casks/${CASK_NAME}.rb"

# Create or update the Cask file
cat <<EOF > "$CASK_FILE"
cask "${CASK_NAME}" do
  arch arm: "aarch64", intel: "x64"

  version "${VERSION}"
  sha256 arm:   "${SHA256_ARM}",
         intel: "${SHA256_X64}"

  url "https://github.com/blue1st/caffei-native/releases/download/v#{version}/Caffei.Native_#{version}_#{arch}.dmg"
  name "Caffei Native"
  desc "Sleep suppression tool with process monitoring"
  homepage "https://github.com/blue1st/caffei-native"

  app "Caffei Native.app"

  postflight do
    system_command "xattr",
                   args: ["-cr", "#{appdir}/Caffei Native.app"],
                   sudo: false
  end

  zap trash: [
    "~/Library/Application Support/com.blue1st.caffei-native",
    "~/Library/Preferences/com.blue1st.caffei-native.plist",
    "~/Library/Saved Application State/com.blue1st.caffei-native.savedState",
  ]
end
EOF

# Commit and push
cd "$TMP_DIR"
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add "Casks/${CASK_NAME}.rb"
git commit -m "Update ${CASK_NAME} to v${VERSION}" || echo "No changes to commit"
git push origin main

echo "Homebrew tap updated successfully!"
