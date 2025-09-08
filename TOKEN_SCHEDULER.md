# Token Scheduler Enhancement

## Overview
The application now includes a **separate TokenScheduler module** that handles automatic token refresh. This provides better separation of concerns and reusable scheduling functionality for authentication tokens.

## Architecture

### Separated Components
- **TokenScheduler** (`src/services/token_scheduler.rs`) - Standalone scheduler module
- **LoginHandler** (`src/services/permata_login.rs`) - Uses TokenScheduler for automatic refresh
- **Clean separation** - Scheduler logic is independent and reusable

## Key Features

### 1. Automatic Token Refresh
- **Timing**: Token is refreshed 5 minutes before expiration
- **Background**: Runs as a background task using Tokio spawn
- **Proactive**: No waiting for requests to trigger token refresh

### 2. Smart Scheduling
- **Single Scheduler**: Only one scheduler runs at a time
- **Auto Replacement**: New scheduler replaces old one when token is refreshed
- **Safety Checks**: Skips scheduling if token expires too soon (< 1 minute)

### 3. Graceful Shutdown
- **Clean Shutdown**: Scheduler is properly stopped during application shutdown
- **Resource Cleanup**: No hanging background tasks

## How It Works

### Token Flow
1. **First Token Request**: 
   - Fetches token from API
   - Caches token with expiration time
   - **NEW**: Starts scheduler for automatic refresh

2. **Scheduler Triggers**:
   - Sleeps until 5 minutes before expiration
   - Clears token cache
   - Fetches fresh token automatically
   - Starts new scheduler for the new token

3. **Subsequent Requests**:
   - Uses fresh cached token
   - No delays due to expired tokens

### Implementation Details

#### TokenScheduler Module
```rust
// Separated TokenScheduler in src/services/token_scheduler.rs
pub struct TokenScheduler {
    scheduler_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl TokenScheduler {
    // Async callback support for complex operations
    pub fn start_refresh_scheduler_async<F, Fut>(&self, expires_at: Instant, callback: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static;

    // Simple callback support for basic operations  
    pub fn start_refresh_scheduler_simple<F>(&self, expires_at: Instant, callback: F)
    where F: Fn() + Send + Sync + 'static;
}
```

#### LoginHandler Integration
```rust
// In LoginHandler - now uses TokenScheduler
struct LoginHandler {
    // ... existing fields
    token_scheduler: TokenScheduler,  // <-- Separated component
}

// Simplified scheduler startup
fn start_token_refresh_scheduler(&self, expires_at: Instant) {
    let cache = Arc::clone(&self.token_cache);
    let handler_clone = self.clone();

    self.token_scheduler.start_refresh_scheduler_async(expires_at, move || {
        // Async closure for token refresh
        let cache_clone = Arc::clone(&cache);
        let handler = handler_clone.clone();
        
        async move {
            // Clear cache and refresh token
            cache_clone.lock().unwrap().clear();
            handler.get_token_with_context(None, Some("scheduler")).await?;
            Ok(())
        }
    });
}
```

## Benefits

### Core Benefits
1. **Zero Downtime**: No authentication failures due to expired tokens
2. **Better Performance**: No request delays waiting for token refresh  
3. **Proactive Management**: Tokens are refreshed before they're needed

### Architectural Benefits
4. **Separation of Concerns**: Scheduler logic isolated in dedicated module
5. **Reusability**: TokenScheduler can be used for other scheduling needs
6. **Testability**: Scheduler can be unit tested independently
7. **Maintainability**: Clear boundaries between scheduling and authentication logic
8. **Clean Shutdown**: Proper cleanup and resource management

## Configuration

The scheduler is now fully configurable with `SchedulerConfig`:

### Default Configuration
```rust
SchedulerConfig {
    refresh_buffer_secs: 300,     // 5 minutes before expiration
    min_schedule_time_secs: 60,   // Minimum 1 minute to schedule
}
```

### Custom Configuration
```rust
use crate::services::{TokenScheduler, SchedulerConfig};

// Custom timing
let config = SchedulerConfig {
    refresh_buffer_secs: 180,    // 3 minutes buffer
    min_schedule_time_secs: 30,  // 30 seconds minimum
};

let scheduler = TokenScheduler::with_config(config);

// Or update existing scheduler
let mut scheduler = TokenScheduler::new();
scheduler.update_config(SchedulerConfig {
    refresh_buffer_secs: 600,    // 10 minutes buffer
    min_schedule_time_secs: 120, // 2 minutes minimum
});
```

### Configuration Options
- **refresh_buffer_secs**: Time before token expiry to trigger refresh
- **min_schedule_time_secs**: Minimum time required to schedule (prevents immediate expiry)

## Monitoring

The scheduler provides logging for monitoring:
- Scheduler start/stop events
- Token refresh activities
- Error handling for failed refreshes

## Usage

The scheduler works automatically once the application starts. No manual intervention required.

```bash
# Start the application
cargo run

# Scheduler will be visible in logs:
# "Starting token refresh scheduler, will refresh in X seconds"
# "Token refresh by scheduler completed successfully"
```

## Error Handling

- **Authentication Failures**: Logged but don't crash the scheduler
- **Network Issues**: Uses existing retry mechanisms
- **Scheduler Failures**: Logged for debugging

The application remains functional even if the scheduler fails, as tokens can still be refreshed on-demand during regular requests.

## Module Usage Examples

### Using TokenScheduler Directly
```rust
use crate::services::{TokenScheduler, SchedulerConfig};

// Default configuration
let scheduler = TokenScheduler::new();

// Custom configuration
let config = SchedulerConfig {
    refresh_buffer_secs: 120,  // 2 minutes buffer
    min_schedule_time_secs: 30, // 30 seconds minimum
};
let scheduler = TokenScheduler::with_config(config);

// For simple operations
scheduler.start_refresh_scheduler_simple(expires_at, || {
    println!("Token refresh triggered!");
});

// For async operations
scheduler.start_refresh_scheduler_async(expires_at, || async {
    // Perform async token refresh
    some_async_operation().await?;
    Ok(())
});

// Check detailed status
if let Some(info) = scheduler.get_scheduler_info() {
    println!("Scheduler status: {}", info);
}

// Get current configuration
let current_config = scheduler.get_config();
println!("Buffer: {}s, Min schedule: {}s", 
    current_config.refresh_buffer_secs, 
    current_config.min_schedule_time_secs);

// Clean shutdown
scheduler.shutdown();
```

### Integration with Other Services
The TokenScheduler is designed to be reusable for any scheduled refresh operations:

```rust
pub struct SomeService {
    scheduler: TokenScheduler,
    // ... other fields
}

impl SomeService {
    pub fn schedule_refresh(&self, expires_at: Instant) {
        let service = self.clone();
        self.scheduler.start_refresh_scheduler_async(expires_at, move || {
            let service = service.clone();
            async move {
                service.perform_refresh().await
            }
        });
    }
}
```

## File Structure
```
src/services/
├── mod.rs                          # Exports TokenScheduler
├── token_scheduler.rs              # Standalone scheduler module
├── permata_login.rs                # Uses TokenScheduler  
├── permata_callbackstatus_client.rs
└── webhook_processor.rs
```