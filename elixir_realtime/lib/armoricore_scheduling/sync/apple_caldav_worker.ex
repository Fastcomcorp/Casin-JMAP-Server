# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmoricoreScheduling.Sync.AppleCalDavWorker do
  @moduledoc """
  Translates JSCalendar payloads into standard iCalendar (.ics) format and 
  pushes them to the employee's personal Apple iCloud / CalDAV server.
  """
  use Oban.Worker, queue: :apple_sync, max_attempts: 3
  require Logger

  @impl Oban.Worker
  def perform(%Oban.Job{args: %{"event_data" => event, "caldav_url" => url, "token" => token}}) do
    Logger.info("Syncing event #{event["uid"]} to Apple CalDAV at #{url}")

    # 1. Translate JMAP JSON to RFC 5545 iCalendar format (.ics)
    ics_payload = build_icalendar(event)

    # 2. Fire the HTTP PUT request to the CalDAV server
    # CalDAV requires sending the raw .ics text body
    # Using a dummy HTTP client call for the architecture stub
    # HTTPoison.put(url, ics_payload, [
    #   {"Authorization", "Bearer #{token}"},
    #   {"Content-Type", "text/calendar; charset=utf-8"}
    # ])

    {:ok, :synced}
  end

  defp build_icalendar(event) do
    # Converts JMAP date-times and fields into standard iCal string
    """
    BEGIN:VCALENDAR
    VERSION:2.0
    PRODID:-//Fastcomcorp//Casin JMAP Server//EN
    BEGIN:VEVENT
    UID:#{event["uid"]}
    DTSTART:#{format_datetime(event["start"])}
    DURATION:#{event["duration"]}
    SUMMARY:#{event["title"]}
    END:VEVENT
    END:VCALENDAR
    """
  end

  defp format_datetime(jmap_datetime) do
    # Converts JMAP 2026-06-06T09:00:00Z to iCal 20260606T090000Z
    String.replace(jmap_datetime, ~r/[-:]/, "")
  end
end
