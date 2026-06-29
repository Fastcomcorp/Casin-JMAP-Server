// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: auth::rights
// Description: Enforces JMAP 'myRights' rules. Ensures Employees can only access their own schedules, while CX Agents and Management have appropriate privileges.

use axum::http::StatusCode;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Management,
    CxAgent,
    Employee,
    Customer,
}

#[derive(Debug, Clone)]
pub struct Claims {
    pub employee_id: Uuid,
    pub jmap_account_id: String,
    pub role: Role,
}

pub fn check_rights_for_account(claims: &Claims, target_account_id: &str, method: &str) -> Result<(), StatusCode> {
    match claims.role {
        Role::Management => {
            // Management can query any accountId
            Ok(())
        }
        Role::Employee => {
            // Employee can only access their own accountId
            if claims.jmap_account_id == target_account_id {
                Ok(())
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Role::CxAgent => {
            // CX Agent can call Principal/getAvailability but NOT CalendarEvent/get
            if method.starts_with("Principal/getAvailability") {
                Ok(())
            } else if method.starts_with("CalendarEvent/set") {
                // They might have access to set based on rules, simplified here
                Ok(())
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Role::Customer => Err(StatusCode::FORBIDDEN),
    }
}
