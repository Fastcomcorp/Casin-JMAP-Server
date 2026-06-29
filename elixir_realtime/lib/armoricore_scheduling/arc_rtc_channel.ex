# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmoricoreScheduling.ArcRTCChannel do
  @moduledoc """
  Phoenix Channel acting as the WebRTC Signaling Server.
  Relays Session Description Protocol (SDP) Offers, Answers, and ICE Candidates
  between JMAP client applications (Web, iOS, Android).
  
  This allows clients to establish direct ArcRTC Data Channels for ultra-low latency
  JMAP Calendar payload synchronization, bypassing HTTP entirely.

  ## Protocol Specification
  Once signaled here, clients must transmit standard RFC 8620 JMAP JSON payloads
  wrapped in an `arc_protocol` envelope directly over the UDP Data Channel.
  If the sync occurs offline (Mesh Syncing), the master device is responsible 
  for eventually pushing the aggregated JMAP batch to the Rust backend via HTTP POST.
  """
  use Phoenix.Channel
  require Logger

  # Clients join a specific room or their own private signaling channel
  def join("arc_rtc:signaling:" <> _room_id, _payload, socket) do
    # In production, we would verify the user's JWT token here
    Logger.info("Client #{socket.id} joined ArcRTC signaling channel")
    {:ok, socket}
  end

  # Relay SDP Offers from Caller to Callee
  def handle_in("sdp_offer", %{"target_client_id" => target_id, "sdp" => sdp}, socket) do
    Logger.debug("Relaying SDP Offer to #{target_id}")
    # Broadcast to the specific target client
    broadcast!(socket, "sdp_offer_received", %{
      "from_client_id" => socket.id,
      "sdp" => sdp
    })
    {:noreply, socket}
  end

  # Relay SDP Answers back from Callee to Caller
  def handle_in("sdp_answer", %{"target_client_id" => target_id, "sdp" => sdp}, socket) do
    Logger.debug("Relaying SDP Answer to #{target_id}")
    broadcast!(socket, "sdp_answer_received", %{
      "from_client_id" => socket.id,
      "sdp" => sdp
    })
    {:noreply, socket}
  end

  # Relay ICE Candidates for NAT Traversal
  def handle_in("ice_candidate", %{"target_client_id" => target_id, "candidate" => candidate}, socket) do
    broadcast!(socket, "ice_candidate_received", %{
      "from_client_id" => socket.id,
      "candidate" => candidate
    })
    {:noreply, socket}
  end
end
