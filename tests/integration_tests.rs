// Integration test runner
// This file runs all integration tests

#[cfg(test)]
mod integration {
    include!("integration/built_in_tests.rs");
    include!("integration/integration_tests.rs");
    include!("integration/integration_test_suite.rs");
}