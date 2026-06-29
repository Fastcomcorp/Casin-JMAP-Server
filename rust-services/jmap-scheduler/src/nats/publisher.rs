// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: nats::publisher
// Description: Publishes schedule change events to NATS JetStream for Elixir to consume.

use async_nats::Client;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ChangeType {
    Assigned,
    Updated,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShiftChangedEvent {
    pub event_id: Uuid,
    pub shift_id: String, // CalendarEvent uid
    pub employee_id: Uuid,
    pub jmap_account_id: String,
    pub change_type: ChangeType,
    pub new_start: DateTime<Utc>,
    pub new_duration: String,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub ack_deadline_minutes: u32,
}

pub async fn publish_shift_change(client: &Client, event: &ShiftChangedEvent) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(event)?;
    client.publish("scheduling.shift.changed", axum::body::Bytes::from(payload)).await?;
    Ok(())
}
