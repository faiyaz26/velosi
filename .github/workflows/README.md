# Complete Test Suite

This workflow has been consolidated into `complete-test-suite.yml` for better organization.

## What's included in the new workflow:

### 🔧 **Frontend Tests**

- TypeScript type checking
- ESLint linting
- Frontend build verification

### 🦀 **Rust Backend Tests**

- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Build verification
- **All tests consolidated**: `cargo test --verbose` (runs all test suites)
- Cross-platform testing (Ubuntu, Windows, macOS)

### 🔒 **Security Audit**

- `cargo audit` for dependency vulnerabilities

### 🔗 **Integration Tests**

- Full Tauri app build and integration testing

### 📦 **Release Verification**

- Release build verification across all platforms
- Tests in release mode

## Migration Guide

The new workflow automatically runs on:

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches
- When relevant files change (`src/`, `src-tauri/`, `package.json`, etc.)

## Benefits

✅ **Consolidated**: All tests run together in logical sequence  
✅ **Efficient**: Parallel jobs where possible, dependencies managed  
✅ **Comprehensive**: Frontend + Backend + Security + Integration  
✅ **Cross-platform**: Tests on Ubuntu, Windows, and macOS  
✅ **Coverage**: Test coverage reporting to Codecov  
✅ **Release-ready**: Final verification before releases

Use the **"Complete Test Suite"** workflow for all automated testing needs!
