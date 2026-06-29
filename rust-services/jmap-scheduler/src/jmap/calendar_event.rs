// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: jmap::calendar_event
// Description: Handlers for CalendarEvent/get and CalendarEvent/set method calls. Manages optimistic concurrency via If-In-State checks.

use crate::models::calendar_event::CalendarEvent;
use serde_json::{json, Value};
use std::collections::HashMap;

pub async fn handle_get(args: &HashMap<String, Value>) -> Value {
    let account_id = args.get("accountId").and_then(|v| v.as_str()).unwrap_or("");
    let ids = args.get("ids").and_then(|v| v.as_array());

    // Dummy implementation: return empty list or mock event
    let list: Vec<Value> = Vec::new(); // Ideally fetch from DB here
    
    json!({
        "accountId": account_id,
        "state": "sync-state-1",
        "list": list,
        "notFound": []
    })
}

pub async fn handle_set(args: &HashMap<String, Value>) -> Value {
    let account_id = args.get("accountId").and_then(|v| v.as_str()).unwrap_or("");
    let if_in_state = args.get("ifInState").and_then(|v| v.as_str());

    // Optimistic Concurrency Control (If-In-State)
    // In a real app, query the DB for the current `sync_state` of the account/calendar
    // If it doesn't match `if_in_state`, we should immediately reject with a stateMismatch error
    let current_db_state = "sync-state-1";
    if let Some(state) = if_in_state {
        if state != current_db_state {
            return json!({
                "type": "stateMismatch"
            });
        }
    }
    
    // Parse creates, updates, destroys
    let create_args = args.get("create").and_then(|v| v.as_object());
    
    let mut created: HashMap<String, Value> = HashMap::new();
    let mut updated: HashMap<String, Value> = HashMap::new();
    let mut destroyed: Vec<String> = Vec::new();
    
    let mut not_created: HashMap<String, Value> = HashMap::new();
    let mut not_updated: HashMap<String, Value> = HashMap::new();
    let mut not_destroyed: HashMap<String, Value> = HashMap::new();

    // ---------------------------------------------------------
    // Option A: Batch Processing with Partial Success Handling
    // ---------------------------------------------------------
    if let Some(creates) = create_args {
        for (client_id, ev_value) in creates {
            // Attempt to parse the JSCalendar event
            let ev: Result<CalendarEvent, _> = serde_json::from_value(ev_value.clone());
            
            match ev {
                Ok(mut server_event) => {
                    server_event.uid = uuid::Uuid::new_v4().to_string();
                    server_event.updated = chrono::Utc::now();

                    // SQL PSEUDOCODE: 
                    // Let result = try_insert_event_and_participants(&pool, &server_event).await;
                    let result: Result<(), String> = Err("database exclusion constraint violation".to_string()); // Mocking a failure for Employee 3

                    match result {
                        Ok(_) => {
                            // Success: Add to the 'created' map
                            // nats::publisher::publish_shift_change(...)
                            created.insert(client_id.clone(), json!(server_event));
                        }
                        Err(db_err) => {
                            // If the error is our Ironclad Guarantee (double-booking)
                            if db_err.contains("exclusion constraint") {
                                not_created.insert(client_id.clone(), json!({
                                    "type": "invalidProperties",
                                    "description": "Double-booking detected. The employee is already scheduled for an overlapping shift.",
                                    "properties": ["x-shift-employee-id", "start", "duration"]
                                }));
                            } else {
                                // Generic server error
                                not_created.insert(client_id.clone(), json!({
                                    "type": "serverFail",
                                    "description": db_err
                                }));
                            }
                        }
                    }
                }
                Err(err) => {
                    not_created.insert(client_id.clone(), json!({
                        "type": "invalidArguments",
                        "description": format!("Failed to parse JSCalendar: {}", err)
                    }));
                }
            }
        }
    }

    json!({
        "accountId": account_id,
        "oldState": "sync-state-1",
        "newState": "sync-state-2",
        "created": created,
        "updated": updated,
        "destroyed": destroyed,
        "notCreated": not_created,
        "notUpdated": not_updated,
        "notDestroyed": not_destroyed
    })
}
