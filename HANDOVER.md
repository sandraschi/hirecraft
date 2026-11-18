---
updated: 2026-11-16T15:48:00+01:00
agent: cursor
status: docs-updated
tags: [hirecraft, ats, recruiting, onboarding, scorecards, craft-biz, setupcraft]
---

# Handover - HireCraft

## What Was Done
- Shipped full 4-crate pure Rust workspace (`hirecraft-store`, `hirecraft-core`, `hirecraft-mcp`, `hirecraft-app`).
- Implemented 5-stage recruiting pipeline and blind interview scorecards.
- Implemented automated onboarding cascade emitting `candidate.hired` to `contractcraft`, `teamcraft`, and `assetcraft`.
- Native desktop egui 0.37 UI with pipeline view, stage filters, and `--smoke` check.
- Clean `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` compilation with 0 warnings.
- Pushed to GitHub at `https://github.com/sandraschi/hirecraft`.

## Last Concrete Action
- Verified `mcp-central-docs\starts\hirecraft-start.bat --smoke` exits with code 0 (`smoke: ok (hirecraft-app)`).

## Docs on 2026-11-16

- No code change. PRD and changelog now match the drains: TeamCraft opens a thin employee and drops the wage, ContractCraft opens a draft, AssetCraft records an equipment reminder.
- Employee cards are specified for TeamCraft, not for this app. Founding staff come from SetupCraft. Later hires stay here, with a richer `candidate.hired` still to build. Spec: `mcp-central-docs/projects/setupcraft/PRD.md`.

## Next Step

- Add start date, position, and work text to `candidate.hired`, and a test that TeamCraft stores the offered gross on an incomplete Mitarbeiterkarte.
- Pilot the candidate pipeline only for people hired after go-live. The founding batch does not pass through HireCraft.
