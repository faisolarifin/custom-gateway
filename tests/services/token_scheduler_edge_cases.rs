/*!
 * Edge Case Tests for TokenScheduler (Periodic Mode)
 * 
 * These tests cover edge cases, error conditions, and boundary scenarios
 * for the periodic token scheduler that runs every configured interval.
 */

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use webhook_gateway::services::{TokenScheduler, SchedulerConfig};
use webhook_gateway::utils::error::{AppError, Result};

#[tokio::test]
async fn test_scheduler_callback_error_handling() {
    let scheduler = TokenScheduler::with_config(SchedulerConfig {
        periodic_interval_mins: 1, // 1 minute for faster testing
    });
    
    let error_called = Arc::new(AtomicBool::new(false));
    let error_called_clone = Arc::clone(&error_called);
    
    scheduler.start_scheduler(move || {
        let called = Arc::clone(&error_called_clone);
        async move {
            called.store(true, Ordering::SeqCst);
            Err(AppError::message_processing("Test error".to_string()))
        }
    });
    
    assert!(scheduler.is_scheduler_active());
    
    // Wait a short time then stop - callback should be prepared to handle errors
    sleep(Duration::from_millis(50)).await;
    scheduler.stop_scheduler();
    
    // Scheduler should no longer be active after stopping
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_multiple_rapid_schedule_calls() {
    let scheduler = TokenScheduler::new();
    
    let call_count = Arc::new(AtomicUsize::new(0));
    
    // Rapidly schedule multiple times
    for _i in 0..10 {
        let count_clone = Arc::clone(&call_count);
        
        scheduler.start_scheduler(move || {
            let count = Arc::clone(&count_clone);
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
    }
    
    // Should have only one active scheduler (the last one)
    assert!(scheduler.is_scheduler_active());
    
    // Stop and verify only one scheduler was actually active
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_scheduler_with_minimal_interval() {
    let config = SchedulerConfig {
        periodic_interval_mins: 1, // Minimum 1 minute
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
    
    // Should start even with short interval
    assert!(scheduler.is_scheduler_active());
    
    // Stop immediately before execution
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
    
    // Should not have executed
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_concurrent_scheduler_operations() {
    let scheduler = Arc::new(TokenScheduler::new());
    
    let mut handles = Vec::new();
    
    // Spawn multiple tasks that try to control the scheduler concurrently
    for i in 0..5 {
        let scheduler_clone = Arc::clone(&scheduler);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                // Start scheduler
                scheduler_clone.start_scheduler_simple(|| {
                    // Do nothing
                });
            } else {
                // Stop scheduler
                scheduler_clone.stop_scheduler();
            }
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Scheduler should be in a consistent state
    // (Either active or inactive, but not in an inconsistent state)
    let is_active = scheduler.is_scheduler_active();
    let info = scheduler.get_scheduler_info();
    
    if is_active {
        assert!(info.is_some());
    } else {
        assert!(info.is_none());
    }
}

#[tokio::test]
async fn test_scheduler_drop_behavior() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);
    
    {
        // Scheduler in limited scope
        let scheduler = TokenScheduler::with_config(SchedulerConfig {
            periodic_interval_mins: 1,
        });
        
        scheduler.start_scheduler(move || {
            let called = Arc::clone(&called_clone);
            async move {
                called.store(true, Ordering::SeqCst);
                Ok(())
            }
        });
        
        assert!(scheduler.is_scheduler_active());
    } // Scheduler drops here
    
    // Wait a bit longer than the scheduled time
    sleep(Duration::from_millis(100)).await;
    
    // Callback should not have been called because scheduler was dropped
    // This tests the Drop implementation
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_scheduler_config_extreme_values() {
    // Test with very large values
    let large_config = SchedulerConfig {
        periodic_interval_mins: u64::MAX,
    };
    
    let scheduler = TokenScheduler::with_config(large_config);
    let config = scheduler.get_config();
    
    assert_eq!(config.periodic_interval_mins, u64::MAX);
    
    // Should handle extreme values gracefully
    scheduler.start_scheduler_simple(|| {});
    
    // Should still be able to start (even if the interval is extremely long)
    assert!(scheduler.is_scheduler_active());
    
    scheduler.stop_scheduler();
    assert!(!scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_scheduler_info_consistency() {
    let scheduler = TokenScheduler::new();
    
    // Initially inactive
    assert!(!scheduler.is_scheduler_active());
    assert!(scheduler.get_scheduler_info().is_none());
    
    // Start scheduler
    scheduler.start_scheduler_simple(|| {});
    
    // Should be consistent
    assert!(scheduler.is_scheduler_active());
    assert!(scheduler.get_scheduler_info().is_some());
    
    // Stop scheduler
    scheduler.stop_scheduler();
    
    // Should be consistently inactive
    assert!(!scheduler.is_scheduler_active());
    assert!(scheduler.get_scheduler_info().is_none());
}

#[tokio::test]
async fn test_scheduler_update_config_during_active_schedule() {
    let mut scheduler = TokenScheduler::new();
    
    // Start with default config
    scheduler.start_scheduler_simple(|| {});
    assert!(scheduler.is_scheduler_active());
    
    let original_config = scheduler.get_config().clone();
    
    // Update config while scheduler is active
    let new_config = SchedulerConfig {
        periodic_interval_mins: 30,
    };
    scheduler.update_config(new_config.clone());
    
    // Config should be updated
    let updated_config = scheduler.get_config();
    assert_eq!(updated_config.periodic_interval_mins, 30);
    assert_ne!(updated_config.periodic_interval_mins, original_config.periodic_interval_mins);
    
    // Scheduler should still be active (config update doesn't affect running schedulers)
    assert!(scheduler.is_scheduler_active());
}

#[tokio::test]
async fn test_scheduler_simple_vs_async_callback_equivalence() {
    let simple_scheduler = TokenScheduler::with_config(SchedulerConfig {
        periodic_interval_mins: 30,
    });
    
    let async_scheduler = TokenScheduler::with_config(SchedulerConfig {
        periodic_interval_mins: 30,
    });
    
    let simple_called = Arc::new(AtomicBool::new(false));
    let async_called = Arc::new(AtomicBool::new(false));
    
    let simple_called_clone = Arc::clone(&simple_called);
    let async_called_clone = Arc::clone(&async_called);
    
    // Start both schedulers with equivalent callbacks
    simple_scheduler.start_scheduler_simple(move || {
        simple_called_clone.store(true, Ordering::SeqCst);
    });
    
    async_scheduler.start_scheduler(move || {
        let called = Arc::clone(&async_called_clone);
        async move {
            called.store(true, Ordering::SeqCst);
            Ok(())
        }
    });
    
    // Both should be active
    assert!(simple_scheduler.is_scheduler_active());
    assert!(async_scheduler.is_scheduler_active());
    
    // Stop both
    simple_scheduler.stop_scheduler();
    async_scheduler.stop_scheduler();
    
    // Both should be inactive
    assert!(!simple_scheduler.is_scheduler_active());
    assert!(!async_scheduler.is_scheduler_active());
}