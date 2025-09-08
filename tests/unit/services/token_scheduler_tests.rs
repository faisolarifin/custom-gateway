use webhook_gateway::services::{TokenScheduler, SchedulerConfig};

// Constants from the module for testing
const DEFAULT_PERIODIC_INTERVAL_MINS: u64 = 15;

#[tokio::test]
async fn test_scheduler_creation() {
    let scheduler = TokenScheduler::new();
    assert!(!scheduler.is_scheduler_active());
    assert!(scheduler.get_scheduler_info().is_none());
}

#[tokio::test]
async fn test_scheduler_with_custom_config() {
    let config = SchedulerConfig {
        periodic_interval_mins: 10,
    };
    let scheduler = TokenScheduler::with_config(config);
    
    // Should accept custom config
    assert!(!scheduler.is_scheduler_active()); // Not started yet
    
    let config = scheduler.get_config();
    assert_eq!(config.periodic_interval_mins, 10);
}

#[tokio::test] 
async fn test_config_getters_and_setters() {
    let mut scheduler = TokenScheduler::new();
    
    // Test default config
    let config = scheduler.get_config();
    assert_eq!(config.periodic_interval_mins, DEFAULT_PERIODIC_INTERVAL_MINS);
    
    // Test config update
    let new_config = SchedulerConfig {
        periodic_interval_mins: 30,
    };
    scheduler.update_config(new_config.clone());
    
    let updated_config = scheduler.get_config();
    assert_eq!(updated_config.periodic_interval_mins, 30);
}

#[tokio::test]
async fn test_scheduler_stop() {
    let scheduler = TokenScheduler::new();
    
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
    
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_scheduler_replacement() {
    let scheduler = TokenScheduler::new();
    
    // Start first scheduler
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
    
    // Start second scheduler - should replace the first
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
    
    // Should still have only one active scheduler
    let info = scheduler.get_scheduler_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("Periodic token refresh scheduler active"));
}

#[tokio::test]
async fn test_scheduler_shutdown() {
    let scheduler = TokenScheduler::new();
    
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
    
    scheduler.shutdown();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_scheduler_config_validation() {
    // Test extreme values
    let extreme_config = SchedulerConfig {
        periodic_interval_mins: 1,
    };
    let scheduler = TokenScheduler::with_config(extreme_config);
    
    let config = scheduler.get_config();
    assert_eq!(config.periodic_interval_mins, 1);
    
    // Should handle extreme values gracefully
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
}

#[tokio::test] 
async fn test_default_vs_custom_config() {
    let default_scheduler = TokenScheduler::new();
    let custom_scheduler = TokenScheduler::with_config(SchedulerConfig {
        periodic_interval_mins: 30,
    });
    
    let default_config = default_scheduler.get_config();
    let custom_config = custom_scheduler.get_config();
    
    assert_ne!(default_config.periodic_interval_mins, custom_config.periodic_interval_mins);
    
    assert_eq!(default_config.periodic_interval_mins, DEFAULT_PERIODIC_INTERVAL_MINS);
    assert_eq!(custom_config.periodic_interval_mins, 30);
}