# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreWeb.Scheduling.ManagementDashboardLive do
  @moduledoc """
  LiveView for the management dashboard, showing all employees and highlighting unacknowledged shifts.
  """
  use ArmorcoreWeb, :live_view

  def mount(_params, _session, socket) do
    if connected?(socket) do
      ArmorcoreWeb.Endpoint.subscribe("schedule:management")
    end

    {:ok, assign(socket, unacked_alerts: [])}
  end

  def render(assigns) do
    ~H"""
    <div class="management-dashboard">
      <h1>Workforce Scheduling Control</h1>
      
      <section class="critical-alerts" style="color: red;">
        <%= for alert <- @unacked_alerts do %>
          <div class="alert">
            URGENT: Employee <%= alert.employee_id %> has not acknowledged shift <%= alert.shift_id %>!
          </div>
        <% end %>
      </section>

      <!-- Calendar grid of all employees -->
    </div>
    """
  end

  def handle_info(%Phoenix.Socket.Broadcast{event: "shift:unacknowledged", payload: payload}, socket) do
    # Received from AckDeadlineServer when deadline expires
    {:noreply, assign(socket, unacked_alerts: [payload | socket.assigns.unacked_alerts])}
  end
end
