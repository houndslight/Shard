// imports
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

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

// program loop
fn main() {
    println!("Starting Shard on localhost:8080");

    // http server
    let server = Server::http("0.0.0.0:8080")
        .expect("Server initialization failed!");
    
    // key value store
    let store: Arc<Mutex<HashMap<String, String>>> = 
        Arc::new(Mutex::new(HashMap::new()));

    // loop indefinitely (request handling)
    for mut request in server.incoming_requests() {
        // clones store reference so each request can have access
        let store = Arc::clone(&store);

        // etracts HTTP method (GET, PUT, etc.)
        let method = request.method().clone();

        // extracts url path
        let url = request.url().to_string();


        // health check
        if url == "/health" {
                let response = 
                    Response::from_string("OK");
                let _ = request.respond(response);
                }

            //kv routes

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
                    } else {
                        let response = 
                            Response::from_string("JSON Failed to validate")
                                .with_status_code(400);
                        let _ = request.respond(response);
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
                    } else {
                        let response = 
                            Response::from_string("Key was not found")
                                .with_status_code(404);
                        let _ = request.respond(response);
                    }
                }

                _ => {
                    let response = 
                        Response::from_string("Method not allowed!")
                            .with_status_code(405);
                    let _ = request.respond(response);
                }
            }
        } else {
            // if route is unknown
            let response = 
                Response::from_string("Not found!")
                    .with_status_code(404);
            let _ = request.respond(response);
        }
    }
}
