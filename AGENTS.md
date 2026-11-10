# HireCraft — AGENTS.md

Local-first sovereign Applicant Tracking System (ATS), blind scorecard interview pipeline, and employee onboarding engine.

## Crates

- `crates/hirecraft-store`: Pure SQLite database layer (`JobRequisitions`, `Candidates`, `Scorecards`).
- `crates/hirecraft-core`: Structured recruiting stage pipeline, blind scorecard aggregation, onboarding cascade dispatcher.
- `crates/hirecraft-mcp`: Stdio JSON-RPC FastMCP server & CLI binary (`hirecraft`).
- `crates/hirecraft-app`: Native desktop GUI executable (`hirecraft-app`) via `egui 0.37`.

## Bus Events

- Outbound: `candidate.hired` (emitted to `contractcraft\inbox`, `teamcraft\inbox`, `assetcraft\inbox`)

## Smoke Test

```powershell
.\start.ps1 -Smoke
```
