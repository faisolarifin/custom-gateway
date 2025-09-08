# Test Organization for Permata Gateway

This document describes the test organization structure for the Permata Gateway project.

## Directory Structure

```
tests/
├── lib.rs                      # Test library entry point
├── unit_tests.rs              # Unit test runner
├── integration_tests.rs       # Integration test runner
├── unit/                      # Unit tests
│   ├── mod.rs                 # Unit test module entry
│   ├── config/                # Configuration tests
│   │   ├── mod.rs
│   │   └── config_tests.rs
│   ├── handlers/              # HTTP handler tests
│   │   └── mod.rs
│   ├── models/                # Data model tests
│   │   └── mod.rs
│   ├── providers/             # Provider tests (logging, etc)
│   │   ├── mod.rs
│   │   └── test_logging.rs
│   ├── services/              # Service layer tests
│   │   ├── mod.rs
│   │   ├── login_tests.rs
│   │   ├── webhook_processor.rs
│   │   ├── token_scheduler_tests.rs
│   │   ├── token_scheduler_periodic_tests.rs
│   │   ├── token_scheduler_edge_cases.rs
│   │   └── token_scheduler_periodic_debug.rs
│   └── utils/                 # Utility function tests
│       ├── mod.rs
│       ├── json_utils_tests.rs
│       └── signature_tests.rs
└── integration/               # Integration tests
    ├── mod.rs
    ├── built_in_tests.rs
    ├── integration_tests.rs
    └── integration_test_suite.rs
```

## Test Types

### Unit Tests
Unit tests focus on testing individual components in isolation:
- **Config Tests**: Configuration loading and validation
- **Utils Tests**: JSON utilities, signature generation, error handling
- **Services Tests**: Token management, webhook processing, HTTP clients
- **Providers Tests**: Logging functionality
- **Models Tests**: Data structures and serialization (future)
- **Handlers Tests**: HTTP request handling (future)

### Integration Tests
Integration tests verify interaction between components:
- **Built-in Tests**: Basic configuration and component integration
- **Integration Tests**: Cross-component functionality
- **Integration Test Suite**: End-to-end workflow testing

## Running Tests

### Using Cargo
```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test unit_tests

# Run only integration tests  
cargo test --test integration_tests

# Run specific test module
cargo test config_tests
cargo test json_utils_tests
```

### Using Test Scripts
```bash
# Linux/Mac
./scripts/test.sh all          # Run all tests
./scripts/test.sh unit         # Run unit tests only
./scripts/test.sh integration  # Run integration tests only
./scripts/test.sh config       # Run config module tests
./scripts/test.sh utils        # Run utils module tests
./scripts/test.sh services     # Run services module tests

# Windows
scripts\test.bat all           # Run all tests
scripts\test.bat unit          # Run unit tests only
scripts\test.bat integration   # Run integration tests only
```

## Test Guidelines

### Unit Test Conventions
- Each unit test file should focus on a specific module or functionality
- Use descriptive test names that explain what is being tested
- Mock external dependencies using `mockall` crate
- Test both success and failure scenarios

### Integration Test Conventions
- Test complete workflows from end-to-end
- Use real configurations where possible
- Test error conditions and retry mechanisms
- Verify logging output and side effects

### Test Data
- Use realistic test data that mirrors production scenarios
- Store common test fixtures in dedicated modules
- Avoid hardcoding sensitive information in tests

### Naming Conventions
- Test files: `{module}_tests.rs`
- Test functions: `test_{functionality}_{scenario}`
- Test modules: Follow the same structure as `src/`

## Adding New Tests

### For Unit Tests
1. Create test file in appropriate `tests/unit/{module}/` directory
2. Add module declaration to `tests/unit/{module}/mod.rs`
3. Update `tests/unit_tests.rs` to include the new test file

### For Integration Tests
1. Create test file in `tests/integration/` directory
2. Add module declaration to `tests/integration/mod.rs`  
3. Update `tests/integration_tests.rs` to include the new test file

## CI/CD Integration

The test structure supports different test execution strategies:
- **Pull Request**: Run unit tests for fast feedback
- **Pre-merge**: Run all tests including integration
- **Nightly**: Run extended test suites with real external dependencies

## Dependencies

Test-specific dependencies are defined in `Cargo.toml`:
```toml
[dev-dependencies]
mockall = "0.13"      # Mocking framework
tokio-test = "0.4"    # Async test utilities
wiremock = "0.6"      # HTTP mocking
tempfile = "3.0"      # Temporary file handling
```