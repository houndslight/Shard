# Shard

Shard is a small Rust HTTP service that implements a thread-safe, in-memory key value store.

The focus of this project is core backend fundamentals: request handling, concurrency, observability, and clear system behavior. The implementation is intentionally minimal so the important pieces are easy to reason about.

## Features

- In-memory key value storage
- Thread-safe shared state using `Arc<Mutex<>>`
- HTTP API with GET and PUT support
- `/health` endpoint with uptime and key count
- `/metrics` endpoint with Prometheus-style metrics
- Per-request logging with latency and status codes

## API

### Health check

```
GET /health
```

Returns service status and runtime information.

Example:
```json
{
  "status": "ok",
  "service": "shard",
  "version": "0.1.0",
  "uptime_seconds": 123,
  "keys": 5
}
```

### Metrics

```
GET /metrics
```

Prometheus-compatible metrics output.

Example:
```
shard_uptime_seconds 123
shard_keys 5
```

### Key value operations

#### Store a value

```
PUT /kv/<key>
```

Body:
```json
{
  "value": "hello world"
}
```

Response:
```
OK
```

#### Fetch a value

```
GET /kv/<key>
```

Response:
```json
{
  "value": "hello world"
}
```

Returns `404` if the key does not exist.

## Logging

Each request is logged with method, path, status code, and latency in milliseconds.

Example:
```
INFO method=Get path=/kv/test status=200 latency_ms=3
```

This mirrors the basic logging patterns used in production services.

## Implementation details

- Language: Rust
- HTTP server: `tiny_http`
- Serialization: `serde`
- Storage: in-memory `HashMap`
- Concurrency: `Arc<Mutex<>>`

No framework, async runtime, or database is used. This is a deliberate choice to keep behavior explicit and easy to inspect.

## Running locally

```
cargo run
```

The server listens on:
```
http://localhost:8080
```

## Purpose

Shard exists as a learning project to demonstrate how a simple backend service is built from first principles. It emphasizes correctness, clarity, and observability over feature depth.

## License

MIT
