# Casin JMAP Server: Developer Handbook

Welcome to the **Casin Scheduling Engine**. This architecture is designed to manage workforce logistics at an enterprise scale (up to 500,000 employees). This handbook explains the core architectural pillars and provides a troubleshooting matrix.

---

## 1. Architectural Overview: The Split-Brain

Casin relies on a "Split-Brain" architecture to achieve both mathematical safety and extreme real-time concurrency.

> [!NOTE]
> **Why Two Languages?**
> Rust is perfect for parsing massive JSON payloads and doing rigorous database math. Elixir is perfect for routing millions of WebSockets simultaneously. By splitting them, we get the best of both worlds without compromising speed or safety.

```mermaid
graph TD
    A[JMAP Client App] -->|HTTP POST JSON| B[Rust Axum Server]
    B -->|Ironclad Math| C[(PostgreSQL)]
    B -->|JSON Publishes| D[NATS JetStream]
    D -->|Streams| E[Elixir Phoenix Cluster]
    E -->|WebSockets| A
    E -->|Oban Queue| F[Google / MS / Apple Calendars]
```

---

## 2. The Ironclad Guarantee (PostgreSQL)

The core feature of this entire project is the mathematical prohibition of double-booking an employee.

### Declarative Partitioning
The `event_participants` table is partitioned by `RANGE (shift_start)` into monthly tables. 
> [!WARNING]
> If you add a new index or unique constraint to `event_participants`, you **MUST** include the `shift_start` column in the key, otherwise PostgreSQL will throw an error.

### JMAP Partial Success (`notCreated`)
If a dispatcher assigns 3 employees to a shift, but Employee C is already double-booked, the GiST exclusion constraint violently blocks the insertion.
The Rust handler (`src/jmap/calendar_event.rs`) specifically catches the `exclusion constraint` error. It routes Employees A and B to the `created` block, and routes Employee C to the `notCreated` block, allowing the client App to gracefully handle the partial failure.

---

## 3. The Oban Sync Matrix

To ensure the workforce's master schedule syncs to their personal devices (Google, Outlook, iPhone), we use Elixir's Oban background job processor.

### Third-Party Rate Limits
Big Tech APIs (Google Workspace, Microsoft Graph) will block our IP address if we fire too many concurrent HTTP requests.
> [!IMPORTANT]
> The `config.exs` explicitly throttles the queues: `google_sync: 5`, `microsoft_sync: 5`, `apple_sync: 5`. At this concurrency, we max out around 25 requests/sec. Do **not** increase this limit unless you have secured a quota increase from Google Cloud or Microsoft Azure Support.

---

## 4. ArcRTC: High-Velocity & Mesh Syncing

The Elixir layer contains `arc_rtc_channel.ex`. This is a WebRTC Signaling Server.
When JMAP clients require zero-latency updates (e.g., dispatcher UI) or are operating offline in a dead-zone, they establish direct peer-to-peer UDP Data Channels using ArcRTC.
They serialize standard JMAP JSON payloads into a binary envelope and fire them over UDP.

---

## 5. Local Setup & Testing

Before contributing to the Casin JMAP Server, you must be able to boot the stack and pass the security test suite locally.

### Prerequisites
* **Rust:** `v1.70+`
* **Elixir:** `v1.15+` (Erlang/OTP 25+)
* **PostgreSQL:** `v15.0+` (Running locally on port 5432)
* **NATS Server:** `v2.9.0+` (Running locally on port 4222)

### Booting the Stack
Do not hardcode database passwords into the source code. Export your credentials in your shell environment before running the startup script:

```bash
# 1. Export local DB credentials
export JMAP_DATABASE_URL="postgres://your_user:your_password@127.0.0.1:5432/casin_jmap"
export DATABASE_URL="postgres://your_user:your_password@127.0.0.1:5432/casin_jmap"

# 2. Boot both Rust and Elixir services
./start.sh
```

### Running the Fuzz Tests (Required)
Because Casin handles dynamic JSON payloads from millions of external users, the JSON parsing boundary is the most vulnerable attack vector. If you modify any JMAP object struct, you **must** run the fuzz tester to ensure you haven't introduced a panic/memory exhaustion vulnerability.

```bash
cd rust-services/jmap-scheduler
cargo run --bin fuzz_jmap_parser
```
*Note: This script will blast the parser with millions of mutated, highly-nested malicious JSON objects. It must complete without panicking before you open a Pull Request.*

---

## 6. Troubleshooting Guide

If the system degrades in production, follow this matrix:

### Symptom: "The API is rejecting all requests from the App."
*   **Diagnosis:** The API Gateway security boundary is failing.
*   **Fix:** Check `src/auth/jwt.rs`. Ensure the JWT token sent by the app is valid, not expired, and contains the required `employee_id` payload. Check the CORS settings in `main.rs` to ensure `app.domain.com` is exactly correct.

### Symptom: "Employees are not seeing shifts in Google Calendar."
*   **Diagnosis:** The Oban worker queue is stalled or rate-limited.
*   **Fix:** Connect to the Elixir console (`iex -S mix`) and run `Oban.check()`. If jobs are marked `discarded` or `retryable`, look at the error logs. If Google returned HTTP 403, we hit a rate limit. Decrease the concurrency in `config.exs`.

### Symptom: "Saving shifts on Monday morning takes 30 seconds."
*   **Diagnosis:** The PostgreSQL GiST index is collapsing under weight.
*   **Fix:** Verify that the DBA ran the script to create the new month's partition for `event_participants`. If all shifts are hitting the default partition alongside 10 million other rows, the database will crawl.

### Symptom: "WebSockets are dropping under heavy load."
*   **Diagnosis:** The Elixir nodes are not clustering.
*   **Fix:** Verify that UDP multicast (Port 45892) is allowed in your AWS Security Groups. If `libcluster` cannot form the mesh, each Elixir node acts independently, preventing PubSub broadcasts from spanning across servers.
