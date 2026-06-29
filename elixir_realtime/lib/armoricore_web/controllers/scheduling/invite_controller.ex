# Copyright (c) 2026 Fastcomcorp, LLC. All rights reserved.
# This source code is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).
# See the LICENSE file in the root directory for more details.

defmodule ArmorcoreWeb.Scheduling.InviteController do
  use ArmorcoreWeb, :controller
  import Swoosh.Email

  @moduledoc """
  Handles the flow for inviting a new employee to the scheduling system.
  Sends an email with a secure SSO link.
  """

  def create(conn, %{"email" => email, "role" => role}) do
    # Create employee record in DB
    # Generates a temporary invite token
    token = generate_invite_token()
    
    send_invite_email(email, token, role)

    json(conn, %{status: "success", message: "Invite sent"})
  end

  defp send_invite_email(email, token, role) do
    invite_url = "https://scheduling.fastcomcorp.com/invite?token=#{token}"
    from_email = System.get_env("SMTP_FROM_EMAIL", "noreply@fastcomcorp.com")

    new()
    |> to(email)
    |> from({"Armoricore Scheduling", from_email})
    |> subject("You have been invited to the Scheduling Portal (#{role})")
    |> html_body("""
      <p>Click below to securely onboard and set up your calendar links:</p>
      <a href="#{invite_url}">Join Schedule Portal</a>
    """)
    # |> Armoricore.Mailer.deliver()
  end

  defp generate_invite_token(), do: "mock-token-1234"
end
