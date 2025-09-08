use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use webhook_gateway::services::{TokenScheduler, SchedulerConfig};

#[tokio::test]
async fn test_periodic_scheduler_creation() {
    let config = SchedulerConfig {
        periodic_interval_mins: 15,
    };
    let scheduler = TokenScheduler::with_config(config);
    
    assert!(!scheduler.is_scheduler_active());
    assert!(scheduler.get_scheduler_info().is_none());
}

#[tokio::test]
async fn test_periodic_scheduler_start_stop() {
    let config = SchedulerConfig {
        periodic_interval_mins: 1, // 1 minute for faster testing
    };
    let scheduler = TokenScheduler::with_config(config);
    
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);
    
    scheduler.start_scheduler_simple(move || {
        called_clone.store(true, Ordering::SeqCst);
    });
    
    assert!(scheduler.is_scheduler_active());
    
    // Verify scheduler info
    let info = scheduler.get_scheduler_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("1 minutes"));
    
    // Stop scheduler
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_periodic_scheduler_async_callback() {
    let config = SchedulerConfig {
        periodic_interval_mins: 1,
    };
    let scheduler = TokenScheduler::with_config(config);
    
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);
    
    scheduler.start_scheduler(move || {
        let called = Arc::clone(&called_clone);
        async move {
            called.store(true, Ordering::SeqCst);
            Ok(())
        }
    });
    
    assert!(scheduler.is_scheduler_active());
    
    // Stop before any execution
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
    
    // Should not have been called since we stopped it immediately
    sleep(Duration::from_millis(100)).await;
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_periodic_scheduler_replacement() {
    let config = SchedulerConfig {
        periodic_interval_mins: 30,
    };
    let scheduler = TokenScheduler::with_config(config);
    
    // Start first scheduler
    let count1 = Arc::new(AtomicUsize::new(0));
    let count1_clone = Arc::clone(&count1);
    scheduler.start_scheduler_simple(move || {
        count1_clone.fetch_add(1, Ordering::SeqCst);
    });
    assert!(scheduler.is_scheduler_active());
    
    // Start second scheduler - should replace the first
    let count2 = Arc::new(AtomicUsize::new(0));
    let count2_clone = Arc::clone(&count2);
    scheduler.start_scheduler_simple(move || {
        count2_clone.fetch_add(1, Ordering::SeqCst);
    });
    assert!(scheduler.is_scheduler_active());
    
    // Should still have only one active scheduler
    let info = scheduler.get_scheduler_info();
    assert!(info.is_some());
    assert!(info.unwrap().contains("30 minutes"));
    
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_periodic_scheduler_config() {
    let config = SchedulerConfig {
        periodic_interval_mins: 45,
    };
    let scheduler = TokenScheduler::with_config(config);
    
    let scheduler_config = scheduler.get_config();
    assert_eq!(scheduler_config.periodic_interval_mins, 45);
}

#[tokio::test]
async fn test_periodic_scheduler_shutdown() {
    let config = SchedulerConfig {
        periodic_interval_mins: 5,
    };
    let scheduler = TokenScheduler::with_config(config);
    
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
    
    scheduler.shutdown();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_periodic_scheduler_drop_behavior() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);
    
    {
        let config = SchedulerConfig {
            periodic_interval_mins: 1,
        };
        let scheduler = TokenScheduler::with_config(config);
        
        scheduler.start_scheduler_simple(move || {
            called_clone.store(true, Ordering::SeqCst);
        });
        
        assert!(scheduler.is_scheduler_active());
    } // Scheduler drops here
    
    // Wait a bit
    sleep(Duration::from_millis(100)).await;
    
    // Callback should not have been called because scheduler was dropped
    assert!(!called.load(Ordering::SeqCst));
}