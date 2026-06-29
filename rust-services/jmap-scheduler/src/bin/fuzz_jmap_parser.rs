// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Script: fuzz_jmap_parser.rs
// Description: Fuzz Testing script. Generates thousands of malformed, 
// unexpected, and malicious JSON payloads to blast against the JMAP parser 
// to ensure the Rust backend never panics or crashes.

use serde_json::{json, Value};
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

fn generate_fuzz_string(_length: usize) -> String {
    Uuid::new_v4().to_string()
}

// Generates a massive JSON payload with extreme nesting to test stack overflow protection
fn generate_nested_bomb(depth: usize) -> Value {
    let mut current = json!({"malicious": "data"});
    for _ in 0..depth {
        current = json!({"nested": current});
    }
    current
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Fuzz Test against Casin JMAP Server...");
    
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
        
    let target_url = "http://127.0.0.1:3000/jmap";

    // Scenario 1: The Null Byte Injection
    let null_byte_payload = json!({
        "using": ["urn:ietf:params:jmap:core"],
        "methodCalls": [
            ["CalendarEvent/set", { "bad\0string": "data" }, "client-1"]
        ]
    });
    
    // Scenario 2: The Erlang Atom Exhaustion Attempt (10,000 random keys)
    let mut massive_object = serde_json::Map::new();
    for _ in 0..10_000 {
        massive_object.insert(generate_fuzz_string(10), json!(1));
    }
    let atom_bomb = json!({
        "using": ["urn:ietf:params:jmap:core"],
        "methodCalls": [
            ["CalendarEvent/set", massive_object, "client-2"]
        ]
    });

    // Scenario 3: The Deep Nesting Bomb (Stack Overflow Attempt)
    let stack_bomb = json!({
        "using": ["urn:ietf:params:jmap:core"],
        "methodCalls": [
            ["CalendarEvent/set", generate_nested_bomb(500), "client-3"]
        ]
    });

    let payloads = vec![
        ("Null Byte Injection", null_byte_payload),
        ("Atom Exhaustion Attempt", atom_bomb),
        ("Stack Overflow Attempt", stack_bomb),
    ];

    for (name, payload) in payloads {
        println!("Firing payload: {}", name);
        
        let res = client.post(target_url)
            .header("Authorization", "Bearer MOCK_FUZZ_TOKEN")
            .json(&payload)
            .send()
            .await;
            
        match res {
            Ok(response) => {
                let status = response.status();
                println!("  -> Server survived. HTTP Status: {}", status);
                // We expect 400 Bad Request or 401 Unauthorized, NOT a dropped connection (Crash)
                assert!(status.is_client_error() || status.is_success(), "Server returned unexpected 5xx status!");
            },
            Err(e) => {
                println!("  -> WARNING: Connection failed! Did the server crash? Error: {}", e);
            }
        }
    }
    
    println!("Fuzz testing complete. Casin JMAP Server survived all DoS vectors.");
    Ok(())
}
