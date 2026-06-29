# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreScheduling.Email.ShiftNotifier do
  @moduledoc """
  Uses Swoosh (SMTP client) to send fallback or initial notification emails
  about shift changes through the company's existing G-Suite or O365 SMTP relay.
  """
  import Swoosh.Email
  require Logger

  def send_shift_notification(event) do
    # In a real app, fetch employee details from DB
    employee_email = "employee@example.com"
    employee_name = "Jane Doe"
    from_email = System.get_env("SMTP_FROM_EMAIL", "schedule-noreply@fastcomcorp.com")

    Logger.info("Sending shift email to #{employee_email}")

    new()
    |> to({employee_name, employee_email})
    |> from({"Work Schedule", from_email})
    |> subject("Shift Update: #{event.new_start}")
    |> html_body(render_shift_email(event))
    |> text_body("Your shift has been updated. Login to acknowledge.")
    # |> Armoricore.Mailer.deliver() # Uncomment when Mailer is defined in core
  end

  defp render_shift_email(event) do
    """
    <h1>Shift Update</h1>
    <p>Your shift has been <strong>#{event.change_type}</strong>.</p>
    <p>Start: #{event.new_start}</p>
    <p><a href="https://scheduling.fastcomcorp.com/schedule/#{event.shift_id}">Click here to acknowledge this change.</a></p>
    """
  end
end
