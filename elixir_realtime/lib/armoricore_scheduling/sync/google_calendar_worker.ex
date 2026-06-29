# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.Sync.GoogleCalendarWorker do
  @moduledoc """
  Oban worker to sync JMAP calendar events to an employee's linked Google Calendar.
  Handles token refreshes and OAuth API calls.
  """
  use Oban.Worker, queue: :google_sync, max_attempts: 5
  require Logger

  @google_events_url "https://www.googleapis.com/calendar/v3/calendars"

  def enqueue(shift_event) do
    # Only enqueue if employee has a linked Google Calendar
    # Mocking DB fetch for linked calendar
    case get_google_link(shift_event.employee_id) do
      {:ok, link} ->
        %{shift_event: shift_event, link_id: link.link_id}
        |> new()
        |> Oban.insert()
      _ -> :ok
    end
  end

  @impl Oban.Worker
  def perform(%Oban.Job{args: %{"shift_event" => event, "link_id" => link_id}}) do
    with {:ok, link} <- get_link(link_id),
         {:ok, access_token} <- refresh_token_if_needed(link),
         {:ok, google_event} <- translate_to_google_event(event) do
      
      Logger.info("Syncing shift #{event["shift_id"]} to Google Calendar for link #{link_id}")
      upsert_google_event(access_token, link.external_calendar_id, google_event)
    end
  end

  defp translate_to_google_event(jscalendar_event) do
    {:ok, %{
      "summary" => jscalendar_event["title"] || "Work Shift",
      "start" => %{
        "dateTime" => jscalendar_event["start"],
        "timeZone" => jscalendar_event["timeZone"] || "UTC"
      },
      "end" => %{
        "dateTime" => calculate_end(jscalendar_event),
        "timeZone" => jscalendar_event["timeZone"] || "UTC"
      },
      "extendedProperties" => %{
        "private" => %{
          "jmapShiftId" => jscalendar_event["uid"],
          "jmapAck" => "false"
        }
      }
    }}
  end

  # Dummy helpers to satisfy compilation
  defp get_google_link(_), do: {:error, :not_found}
  defp get_link(id), do: {:ok, %{link_id: id, external_calendar_id: "primary"}}
  defp refresh_token_if_needed(_), do: {:ok, "mock_token"}
  defp calculate_end(event), do: event["start"] # Simplification
  defp upsert_google_event(_, _, _), do: :ok
end
