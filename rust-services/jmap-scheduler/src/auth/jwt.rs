// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: auth::jwt
// Description: Validates incoming JWT tokens from API requests, extracting the Fastcomcorp single sign-on identities.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode, header::AUTHORIZATION},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct JmapUser {
    pub account_id: String,
    pub employee_id: String,
}

pub struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "type": "urn:ietf:params:jmap:error:notAuthorized",
            "status": 401,
            "detail": "Invalid or missing Bearer token"
        }));
        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for JmapUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(auth_header) if auth_header.starts_with("Bearer ") => {
                let token = &auth_header["Bearer ".len()..];
                
                // In production, we would use jsonwebtoken::decode here to mathematically
                // verify the RSA/HMAC signature against the corporate SSO provider.
                // For now, if a token is provided, we mock the extracted claims.
                if token.is_empty() {
                    return Err(AuthError);
                }

                Ok(JmapUser {
                    account_id: "default_account_id".to_string(),
                    employee_id: "mock-employee-uuid".to_string(),
                })
            }
            _ => Err(AuthError),
        }
    }
}
