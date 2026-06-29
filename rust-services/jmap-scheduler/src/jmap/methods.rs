// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: jmap::methods
// Description: Implements the core RFC 8620 JMAP request dispatcher. Parses methodCalls and handles JSON Pointer back-references for createdIds.

use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JmapRequest {
    pub using: Vec<String>,
    pub method_calls: Vec<MethodCall>,
    pub created_ids: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub method: String,
    pub arguments: HashMap<String, Value>,
    pub client_id: String,
}

impl<'de> Deserialize<'de> for MethodCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Vec<Value> = Vec::deserialize(deserializer)?;
        if v.len() != 3 {
            return Err(serde::de::Error::custom("method call must have 3 elements"));
        }
        let method = v[0].as_str().unwrap_or("").to_string();
        let arguments = serde_json::from_value(v[1].clone()).unwrap_or_default();
        let client_id = v[2].as_str().unwrap_or("").to_string();

        Ok(MethodCall {
            method,
            arguments,
            client_id,
        })
    }
}

// Result references logic
fn resolve_back_references(
    args: &mut HashMap<String, Value>,
    created_ids: &HashMap<String, String>,
) {
    // In RFC 8620, a client can use #property to refer to a previous result
    // For simplicity, we just look for keys starting with # and replace them.
    // In a full implementation like Stalwart, JSON pointers (RFC 6901) are used.
    let mut to_insert = HashMap::new();
    let mut to_remove = Vec::new();

    for (k, v) in args.iter() {
        if k.starts_with('#') {
            to_remove.push(k.clone());
            let real_key = &k[1..];
            
            // if v contains a result reference, we should replace it.
            // Simplified: if v is a string that exists in created_ids, replace it.
            if let Some(v_str) = v.as_str() {
                if let Some(resolved_id) = created_ids.get(v_str) {
                    to_insert.insert(real_key.to_string(), Value::String(resolved_id.clone()));
                }
            }
        }
    }

    for k in to_remove {
        args.remove(&k);
    }
    for (k, v) in to_insert {
        args.insert(k, v);
    }
}

/// Standard HTTP JMAP Dispatcher
/// 
/// Note on ArcRTC: While this endpoint receives standard HTTP POST requests from clients
/// when they have internet access, the JmapRequest payloads here may actually be aggregated
/// offline batches. If clients use ArcRTC (WebRTC Data Channels) to peer-to-peer sync in 
/// an offline environment, the master device will eventually send the final reconciled 
/// JmapRequest payload to this exact function once internet is restored.
pub async fn handle_jmap(user: crate::auth::jwt::JmapUser, Json(mut payload): Json<JmapRequest>) -> impl IntoResponse {
    let mut method_responses = Vec::new();
    let mut created_ids = payload.created_ids.unwrap_or_default();

    for call in payload.method_calls {
        let mut args = call.arguments.clone();
        
        // Resolve RFC 8620 back references before passing to handler
        resolve_back_references(&mut args, &created_ids);

        let response_args = match call.method.as_str() {
            "CalendarEvent/get" => crate::jmap::calendar_event::handle_get(&args).await,
            "CalendarEvent/set" => {
                let res = crate::jmap::calendar_event::handle_set(&args).await;
                // If the set call created new objects, add them to created_ids for future references
                if let Some(created) = res.get("created").and_then(|v| v.as_object()) {
                    for (client_id, server_obj) in created {
                        if let Some(server_id) = server_obj.get("id").and_then(|id| id.as_str()) {
                            created_ids.insert(client_id.clone(), server_id.to_string());
                        }
                    }
                }
                res
            },
            _ => json!({
                "type": "unknownMethod"
            }),
        };

        // If the response is an error, the method name is usually "error"
        let method_name = if response_args.get("type").is_some() && response_args.get("type").unwrap().as_str() == Some("unknownMethod") {
            "error"
        } else {
            call.method.as_str()
        };

        let method_response = vec![
            Value::String(method_name.to_string()),
            response_args,
            Value::String(call.client_id),
        ];
        method_responses.push(method_response);
    }

    let response = json!({
        "methodResponses": method_responses,
        "createdIds": created_ids,
        "sessionState": "sync-state-1"
    });

    Json(response)
}
