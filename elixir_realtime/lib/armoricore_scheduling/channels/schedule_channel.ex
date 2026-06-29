# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.Channels.ScheduleChannel do
  @moduledoc """
  Phoenix Channel handling WebSockets for management dashboards and individual employees.
  """
  use Phoenix.Channel

  def join("schedule:management", _params, socket) do
    case socket.assigns.role do
      "management" -> {:ok, socket}
      _ -> {:error, %{reason: "unauthorized"}}
    end
  end

  def join("schedule:employee:" <> employee_id, _params, socket) do
    if socket.assigns.employee_id == employee_id do
      {:ok, socket}
    else
      {:error, %{reason: "unauthorized"}}
    end
  end

  def broadcast_management(event, payload) do
    # ArmorcoreWeb.Endpoint.broadcast("schedule:management", event, payload)
    :ok
  end

  def broadcast_to_employee(employee_id, event, payload) do
    # ArmorcoreWeb.Endpoint.broadcast("schedule:employee:#{employee_id}", event, payload)
    :ok
  end
end
