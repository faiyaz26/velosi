# Velosi Tracker - Rust Backend Tests

This directory contains comprehensive test cases for the Velosi Tracker Rust backend, specifically focusing on testing all Tauri commands and core functionality.

## Test Structure

### 1. Unit Tests (`src/tests.rs`)
Tests individual Tauri commands in isolation:
- **Tracking Commands**: `start_tracking`, `stop_tracking`, `get_tracking_status`, `toggle_tracking`
- **Pause/Resume Commands**: `pause_tracking`, `resume_tracking`, `get_pause_status`
- **Activity Management**: `get_current_activity`, `set_current_activity`
- **Category Management**: `get_categories`, `add_category`, `update_category`, `delete_category`
- **App Mapping Commands**: `get_app_mappings`, `add_app_mapping`, `remove_app_mapping`
- **URL Mapping Commands**: `get_url_mappings`, `add_url_mapping`, `remove_url_mapping`
- **Focus Mode Commands**: `enable_focus_mode`, `disable_focus_mode`, `get_focus_mode_status`
- **Focus Mode Categories**: `set_focus_mode_categories`, `get_focus_mode_categories`
- **Focus Mode Apps**: `allow_app`, `get_focus_mode_allowed_apps`, `remove_focus_mode_allowed_app`
- **Data Retrieval**: `get_activities_by_date`, `get_activity_summary`, `get_timeline_data`
- **Window Management**: `show_main_window`, `hide_main_window`, `close_main_window`

### 2. Database Tests (`src/database_tests.rs`)
Tests database operations directly:
- **Activity CRUD**: Creating, reading, updating, and deleting activities
- **Category Management**: User category operations
- **App/URL Mappings**: Mapping management operations
- **Focus Mode Settings**: Focus mode database operations
- **Data Queries**: Date-based queries and summaries
- **Edge Cases**: Invalid data handling, cleanup operations

### 3. Focus Mode Tests (`src/focus_mode_tests.rs`)
Tests focus mode functionality:
- **Basic Operations**: Enable/disable focus mode
- **App Blocking Logic**: Testing app allow/block decisions
- **Category-based Filtering**: Testing category-based app filtering
- **Temporary App Allowance**: Testing temporary app permissions
- **Pattern Matching**: Testing app name and bundle ID matching
- **Edge Cases**: Empty categories, case sensitivity, Velosi app exceptions

### 4. Tracker Tests (`src/tracker_tests.rs`)
Tests activity tracking data structures:
- **CurrentActivity**: Creation, serialization, cloning
- **SegmentInfo**: Different segment types and metadata
- **Data Validation**: Testing with minimal and maximal data
- **Serialization**: JSON serialization/deserialization

## Running Tests

### Run All Tests
```bash
cd src-tauri
cargo test
```

### Run Specific Test Modules
```bash
# Run only unit tests
cargo test tests::

# Run only database tests
cargo test database_tests::

# Run only focus mode tests
cargo test focus_mode_tests::

# Run only tracker tests
cargo test tracker_tests::
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Release Mode
```bash
cargo test --release
```

## Test Coverage

The test suite covers:

### ✅ Tauri Commands (100% coverage)
- All 47 Tauri commands are tested
- Both success and error cases
- Parameter validation
- State management

### ✅ Database Operations
- SQLite operations
- Data persistence
- Query correctness
- Transaction handling

### ✅ Focus Mode Logic
- App blocking/allowing logic
- Category-based filtering
- Temporary permissions
- Pattern matching algorithms

### ✅ Data Structures
- Model serialization/deserialization
- Data validation
- Edge case handling

## Test Data

Tests use in-memory SQLite databases (`sqlite::memory:`) to ensure:
- Fast test execution
- No side effects between tests
- Clean state for each test
- No external dependencies

## Mock Objects

The tests use Tauri's built-in mocking system:
- `mock_app()`: Creates a mock Tauri application
- `MockRuntime`: Provides a test runtime environment
- Mock state management for isolated testing

## Continuous Integration

These tests are designed to run in CI environments:
- No external dependencies
- Deterministic results
- Fast execution
- Comprehensive error reporting

## Adding New Tests

When adding new Tauri commands or functionality:

1. **Add unit tests** in `src/tests.rs` for the command
2. **Add database tests** in `src/database_tests.rs` if database operations are involved
3. **Add integration tests** for complex workflows
4. **Update this README** with new test coverage information

### Test Template

```rust
#[tokio::test]
async fn test_new_command() {
    let (app, state) = create_mock_app().await;
    let app_handle = app.handle().clone();
    let state_ref = State::from(&state);

    // Setup test data
    // ...

    // Execute command
    let result = new_command(state_ref, app_handle, /* params */).await;
    
    // Verify results
    assert!(result.is_ok());
    // Additional assertions...
}
```

## Known Limitations

1. **Platform-specific code**: Some tracker functionality is platform-specific and may not be fully testable in all environments
2. **UI interactions**: Window management commands are tested for basic functionality but not actual UI behavior
3. **Timing-dependent tests**: Some tests involving timeouts or expiry may be sensitive to system performance

## Debugging Tests

For debugging failing tests:

```bash
# Run with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run a specific test
cargo test test_specific_function_name -- --nocapture

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```