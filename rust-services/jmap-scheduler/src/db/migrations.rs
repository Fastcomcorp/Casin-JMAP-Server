// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: db::migrations
// Description: Contains PostgreSQL schema initialization for calendars, events, sync states, and employee identity.

use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
-- Enable btree_gist for exclusion constraints combining = and &&
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- Core employee identity table
CREATE TABLE IF NOT EXISTS employees (
    employee_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jmap_account_id     TEXT NOT NULL UNIQUE,
    full_name           TEXT NOT NULL,
    email               TEXT NOT NULL UNIQUE,
    phone               TEXT,
    role                TEXT NOT NULL CHECK (role IN ('management', 'cx_agent', 'employee', 'customer')),
    timezone            TEXT NOT NULL DEFAULT 'UTC',
    status              TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'onboarding')),
    auth_provider_id    TEXT UNIQUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- JSCalendar events stored as JSONB for schema flexibility
CREATE TABLE IF NOT EXISTS calendar_events (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uid                 TEXT NOT NULL UNIQUE,
    account_id          TEXT NOT NULL,
    calendar_id         UUID NOT NULL,
    event_data          JSONB NOT NULL,
    privacy             TEXT NOT NULL DEFAULT 'private',
    state_string        TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_calendar_events_account ON calendar_events(account_id);

-- Junction table to support multi-participant events and single grouped shifts
-- ENTERPRISE UPGRADE: Partitioned by RANGE (shift_start) to support 130M+ rows
CREATE TABLE IF NOT EXISTS event_participants (
    participant_id      UUID DEFAULT gen_random_uuid(),
    event_id            UUID NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    employee_id         UUID NOT NULL REFERENCES employees(employee_id) ON DELETE CASCADE,
    participant_status  TEXT NOT NULL DEFAULT 'needs-action',
    shift_start         TIMESTAMPTZ NOT NULL, -- Partition Key
    shift_end           TIMESTAMPTZ NOT NULL, 
    shift_ack_at        TIMESTAMPTZ,
    
    -- Partition keys MUST be included in Primary Keys and Unique constraints
    PRIMARY KEY (participant_id, shift_start),
    UNIQUE(event_id, employee_id, shift_start)
) PARTITION BY RANGE (shift_start);

-- Create initial monthly partitions for 2026
CREATE TABLE IF NOT EXISTS event_participants_2026_06 PARTITION OF event_participants FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');
CREATE TABLE IF NOT EXISTS event_participants_2026_07 PARTITION OF event_participants FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');
CREATE TABLE IF NOT EXISTS event_participants_2026_08 PARTITION OF event_participants FOR VALUES FROM ('2026-08-01') TO ('2026-09-01');

-- The Ironclad Guarantee must be applied to the partitioned table
ALTER TABLE event_participants ADD CONSTRAINT prevent_double_booking EXCLUDE USING gist (
    employee_id WITH =,
    tstzrange(shift_start, shift_end) WITH &&
);

CREATE INDEX IF NOT EXISTS idx_event_participants_employee ON event_participants(employee_id);

-- Trigger to keep denormalized times in sync
CREATE OR REPLACE FUNCTION sync_event_participant_times()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE event_participants
    SET shift_start = (NEW.event_data->>'start')::timestamptz,
        shift_end = (NEW.event_data->>'start')::timestamptz + (NEW.event_data->>'duration')::interval
    WHERE event_id = NEW.id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_sync_event_times
AFTER UPDATE OF event_data ON calendar_events
FOR EACH ROW
EXECUTE FUNCTION sync_event_participant_times();
CREATE INDEX IF NOT EXISTS idx_calendar_events_start ON calendar_events((event_data->>'start'));

-- Calendars (each employee has at least one)
CREATE TABLE IF NOT EXISTS calendars (
    calendar_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          TEXT NOT NULL,
    name                TEXT NOT NULL DEFAULT 'Work Schedule',
    color               TEXT DEFAULT '#0052CC',
    my_rights           JSONB NOT NULL DEFAULT '{"mayReadItems":true,"mayWriteAll":false}',
    is_default          BOOLEAN DEFAULT true,
    sync_state          UUID NOT NULL DEFAULT gen_random_uuid(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- External calendar links per employee (Google, Microsoft, Apple)
CREATE TABLE IF NOT EXISTS employee_linked_calendars (
    link_id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id         UUID NOT NULL REFERENCES employees(employee_id) ON DELETE CASCADE,
    provider            TEXT NOT NULL CHECK (provider IN ('google', 'microsoft', 'apple_caldav')),
    external_calendar_id TEXT NOT NULL,
    oauth_access_token  TEXT NOT NULL,
    oauth_refresh_token TEXT NOT NULL,
    oauth_expires_at    TIMESTAMPTZ NOT NULL,
    sync_enabled        BOOLEAN NOT NULL DEFAULT true,
    last_synced_at      TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(employee_id, provider)
);

-- Web Push + WebSocket tokens per device
CREATE TABLE IF NOT EXISTS employee_push_tokens (
    token_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id         UUID NOT NULL REFERENCES employees(employee_id) ON DELETE CASCADE,
    channel             TEXT NOT NULL CHECK (channel IN ('web_push', 'websocket')),
    token               TEXT NOT NULL,
    device_label        TEXT,
    last_used_at        TIMESTAMPTZ DEFAULT NOW(),
    is_active           BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- CX scheduling rules (JSON-configurable)
CREATE TABLE IF NOT EXISTS scheduling_rules (
    rule_id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT NOT NULL,
    rule_data           JSONB NOT NULL,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit trail — every schedule change
CREATE TABLE IF NOT EXISTS schedule_audit_log (
    audit_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_uid           TEXT NOT NULL,
    changed_by          UUID NOT NULL REFERENCES employees(employee_id),
    change_type         TEXT NOT NULL,
    old_data            JSONB,
    new_data            JSONB,
    changed_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
