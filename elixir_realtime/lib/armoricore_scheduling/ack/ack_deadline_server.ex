# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.Ack.AckDeadlineServer do
  @moduledoc """
  Tracks unacknowledged shifts using ETS. Fires escalations when deadlines expire.
  """
  use GenServer
  require Logger

  def start_link(_) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def init(_) do
    table = :ets.new(:ack_deadlines, [:set, :protected, :named_table])
    {:ok, %{table: table}}
  end

  def start_deadline(shift_id, employee_id, deadline_minutes) do
    GenServer.cast(__MODULE__, {:start_deadline, shift_id, employee_id, deadline_minutes})
  end

  def mark_acknowledged(shift_id) do
    GenServer.cast(__MODULE__, {:acknowledged, shift_id})
  end

  def handle_cast({:start_deadline, shift_id, employee_id, minutes}, state) do
    deadline_ms = :timer.minutes(minutes)
    timer_ref = Process.send_after(self(), {:deadline_expired, shift_id, employee_id}, deadline_ms)
    :ets.insert(:ack_deadlines, {shift_id, employee_id, timer_ref, :pending})
    {:noreply, state}
  end

  def handle_cast({:acknowledged, shift_id}, state) do
    case :ets.lookup(:ack_deadlines, shift_id) do
      [{^shift_id, _employee_id, timer_ref, :pending}] ->
        Process.cancel_timer(timer_ref)
        :ets.delete(:ack_deadlines, shift_id)
        Logger.info("Shift #{shift_id} acknowledged")
      _ -> :ok
    end
    {:noreply, state}
  end

  def handle_info({:deadline_expired, shift_id, employee_id}, state) do
    :ets.delete(:ack_deadlines, shift_id)
    Logger.warning("Ack deadline expired for shift #{shift_id}, employee #{employee_id}")
    
    # Escalate to management
    ArmorcoreScheduling.Channels.ScheduleChannel.broadcast_management(
      "shift:unacknowledged",
      %{shift_id: shift_id, employee_id: employee_id}
    )
    
    {:noreply, state}
  end
end
