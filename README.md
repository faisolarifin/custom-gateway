# Permata Gateway

Webhook gateway service untuk integrasi dengan Permata Bank API. Menerima webhook dari WhatsApp Business API dan meneruskan ke Permata Bank callback status endpoint dengan proper authentication dan retry mechanism.

# Docs
- [SonarQube](https://sonarqube.jatismobile.com/dashboard?id=waba-integrate_PermataGateway)

## Version History

- v0.1.0 (Initial Release):

```
- Webhook server dengan filtering payload DR dan Inbound Flow
- Integrasi dengan Permata Bank Login API (OAuth2)
- Automatic token refresh dengan scheduler
- Structured logging dengan daily rotation
- Retry mechanism untuk HTTP requests
- HMAC signature generation
```


## Requirements
- Rust 1.75+ untuk development
- Docker
- Access ke Permata Bank API endpoints

## Endpoints
```python 
## webhook receiver
POST /webhook

## health check
GET /webhook
```

## Development
```bash
cp config.yaml.example config.yaml
cargo run
```

## Testing
### Unit test only
```bash
cargo test
```
### Unit test dengan verbose output
```bash
cargo test -- --nocapture
```
## Deployment (Docker Build)
- Copy (create) config file refer to `config.yaml.example`
```bash
cp config.yaml.example config.yaml
```
- Update config if required seperti (Permata Bank API endpoints, credentials, server port)
- Build the app using docker
```bash
docker build -t permata-gateway:{VERSION} .
```
- Run the app using docker
```bash
docker run -d --name permata-gateway -v "$(pwd)"/log:/app/log -p {YOUR_OPEN_PORT}:8080 permata-gateway:{VERSION}
```
- Check your service is running using `docker ps` or access GET {server}:{port}/webhook

## Configuration
The application uses `config.yaml` for configuration. Key settings include:
- **Server**: Listen host, port dan webhook path configuration
- **WebClient**: HTTP timeout, retry mechanism configuration  
- **Permata Bank Login**: OAuth2 credentials dan token endpoint
- **Permata Bank Webhook**: Callback status URL dan organization name
- **Token Scheduler**: Automatic token refresh interval
- **Logger**: Structured logging dengan daily rotation dan compression

## Architecture
- **Webhook Server**: Built dengan Hyper untuk high-performance HTTP handling
- **Authentication**: OAuth2 token management dengan automatic refresh
- **Payload Filtering**: JSON path-based filtering untuk DR dan Inbound Flow payloads
- **Retry Mechanism**: Configurable retry untuk failed requests
- **Logging**: Structured logging dengan JSON format dan file rotation
- **Signature**: HMAC-SHA256 signature generation untuk API security
