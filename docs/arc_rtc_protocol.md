# ArcRTC Protocol Specification (JMAP Data Channel)

The Casin Scheduling system uses standard HTTP/WebSockets for normal operations. However, for ultra-low latency and offline mesh networking, the system utilizes **ArcRTC (Armoricore Real-Time Communication)**.

This document outlines the core ArcRTC specification (inherited from the Fastcomcorp Armoricore foundations) and details how JMAP payloads are serialized and transmitted over its P2P data channels.

---

## 1. Overview & Design Philosophy

**ArcRTC** is a high-performance, low-latency transport protocol optimized for environments where standard WebRTC overhead is too heavy, or where cellular networks are entirely unavailable.

- **Ultra-low latency** (< 50ms end-to-end)
- **High performance** (minimal overhead, no browser stack required)
- **Full control** (packet-level optimization and routing)
- **Native platforms** (mobile, desktop, embedded devices)
- **Hybrid compatibility** (works alongside standard WebRTC when needed)

### Key Differentiators from standard WebRTC
| Feature | WebRTC | ArcRTC |
|---------|--------|--------|
| **Latency** | 100-200ms typical | < 50ms target |
| **Overhead** | Browser stack | Minimal native |
| **Control** | Browser-managed | Full packet control |
| **Codecs** | Browser-supported only | Any payload (specifically JMAP JSON) |
| **Complexity** | High (ICE, SDP, etc.) | Simplified JSON signaling |

---

## 2. Protocol Architecture

The ArcRTC stack replaces traditional HTTP transport with a lightweight UDP protocol layer.

```text
┌─────────────────────────────────────────────────────────┐
│              Application Layer                          │
│  • JMAP Parsing / Collision resolution                  │
│  • Scheduling application logic                         │
└─────────────────────────────────────────────────────────┘
                        │
┌─────────────────────────────────────────────────────────┐
│              ArcRTC Protocol Layer                      │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Signaling Protocol (ArcSignaling)               │   │
│  │  • Connection establishment                      │   │
│  │  • P2P Discovery (Bluetooth/Wi-Fi Direct)        │   │
│  └──────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Data Transport (ArcData)                        │   │
│  │  • Encrypted JMAP Payload delivery               │   │
│  │  • Packet routing & reconciliation               │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                        │
┌─────────────────────────────────────────────────────────┐
│              Transport Layer                            │
│  • UDP (primary)                                        │
│  • TCP (fallback)                                       │
└─────────────────────────────────────────────────────────┘
```

---

## 3. The Signaling Phase (ArcSignaling)

Before devices can transmit calendar data, they must establish an ArcRTC connection. When online, this is facilitated by the Elixir Phoenix cluster (`ArcRTCChannel`). When offline in a dead zone, devices broadcast these signals locally over Bluetooth or Wi-Fi Direct.

### 1. Connection Request (`CONNECT`)
Initiates the P2P connection between devices to form the offline mesh.

```json
{
  "type": "CONNECT",
  "version": "1.0",
  "session_id": "uuid",
  "peer_id": "uuid",
  "capabilities": {
    "transport": ["udp", "tcp"],
    "payload": ["jmap_v1"]
  },
  "network_info": {
    "public_ip": "1.2.3.4",
    "public_port": 50000,
    "nat_type": "cone"
  },
  "timestamp": 1234567890
}
```

### 2. Connection Response (`CONNECT_ACK`)
Accepts the connection, finalizes the transport method, and provisions the encryption keys.

```json
{
  "type": "CONNECT_ACK",
  "session_id": "uuid",
  "peer_id": "uuid",
  "accepted": true,
  "encryption": {
    "algorithm": "aes-128-gcm",
    "key_exchange": "ecdh-p256"
  },
  "timestamp": 1234567891
}
```

---

## 4. JMAP Data Serialization

Once the ArcRTC Data Channel is open and authenticated, standard HTTP headers are abandoned. Devices transmit raw, AES-128-GCM encrypted byte streams directly peer-to-peer.

### The Payload Envelope
To ensure the receiving device understands the payload context without HTTP wrappers, all JMAP requests are wrapped in a lightweight, binary-friendly JSON envelope:

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

### Transmission & Server Reconciliation
Because this operates as a true mesh network, shift changes (like clocking in) propagate from device to device in the dead zone. The moment *any* single device in the mesh regains cellular service, it acts as the master relay, pushing the entire mesh's state back to the Casin Server for final PostgreSQL database reconciliation.
