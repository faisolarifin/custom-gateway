# Webhook Gateway

A high-performance HTTP webhook gateway built in Rust that receives webhook requests and forwards them to configured URLs.

## Features

- HTTP server using standard library (Hyper) in handlers module
- Configurable webhook path and forward URLs
- Request body and headers forwarding
- Health check endpoint
- Structured logging
- Graceful shutdown

## Configuration

The application uses YAML configuration files. Example `config.yaml`:

```yaml
webhook:
  listen_host: "127.0.0.1"
  listen_port: 8080
  webhook_path: "/webhook"
  forward_urls:
    - "http://localhost:3001/webhook"
    - "http://localhost:3002/webhook"
  timeout: 30

logger:
  dir: log/
  file_name: webhook-gateway
  max_backups: 0
  max_size: 10
  max_age: 90
  compress: true
  local_time: true
```

## Usage

1. Build the application:
   ```bash
   cargo build --release
   ```

2. Create your configuration file (`config.yaml`)

3. Run the application:
   ```bash
   cargo run
   ```

4. Run tests:
   ```bash
   # Run all unit tests
   cargo test
   
   # Run built-in test mode
   cargo run -- --test
   
   # Run ignored tests (network-dependent)  
   cargo test -- --ignored
   ```

The server will listen on the configured host and port for webhook requests on the specified path.

## Endpoints

- `POST {webhook_path}` - Main webhook endpoint that forwards requests to configured URLs
- `GET /health` - Health check endpoint

## Request Flow

1. Client sends POST request to webhook endpoint
2. Gateway receives request and extracts headers and body
3. Gateway forwards the request to all configured forward URLs
4. Gateway responds with success/error status

## Changes from RabbitMQ Version

This version has been refactored from a RabbitMQ consumer to a direct HTTP webhook gateway:

- Removed RabbitMQ dependencies (lapin)  
- Removed unused auth and HTTP client configurations
- Added HTTP server using hyper (standard library)
- Direct webhook forwarding instead of queue-based processing
- Configurable webhook path and forward URLs
- Simple configuration with only webhook and logging settings
- Built-in test mode for validation