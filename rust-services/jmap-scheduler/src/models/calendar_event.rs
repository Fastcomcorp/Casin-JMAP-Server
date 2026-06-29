// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: models::calendar_event
// Description: Implements strict RFC 8984 JSCalendar data models including Recurrence (RRULE) and enterprise scheduling attributes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "@type")]
pub enum JSCalendarObject {
    #[serde(rename = "Event")]
    Event(CalendarEvent),
    #[serde(rename = "Task")]
    Task(CalendarTask),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarTask {
    pub uid: String,
    pub updated: DateTime<Utc>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Confirmed,
    Cancelled,
    Tentative,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyLevel {
    Public,
    Private,
    Secret,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FreeBusyStatus {
    Free,
    Busy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Participant {
    #[serde(rename = "@type")]
    pub participant_type: String, // "Participant"
    pub name: Option<String>,
    pub email: Option<String>,
    pub roles: Option<HashMap<String, bool>>,
    pub schedule_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShiftAck {
    pub acknowledged_at: DateTime<Utc>,
    pub acknowledged_by: Uuid,
    pub device_label: Option<String>,
}

// -----------------------------------------
// Recurrence Structures (RFC 8984)
// -----------------------------------------
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NDays {
    #[serde(rename = "@type")]
    pub n_days_type: String, // "NDays"
    pub day: String,         // e.g., "mo", "tu"
    pub nth_of_period: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    Yearly,
    Monthly,
    Weekly,
    Daily,
    Hourly,
    Minutely,
    Secondly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecurrenceRule {
    #[serde(rename = "@type")]
    pub rule_type: String, // "RecurrenceRule"
    pub frequency: Frequency,
    pub interval: Option<u32>,
    pub rscale: Option<String>,
    pub skip: Option<String>,
    pub first_day_of_week: Option<String>,
    pub by_day: Option<Vec<NDays>>,
    pub by_month_day: Option<Vec<i32>>,
    pub by_month: Option<Vec<String>>,
    pub by_year_day: Option<Vec<i32>>,
    pub by_week_no: Option<Vec<i32>>,
    pub by_hour: Option<Vec<u32>>,
    pub by_minute: Option<Vec<u32>>,
    pub by_second: Option<Vec<u32>>,
    pub by_set_position: Option<Vec<i32>>,
    pub count: Option<u32>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub uid: String,
    pub updated: DateTime<Utc>,
    pub title: Option<String>,
    pub description: Option<String>,
    
    // Time mapping
    pub start: DateTime<Utc>,
    pub duration: Option<String>, // ISO 8601 duration e.g. "PT8H"
    pub time_zone: Option<String>, // IANA timezone, validated on insert
    
    // Status
    pub status: Option<EventStatus>,
    pub privacy: Option<PrivacyLevel>,
    pub free_busy_status: Option<FreeBusyStatus>,
    
    // Sharing
    pub participants: Option<HashMap<String, Participant>>,

    // Recurrence
    pub recurrence_id: Option<DateTime<Utc>>, // Used for instances of a recurring event
    pub recurrence_rules: Option<Vec<RecurrenceRule>>,
    pub excluded_recurrence_rules: Option<Vec<RecurrenceRule>>,
    pub recurrence_overrides: Option<HashMap<String, CalendarEvent>>, // Patch objects for instances

    // Custom scheduling extension properties
    #[serde(rename = "x-shift-employee-id", skip_serializing_if = "Option::is_none")]
    pub shift_employee_id: Option<Uuid>,

    #[serde(rename = "x-shift-role", skip_serializing_if = "Option::is_none")]
    pub shift_role: Option<String>,

    #[serde(rename = "x-shift-ack", skip_serializing_if = "Option::is_none")]
    pub shift_ack: Option<ShiftAck>,

    #[serde(rename = "x-coverage-rule-id", skip_serializing_if = "Option::is_none")]
    pub coverage_rule_id: Option<Uuid>,

    #[serde(rename = "x-location-id", skip_serializing_if = "Option::is_none")]
    pub location_id: Option<Uuid>,
}
