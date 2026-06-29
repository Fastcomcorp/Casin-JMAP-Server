# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreWeb.Scheduling.EmployeeDashboardLive do
  @moduledoc """
  LiveView for the individual employee scheduling dashboard.
  Allows enabling Web Push, linking external calendars, and acknowledging shifts.
  """
  use ArmorcoreWeb, :live_view

  def mount(_params, session, socket) do
    if connected?(socket) do
      # Subscribe to the employee's personal schedule channel
      ArmorcoreWeb.Endpoint.subscribe("schedule:employee:#{session["employee_id"]}")
    end

    socket = assign(socket, 
      employee_id: session["employee_id"],
      shifts: fetch_my_shifts(session["employee_id"]),
      unacked_shifts: fetch_unacked_shifts(session["employee_id"]),
      google_linked?: check_link(session["employee_id"], "google"),
      ms_linked?: check_link(session["employee_id"], "microsoft")
    )

    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <div class="employee-dashboard">
      <header>
        <h1>My Work Schedule</h1>
        
        <div class="integrations">
          <h2>Calendar Sync</h2>
          <button phx-click="link_google" class={if @google_linked?, do: "linked"}>
            <%= if @google_linked?, do: "Google Calendar Linked", else: "Link Google Calendar" %>
          </button>
          <button phx-click="link_microsoft" class={if @ms_linked?, do: "linked"}>
            <%= if @ms_linked?, do: "Microsoft Graph Linked", else: "Link Microsoft Calendar" %>
          </button>
          
          <button phx-hook="PushRegistration" id="push-btn">Enable Browser Notifications</button>
        </div>
      </header>

      <section class="unacked-alerts">
        <%= for shift <- @unacked_shifts do %>
          <div class="alert alert-warning">
            <p><strong>Shift Updated!</strong> New Start: <%= shift.start %></p>
            <button phx-click="acknowledge_shift" phx-value-id={shift.id}>Acknowledge</button>
          </div>
        <% end %>
      </section>

      <section class="schedule-grid">
        <!-- Render normal shifts here -->
      </section>
    </div>
    """
  end

  def handle_event("acknowledge_shift", %{"id" => shift_id}, socket) do
    # Here we would normally make a JMAP API call to Rust:
    # POST /jmap { "CalendarEvent/set": { "update": { shift_id: { "x-shift-ack": { ... } } } } }
    
    # And notify ETS
    ArmorcoreScheduling.Ack.AckDeadlineServer.mark_acknowledged(shift_id)

    {:noreply, update(socket, :unacked_shifts, fn shifts -> Enum.reject(shifts, &(&1.id == shift_id)) end)}
  end

  def handle_info(%Phoenix.Socket.Broadcast{event: "shift:updated", payload: event}, socket) do
    # Handle real-time push from NATS
    {:noreply, assign(socket, unacked_shifts: [event | socket.assigns.unacked_shifts])}
  end

  # Dummy helpers
  defp fetch_my_shifts(_), do: []
  defp fetch_unacked_shifts(_), do: []
  defp check_link(_, _), do: false
end
