#!/bin/bash

# Velosi Tracker - Test Runner Script
# This script runs all tests for the Rust backend with proper reporting

set -e  # Exit on any error

echo "🚀 Velosi Tracker - Running Rust Backend Tests"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the src-tauri directory"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi

print_status "Checking Rust toolchain..."
rustc --version
cargo --version

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean

# Check code formatting
print_status "Checking code formatting..."
if cargo fmt -- --check; then
    print_success "Code formatting is correct"
else
    print_warning "Code formatting issues found. Run 'cargo fmt' to fix them."
fi

# Run clippy for linting
print_status "Running Clippy linter..."
if cargo clippy -- -D warnings; then
    print_success "No linting issues found"
else
    print_warning "Linting issues found. Please fix them before proceeding."
fi

# Build the project first
print_status "Building project..."
if cargo build; then
    print_success "Build successful"
else
    print_error "Build failed"
    exit 1
fi

# Run all tests
print_status "Running all tests..."
echo ""

# Test categories
declare -a test_categories=(
    "tests::"
    "database_tests::"
    "focus_mode_tests::"
    "tracker_tests::"
)

total_tests=0
passed_tests=0
failed_tests=0

# Run each test category
for category in "${test_categories[@]}"; do
    category_name=$(echo $category | sed 's/:://g')
    print_status "Running ${category_name} tests..."
    
    if cargo test $category -- --nocapture; then
        print_success "${category_name} tests passed"
        ((passed_tests++))
    else
        print_error "${category_name} tests failed"
        ((failed_tests++))
    fi
    ((total_tests++))
    echo ""
done

# Run integration tests
print_status "Running integration tests..."
if cargo test integration_tests:: -- --nocapture; then
    print_success "Integration tests passed"
    ((passed_tests++))
else
    print_error "Integration tests failed"
    ((failed_tests++))
fi
((total_tests++))

# Generate test coverage report (if tarpaulin is installed)
if command -v cargo-tarpaulin &> /dev/null; then
    print_status "Generating test coverage report..."
    cargo tarpaulin --out Html --output-dir target/coverage
    print_success "Coverage report generated in target/coverage/tarpaulin-report.html"
else
    print_warning "cargo-tarpaulin not installed. Install it with: cargo install cargo-tarpaulin"
fi

# Summary
echo ""
echo "=============================================="
echo "🏁 Test Summary"
echo "=============================================="
echo "Total test categories: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $failed_tests"

if [ $failed_tests -eq 0 ]; then
    print_success "All tests passed! 🎉"
    echo ""
    echo "✅ Unit Tests (Tauri Commands)"
    echo "✅ Database Tests"
    echo "✅ Focus Mode Tests"
    echo "✅ Tracker Tests"
    echo "✅ Integration Tests"
    echo ""
    print_success "Your Rust backend is ready for production!"
    exit 0
else
    print_error "Some tests failed. Please fix the issues before deploying."
    exit 1
fi