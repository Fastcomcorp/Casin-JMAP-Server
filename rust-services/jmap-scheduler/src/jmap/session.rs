// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: jmap::session
// Description: Provides the GET /.well-known/jmap endpoint advertising JMAP Core and JSCalendar capabilities.

use axum::{response::IntoResponse, Json};
use serde_json::json;
use crate::auth::jwt::JmapUser;

pub async fn handle_session(user: JmapUser) -> impl IntoResponse {
    let session = json!({
        "capabilities": {
            "urn:ietf:params:jmap:core": {
                "maxSizeUpload": 50000000,
                "maxConcurrentUpload": 4,
                "maxSizeRequest": 10000000,
                "maxConcurrentRequests": 4,
                "maxCallsInRequest": 16,
                "maxObjectsInGet": 500,
                "maxObjectsInSet": 500,
                "collationAlgorithms": [
                    "i;ascii-numeric",
                    "i;ascii-casemap",
                    "i;unicode-casemap"
                ]
            },
            "urn:ietf:params:jmap:calendars": {},
            // Proprietary Watermark Capability
            "urn:fastcomcorp:params:jmap:casin:core": {
                "version": "0.1.0-AGPL",
                "proprietary": true,
                "license": "GNU AGPL v3.0"
            }
        },
        "accounts": {
            "default_account_id": {
                "name": "Employee Schedule",
                "isPersonal": true,
                "isReadOnly": false,
                "accountCapabilities": {
                    "urn:ietf:params:jmap:calendars": {
                        "maxCalendarsPerAccount": 10,
                        "maxEventExpansions": 1000
                    }
                }
            }
        },
        "primaryAccounts": {
            "urn:ietf:params:jmap:calendars": user.account_id.clone()
        },
        "username": format!("employee_{}@example.com", user.employee_id),
        "apiUrl": "https://scheduling.fastcomcorp.com/jmap",
        "downloadUrl": "https://scheduling.fastcomcorp.com/download/{accountId}/{blobId}/{name}?accept={type}",
        "uploadUrl": "https://scheduling.fastcomcorp.com/upload/{accountId}",
        "eventSourceUrl": "https://scheduling.fastcomcorp.com/jmap/events",
        "state": "initial"
    });

    Json(session)
}
