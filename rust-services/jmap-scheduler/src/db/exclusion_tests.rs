// Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
// This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
// See the LICENSE file in the root directory for more details.

// Module: db::exclusion_tests
// Description: Automated integration tests verifying the Ironclad Guarantee (tstzrange overlapping).

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use std::env;
    use uuid::Uuid;

    // Helper to insert a participant and return Result
    async fn insert_participant(
        pool: &PgPool,
        event_id: Uuid,
        employee_id: Uuid,
        start: &str,
        end: &str,
    ) -> Result<(), sqlx::Error> {
        let query = r#"
            INSERT INTO event_participants (event_id, employee_id, shift_start, shift_end)
            VALUES ($1, $2, $3::timestamptz, $4::timestamptz)
        "#;
        sqlx::query(query)
            .bind(event_id)
            .bind(employee_id)
            .bind(start)
            .bind(end)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Requires active DATABASE_URL to run
    async fn test_ironclad_guarantee() {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&db_url).await.unwrap();

        // Run migrations
        crate::db::migrations::run_migrations(&pool).await.unwrap();

        // ---------------------------------------------------------
        // SETUP: Seed 3 Employees and 3 Events
        // ---------------------------------------------------------
        let emp1 = Uuid::new_v4();
        let emp2 = Uuid::new_v4();
        let emp3 = Uuid::new_v4();

        // Insert mock employees (assuming basic fields, skipping null constraints for brevity in test)
        for emp_id in &[emp1, emp2, emp3] {
            sqlx::query("INSERT INTO employees (employee_id, jmap_account_id, full_name, email, role) VALUES ($1, $2, 'Test', $2, 'employee')")
                .bind(emp_id)
                .bind(emp_id.to_string())
                .execute(&pool).await.unwrap();
        }

        let evt1 = Uuid::new_v4();
        let evt2 = Uuid::new_v4();
        
        sqlx::query("INSERT INTO calendars (account_id) VALUES ('test-acc') RETURNING calendar_id")
            .fetch_one(&pool).await.unwrap(); // Seed a calendar
        
        for evt_id in &[evt1, evt2] {
            sqlx::query("INSERT INTO calendar_events (id, uid, account_id, calendar_id, event_data, state_string) VALUES ($1, $1, 'test-acc', (SELECT calendar_id FROM calendars LIMIT 1), '{}', 'state')")
                .bind(evt_id)
                .execute(&pool).await.unwrap();
        }

        // ---------------------------------------------------------
        // SCENARIO A: Standard Non-Overlapping (Control) - SUCCESS
        // ---------------------------------------------------------
        assert!(insert_participant(&pool, evt1, emp1, "2026-06-01T09:00:00Z", "2026-06-01T13:00:00Z").await.is_ok());
        assert!(insert_participant(&pool, evt2, emp1, "2026-06-01T14:00:00Z", "2026-06-01T18:00:00Z").await.is_ok());

        // ---------------------------------------------------------
        // SCENARIO B: Exact Overlap (Double Booking) - FATAL ERROR
        // ---------------------------------------------------------
        let overlap_res = insert_participant(&pool, evt2, emp1, "2026-06-01T12:00:00Z", "2026-06-01T14:00:00Z").await;
        assert!(overlap_res.is_err(), "Mathematically failed: Allowed an overlapping subset!");

        // ---------------------------------------------------------
        // SCENARIO C: Adjacent Shifts (The Boundary Test) - SUCCESS
        // ---------------------------------------------------------
        // Shift B starts exactly when Shift A ends (13:00)
        assert!(insert_participant(&pool, evt2, emp1, "2026-06-01T13:00:00Z", "2026-06-01T14:00:00Z").await.is_ok());

        // ---------------------------------------------------------
        // SCENARIO D: Multi-Participant Job (Safe) - SUCCESS
        // ---------------------------------------------------------
        let group_evt = Uuid::new_v4();
        sqlx::query("INSERT INTO calendar_events (id, uid, account_id, calendar_id, event_data, state_string) VALUES ($1, $1, 'test', (SELECT calendar_id FROM calendars LIMIT 1), '{}', 's')").bind(group_evt).execute(&pool).await.unwrap();
        
        assert!(insert_participant(&pool, group_evt, emp1, "2026-06-02T08:00:00Z", "2026-06-02T16:00:00Z").await.is_ok());
        assert!(insert_participant(&pool, group_evt, emp2, "2026-06-02T08:00:00Z", "2026-06-02T16:00:00Z").await.is_ok());
        assert!(insert_participant(&pool, group_evt, emp3, "2026-06-02T08:00:00Z", "2026-06-02T16:00:00Z").await.is_ok());

        // ---------------------------------------------------------
        // SCENARIO E: Multi-Participant Collision - FATAL ERROR
        // ---------------------------------------------------------
        let collision_evt = Uuid::new_v4();
        sqlx::query("INSERT INTO calendar_events (id, uid, account_id, calendar_id, event_data, state_string) VALUES ($1, $1, 'test', (SELECT calendar_id FROM calendars LIMIT 1), '{}', 's')").bind(collision_evt).execute(&pool).await.unwrap();

        // Assign emp2 to a personal shift
        assert!(insert_participant(&pool, collision_evt, emp2, "2026-06-03T10:00:00Z", "2026-06-03T14:00:00Z").await.is_ok());

        // Attempt to assign emp2 and emp3 to a group event overlapping emp2's personal shift
        let mut tx = pool.begin().await.unwrap();
        let res1 = insert_participant(&pool, collision_evt, emp3, "2026-06-03T08:00:00Z", "2026-06-03T12:00:00Z").await;
        let res2 = insert_participant(&pool, collision_evt, emp2, "2026-06-03T08:00:00Z", "2026-06-03T12:00:00Z").await; // Should explode

        assert!(res1.is_ok());
        assert!(res2.is_err(), "Mathematically failed: Allowed emp2 to be double booked via a group event!");
    }
}
