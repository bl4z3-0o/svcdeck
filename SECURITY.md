# Security

## Reporting a vulnerability

Please do not open a public issue. Use GitHub's private reporting instead: **Security → Report a vulnerability** in this repository. You get an answer within a week.

## Scope

SvcDeck runs the shell commands from your own configuration (`~/.config/svcdeck/apps.json`) with your user rights; that is intended. Relevant reports are, for example:

- a way to make SvcDeck run commands that are not in the saved configuration (e.g. from the web view)
- status checks or the update check leaking data or prompting for passwords
- the sudo password dialog exposing the password
- unsafe handling of values in the templates (shell injection)

Only the latest release receives fixes.
