# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.Push.EmployeePushServer do
  @moduledoc """
  GenServer responsible for pushing Web Push notifications to a specific employee.
  Dynamically supervised.
  """
  use GenServer

  def start_link(employee_id) do
    GenServer.start_link(__MODULE__, employee_id, name: via_tuple(employee_id))
  end

  def notify(employee_id, event) do
    # Ensure process is started, then cast to it
    # In a real app, you'd dynamically start it if it doesn't exist
    case Registry.lookup(ArmorcoreScheduling.Push.Registry, employee_id) do
      [{pid, _}] -> GenServer.cast(pid, {:notify, event})
      [] -> 
        # For simplicity, fallback to async task if process not running
        Task.start(fn -> deliver_web_push(nil, event) end)
    end
  end

  defp via_tuple(employee_id) do
    {:via, Registry, {ArmorcoreScheduling.Push.Registry, employee_id}}
  end

  def init(employee_id) do
    {:ok, %{employee_id: employee_id, tokens: []}}
  end

  def handle_cast({:notify, event}, state) do
    # Iterate over tokens and push via VAPID
    Enum.each(state.tokens, fn token ->
      deliver_web_push(token, event)
    end)
    {:noreply, state}
  end

  defp deliver_web_push(_token, event) do
    # VAPID push logic goes here
    # ArmorcoreScheduling.Push.WebPushDelivery.send(...)
    :ok
  end
end
