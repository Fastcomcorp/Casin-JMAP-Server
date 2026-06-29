# ArcRTC JMAP Data Channel Protocol

The Casin Scheduling system uses standard HTTP/WebSockets for normal operations. However, for ultra-low latency and offline mesh networking, the system utilizes **ArcRTC (WebRTC Data Channels)**.

This document outlines how JMAP payloads are serialized and transmitted over UDP data channels.

## 1. The Signaling Phase
Before devices can transmit calendar data, they must establish a WebRTC connection. This is facilitated by the Elixir Phoenix cluster (`ArcRTCChannel`).
1. Device A connects to the Elixir WebSocket.
2. Device A sends an SDP Offer to the `arc_rtc:signaling` channel.
3. Elixir relays the offer to Device B.
4. Device B responds with an SDP Answer.
5. Elixir relays the ICE Candidates until a direct UDP P2P connection is formed.

## 2. JMAP Data Serialization
Once the WebRTC Data Channel is open, standard HTTP headers are abandoned. Devices transmit raw byte streams.

### The Payload Envelope
To ensure the receiving device understands the payload, all JMAP requests are wrapped in a lightweight binary-friendly JSON envelope:

```json
{
  "arc_protocol": "jmap_v1",
  "sync_state": "uuid-of-the-current-calendar-state",
  "jmap_payload": {
     "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:calendars"],
     "methodCalls": [
       ["CalendarEvent/set", { "create": { ... } }, "client-1"]
     ]
  }
}
```

### Transmission
1. The JMAP client serializes the envelope to a UTF-8 string.
2. The string is transmitted over the WebRTC Data Channel using `RTCDataChannel.send()`.
3. Because Data Channels use UDP, the payload arrives in milliseconds without TCP handshake overhead.

## 3. Conflict Resolution & Mesh Syncing
In an offline environment (e.g., a basement with no cell service), devices use local network ArcRTC connections to share schedule updates.

### State Reconciliation
When Device A sends a payload to Device B:
1. Device B checks the `sync_state` UUID.
2. If Device A's state is newer (based on vector clocks or timestamp embedded in the payload), Device B accepts the JMAP `CalendarEvent/set` mutation and updates its local SQLite/CoreData cache.
3. If a conflict occurs (two managers edited the same shift offline), the JMAP `notCreated` partial success logic is utilized locally.

### Server Synchronization
When the devices regain internet access:
1. The "Master" device connects to `app.domain.com`.
2. It sends the aggregated JMAP batch update via standard HTTP `POST /jmap`.
3. The Rust backend validates the state and permanently commits the shifts to the PostgreSQL `event_participants` table, calculating the final Ironclad Guarantee.
