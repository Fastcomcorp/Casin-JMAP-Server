# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.NatsConsumer do
  @moduledoc """
  Subscribes to NATS JetStream to receive shift change events from the Rust JMAP server.
  Dispatches to Push, Oban Sync Workers, and WebSocket channels.
  """
  use GenServer
  require Logger

  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def init(_opts) do
    # Assuming Gnat is configured in the application
    {:ok, conn} = Gnat.start_link(%{host: "127.0.0.1", port: 4222})
    {:ok, _sub} = Gnat.sub(conn, self(), "scheduling.shift.changed")
    Logger.info("NATS Consumer started for scheduling events")
    {:ok, %{conn: conn}}
  end

  def handle_info({:msg, %{body: body}}, state) do
    # SECURITY PATCH: Use strings instead of atoms to prevent Erlang Atom Exhaustion (DoS)
    with {:ok, event} <- Jason.decode(body, keys: :strings) do
      Logger.info("Received shift change event for employee #{event["employee_id"]}")
      handle_shift_changed(event)
    else
      err -> Logger.error("Failed to decode NATS scheduling event: #{inspect(err)}")
    end
    {:noreply, state}
  end

  defp handle_shift_changed(event) do
    # Access keys as strings due to security patch
    employee_id = event["employee_id"]

    # 1. Notify via WebSocket if employee is connected
    ArmorcoreScheduling.Channels.ScheduleChannel.broadcast_to_employee(
      employee_id,
      "shift:updated",
      event
    )

    # 2. Send Web Push notification
    ArmorcoreScheduling.Push.EmployeePushServer.notify(employee_id, event)

    # 3. Start ack deadline timer
    ArmorcoreScheduling.Ack.AckDeadlineServer.start_deadline(
      event.shift_id,
      employee_id,
      event.ack_deadline_minutes
    )
  end
end
