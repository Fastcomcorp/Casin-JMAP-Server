# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.Sync.MicrosoftGraphWorker do
  @moduledoc """
  Translates JSCalendar payloads and pushes them to the employee's personal Microsoft Graph / Outlook Calendar.
  """
  use Oban.Worker, queue: :microsoft_sync, max_attempts: 3
  require Logger

  @graph_url "https://graph.microsoft.com/v1.0"

  def enqueue(shift_event) do
    case get_ms_link(shift_event.employee_id) do
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
         {:ok, access_token} <- refresh_ms_token_if_needed(link),
         {:ok, graph_event} <- translate_to_graph_event(event) do
      
      Logger.info("Syncing shift #{event["shift_id"]} to MS Graph for link #{link_id}")
      upsert_graph_event(access_token, link.external_calendar_id, graph_event)
    end
  end

  defp translate_to_graph_event(jscalendar_event) do
    # CRITICAL: Microsoft uses Windows timezone IDs, NOT IANA
    iana_tz = jscalendar_event["timeZone"] || "UTC"
    windows_tz = iana_to_windows_timezone(iana_tz)

    {:ok, %{
      "subject" => jscalendar_event["title"] || "Work Shift",
      "start" => %{
        "dateTime" => jscalendar_event["start"],
        "timeZone" => windows_tz
      },
      "end" => %{
        "dateTime" => calculate_end_ms(jscalendar_event),
        "timeZone" => windows_tz
      },
      "showAs" => "busy",
      "sensitivity" => "private",
      "singleValueExtendedProperties" => [
        %{
          "id" => "String {jmap-shift-id} Name jmapShiftId",
          "value" => jscalendar_event["uid"]
        }
      ]
    }}
  end

  @doc """
  Maps standard IANA timezones (RFC 8984) to legacy Windows Timezone formats required by MS Graph.
  """
  def iana_to_windows_timezone(iana) do
    mapping = %{
      "America/New_York"    => "Eastern Standard Time",
      "America/Chicago"     => "Central Standard Time",
      "America/Denver"      => "Mountain Standard Time",
      "America/Los_Angeles" => "Pacific Standard Time",
      "America/Phoenix"     => "US Mountain Standard Time",
      "America/Anchorage"   => "Alaskan Standard Time",
      "Pacific/Honolulu"    => "Hawaiian Standard Time",
      "Europe/London"       => "GMT Standard Time",
      "Europe/Paris"        => "Romance Standard Time",
      "Asia/Tokyo"          => "Tokyo Standard Time",
      "Australia/Sydney"    => "AUS Eastern Standard Time"
    }
    Map.get(mapping, iana, "UTC")
  end

  # Dummy helpers
  defp get_ms_link(_), do: {:error, :not_found}
  defp get_link(id), do: {:ok, %{link_id: id, external_calendar_id: "primary"}}
  defp refresh_ms_token_if_needed(_), do: {:ok, "mock_token"}
  defp calculate_end_ms(event), do: event["start"]
  defp upsert_graph_event(_, _, _), do: :ok
end
