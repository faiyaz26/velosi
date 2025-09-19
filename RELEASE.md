# Release Process

This document describes the release process for Velosi Tracker, including building, notarization, and publishing.

## Automated Release Workflow

The GitHub Actions workflow (`release.yml`) handles the automated build and release process:

1. **Build Phase**: Builds the application for both macOS (Intel/Silicon) and Windows
2. **Release Creation**: Creates a GitHub release with all built artifacts
3. **Manual Notarization**: macOS DMGs are notarized separately using the local script

## Required GitHub Secrets

Set these secrets in your GitHub repository settings (Settings → Secrets and variables → Actions):

### For macOS Builds

- `APPLE_ID`: Your Apple ID email address
- `APPLE_ID_PASSWORD`: App-specific password from https://appleid.apple.com/account/manage
- `APPLE_TEAM_ID`: Your Apple Developer Team ID (found at https://developer.apple.com/account)
- `APPLE_CERTIFICATE`: Base64-encoded Apple Developer certificate (.p12 file)
- `APPLE_CERTIFICATE_PASSWORD`: Password for the certificate
- `KEYCHAIN_PASSWORD`: Password for the temporary keychain

### For Windows Builds

- `TAURI_PRIVATE_KEY`: Base64-encoded Tauri private key for code signing
- `TAURI_KEY_PASSWORD`: Password for the Tauri private key

### General

- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

## Manual Notarization Process

After the automated release is created, you need to notarize the macOS artifacts locally:

### Setup

1. **Install GitHub CLI** (if not already installed):

   ```bash
   brew install gh
   ```

2. **Authenticate with GitHub**:

   ```bash
   gh auth login
   ```

3. **Configure Apple Credentials**:
   ```bash
   cp .env.example .env
   # Edit .env with your actual Apple credentials
   ```

### Run Notarization

```bash
# Notarize the latest release
./notarize-release.sh

# Or notarize a specific release
./notarize-release.sh v1.2.3
```

The script will:

1. Download macOS DMG files from the GitHub release
2. Submit them to Apple for notarization
3. Wait for notarization to complete
4. Staple notarization tickets to the DMGs
5. Upload the notarized DMGs back to the release

## Manual Release Trigger

To trigger a release manually:

1. Go to GitHub Actions tab
2. Click "Release" workflow
3. Click "Run workflow"
4. Optionally specify a custom tag name
5. Click "Run workflow"

If no tag name is provided, the workflow will use the version from `src-tauri/tauri.conf.json`.

## Troubleshooting

### Notarization Issues

- Ensure your Apple credentials are correct
- Check that your Apple Developer account has the necessary permissions
- Verify that the app bundle identifier doesn't conflict with macOS conventions

### Build Issues

- Check that all required secrets are set
- Ensure the Node.js version matches Vite requirements (currently 20+)
- Verify that Rust and Tauri CLI are properly configured

### Release Issues

- Check that the tag doesn't already exist
- Ensure all artifacts are properly uploaded
- Verify GitHub token permissions

## File Structure

```
├── .github/workflows/release.yml    # Main release workflow
├── notarize-release.sh              # Local notarization script
├── .env.example                     # Example credentials file
└── src-tauri/tauri.conf.json        # App configuration with version
```
