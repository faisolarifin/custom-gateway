use std::sync::{Arc, Mutex};
use tokio::time::{interval, MissedTickBehavior};
use std::time::Duration;
use crate::providers::StructuredLogger;
use crate::utils::error::Result;

/// Re-export SchedulerConfig from config module
pub use crate::config::SchedulerConfig;

/// Constants for scheduler configuration
const DEFAULT_PERIODIC_INTERVAL_MINS: u64 = 1; // 1 minute (configurable via config.yaml)

/// Token scheduler yang menangani automatic token refresh secara periodik
#[derive(Clone)]
pub struct TokenScheduler {
    periodic_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    config: SchedulerConfig,
}

impl TokenScheduler {
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig {
            periodic_interval_mins: DEFAULT_PERIODIC_INTERVAL_MINS,
        })
    }

    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            periodic_handle: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Start periodic scheduler that runs every configured interval
    pub fn start_scheduler<F, Fut>(&self, refresh_callback: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        // Stop any existing periodic scheduler
        self.stop_scheduler();
        
        let interval_mins = self.config.periodic_interval_mins;
        let handle = self.spawn_periodic_task(interval_mins, refresh_callback);
        
        // Store the new handle
        {
            let mut handle_guard = self.periodic_handle.lock().unwrap();
            *handle_guard = Some(handle);
        }
    }

    /// Start scheduler dengan simple callback - for synchronous operations
    pub fn start_scheduler_simple<F>(&self, refresh_callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let callback = Arc::new(refresh_callback);
        self.start_scheduler(move || {
            let callback = Arc::clone(&callback);
            async move {
                callback();
                Ok(())
            }
        });
    }


    /// Spawn periodic task that runs every interval
    fn spawn_periodic_task<F, Fut>(&self, interval_mins: u64, refresh_callback: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let callback = Arc::new(refresh_callback);
        
        tokio::spawn(async move {
            StructuredLogger::log_info(
                &format!("Starting periodic token refresh scheduler, running every {} minutes", interval_mins),
                None,
                None,
                None,
            );
            
            // Create interval timer with proper behavior
            let mut timer = interval(Duration::from_secs(interval_mins * 60));
            
            // Set behavior to skip missed ticks (if system is busy)
            timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
            
            // Execute immediately on first tick
            timer.tick().await;
            
            loop {
                StructuredLogger::log_info(
                    "Periodic token refresh scheduler triggered - executing refresh callback",
                    None,
                    None,
                    None,
                );
                
                // Execute callback with proper error handling
                match callback().await {
                    Ok(_) => {
                        StructuredLogger::log_info(
                            "Periodic token refresh completed successfully",
                            None,
                            Some("periodic_scheduler"),
                            None,
                        );
                    }
                    Err(e) => {
                        StructuredLogger::log_error(
                            &format!("Periodic token refresh failed: {}", e),
                            None,
                            Some("periodic_scheduler"),
                        );
                    }
                }
                
                StructuredLogger::log_info(
                    &format!("Next token refresh in {} minutes", interval_mins),
                    None,
                    None,
                    None,
                );
                
                // Wait for next interval tick
                timer.tick().await;
            }
        })
    }

    /// Stop scheduler yang sedang berjalan
    pub fn stop_scheduler(&self) {
        let mut handle_guard = self.periodic_handle.lock().unwrap();
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            StructuredLogger::log_info(
                "Periodic token refresh scheduler stopped",
                None,
                None,
                None,
            );
        }
    }

    /// Check apakah scheduler sedang aktif
    pub fn is_scheduler_active(&self) -> bool {
        let handle_guard = self.periodic_handle.lock().unwrap();
        handle_guard.is_some()
    }

    /// Get detailed info tentang scheduler
    pub fn get_scheduler_info(&self) -> Option<String> {
        if self.is_scheduler_active() {
            Some(format!(
                "Periodic token refresh scheduler active (interval: {} minutes)",
                self.config.periodic_interval_mins
            ))
        } else {
            None
        }
    }

    /// Get current scheduler configuration
    pub fn get_config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// Update scheduler configuration (only affects future schedules)
    pub fn update_config(&mut self, config: SchedulerConfig) {
        self.config = config;
    }

    /// Shutdown scheduler secara graceful
    pub fn shutdown(&self) {
        StructuredLogger::log_info(
            "Shutting down TokenScheduler",
            None,
            None,
            None,
        );
        self.stop_scheduler();
    }
}

impl Default for TokenScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TokenScheduler {
    fn drop(&mut self) {
        // Tidak auto-stop scheduler saat drop karena bisa menyebabkan race condition
        // Scheduler harus di-stop secara manual via shutdown() method
    }
}

