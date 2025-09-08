// Unit test runner
// This file runs all unit tests organized by module

#[cfg(test)]
mod unit {
    mod config {
        include!("unit/config/config_tests.rs");
    }

    mod utils {
        include!("unit/utils/json_utils_tests.rs");
        include!("unit/utils/signature_tests.rs");
    }

    mod providers {
        include!("unit/providers/test_logging.rs");
    }

    mod services {
        include!("unit/services/login_tests.rs");
        include!("unit/services/webhook_processor.rs");
        include!("unit/services/token_scheduler_tests.rs");
        include!("unit/services/token_scheduler_periodic_tests.rs");
        include!("unit/services/token_scheduler_edge_cases.rs");
        include!("unit/services/token_scheduler_periodic_debug.rs");
    }
}