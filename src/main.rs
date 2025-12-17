// imports
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

// uptime tracking imports
use std::time::Instant;

// simple http implimentation
use tiny_http::{Server, Response, Method};

// json to rust structs
use serde::{Deserialize, Serialize};

// data structures

// PUT REQUEST
#[derive(Serialize, Deserialize)]
struct PutRequest {
    value: String,
}

// GET RESPONSE
#[derive(Serialize, Deserialize)]
struct GetResponse {
    value: String,
}

// HEALTH RESPONSE
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    uptime_seconds: u64,
    keys: usize,
}

fn log_request(method: &Method, path: &str, status: u16, start: Instant) {

    let latency_ms = start.elapsed().as_millis();
    println!(
        "INFO method={:?} path={} status={} latency_ms={}",
        method, path, status, latency_ms
    );
}

// program loop
fn main() {
    println!("Starting Shard on localhost:8080");

    // uptime check
    let start_time = Instant::now();


    // http server
    let server = Server::http("0.0.0.0:8080")
        .expect("Server initialization failed!");
    
    // key value store
    let store: Arc<Mutex<HashMap<String, String>>> = 
        Arc::new(Mutex::new(HashMap::new()));

    // loop indefinitely (request handling)
    for mut request in server.incoming_requests() {

        // logging calls
        let request_start = Instant::now();

        // clones store reference so each request can have access
        let store = Arc::clone(&store);

        // etracts HTTP method (GET, PUT, etc.)
        let method = request.method().clone();

        // extracts url path
        let url = request.url().to_string();


        // health check
        if url == "/health" {

            let uptime = start_time.elapsed().as_secs();
            let key_count = store.lock().unwrap().len();

            let body = serde_json::to_string(&HealthResponse {
                status: "ok".to_string(),
                service: "shard".to_string(),
                version: "0.1.0".to_string(),
                uptime_seconds: uptime,
                keys: key_count,
            })
            .unwrap();

            let response = Response::from_string(body);
            let _ = request.respond(response);

            // health endpoint logging
            log_request(&method, &url, 200, request_start);
        }

        // metrics endpoint
        else if url == "/metrics" {
            let uptime = start_time.elapsed().as_secs();
            let key_count = store.lock().unwrap().len();

            let body = format!(
                "shard_uptime_seconds {}\nshard_keys {}\n",
                uptime,
                key_count
            );

            let response = Response::from_string(body);
            let _ = request.respond(response);

            // metrics endpoint logging
            log_request(&method, &url, 200, request_start);
        }

        // kv route checks
        else if url.starts_with("/kv/") {
            // URL key extraction
            let key = url.replace("/kv/", "");

            match method {
                // PUT
                Method::Put => {
                    // request body
                    let mut body = String::new();
                    request
                        .as_reader()
                        .read_to_string(&mut body)
                        .unwrap();

                    // json parsing
                    let parsed: Result<PutRequest, _> = 
                        serde_json::from_str(&body);

                    if let Ok(put_req) = parsed {
                        // store lock for writing
                        let mut map = store.lock().unwrap();
                        map.insert(key, put_req.value);

                        let response =
                            Response::from_string("OK");
                        let _ = request.respond(response);

                        // put success case
                        log_request(&method, &url, 200, request_start);
                    } else {
                        let response = 
                            Response::from_string("JSON Failed to validate")
                                .with_status_code(400);
                        let _ = request.respond(response);

                        // fail case: bad json
                        log_request(&method, &url, 400, request_start);
                    }
                }

                // GET
                Method::Get => {
                    // store lock for read
                    let map = store.lock().unwrap();

                    if let Some(value) = map.get(&key) {
                        let response_body = 
                            serde_json::to_string(&GetResponse {
                                value: value.clone(),
                            })
                            .unwrap();

                        let response = 
                            Response::from_string(response_body);
                        let _ = request.respond(response);

                        // get kv key success case
                        log_request(&method, &url, 200, request_start);
                    } else {
                        let response = 
                            Response::from_string("Key was not found")
                                .with_status_code(404);
                        let _ = request.respond(response);

                        // kv key fail case
                        log_request(&method, &url, 404, request_start);
                    }
                }

                _ => {
                    let response = 
                        Response::from_string("Method not allowed!")
                            .with_status_code(405);
                    let _ = request.respond(response);

                    // method not allowed
                    log_request(&method, &url, 405, request_start);
                }
            }
        } else {
            // if route is unknown
            let response = 
                Response::from_string("Not found!")
                    .with_status_code(404);
            let _ = request.respond(response);

            // unknown route
            log_request(&method, &url, 404, request_start);
        }
    }
}
