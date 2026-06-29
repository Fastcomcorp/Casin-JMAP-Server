# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

import Config

# Oban Configuration for Armoricore Scheduling
# This configuration specifically limits concurrency to protect against
# Google Calendar and Microsoft Graph API rate limits when scaling to 500+ employees.

config :armoricore_scheduling, Oban,
  repo: Armoricore.Repo,
  plugins: [
    Oban.Plugins.Pruner,
  ],
  queues: [
    default: 10,
    mailers: 20,
    google_sync: 5,
    microsoft_sync: 5,
    apple_sync: 5
  ]

# Enterprise Upgrade: libcluster Topology
# This configures the Elixir nodes to automatically discover each other
# via UDP multicast (Gossip) and form a single massive distributed cluster.
config :libcluster,
  topologies: [
    armoricore_cluster: [
      strategy: Cluster.Strategy.Gossip,
      config: [
        port: 45892,
        if_addr: "0.0.0.0",
        multicast_addr: "230.1.1.251",
        # Ensures that a spike of 300,000 websockets are balanced across all nodes
        broadcast_only: false
      ]
    ]
  ]
