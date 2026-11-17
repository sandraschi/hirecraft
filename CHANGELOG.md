# Changelog

All notable changes to `hirecraft` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-11-16

### Changed
- Docs only. HireCraft keeps the candidate pane. Employee cards will live in TeamCraft. Spec: `mcp-central-docs/projects/setupcraft/PRD.md`.
- The hire event is specified to grow a chosen start date, position, and work text from the requisition. Offered gross will be stored on the TeamCraft card as monthly gross, with no tax calculation. The running payload is still `candidate_id`, `name`, `email`, `req_id`, `salary_eur`, `azg_weekly_cap` 38.5, `annual_leave_days` 25.
- Downstream drains that already exist: TeamCraft opens a thin employee and drops the wage; ContractCraft opens a draft employment contract; AssetCraft records an equipment reminder and does not invent an asset.

## [Unreleased] - 2026-11-13

### Added
- The window menu is File, Edit, View, Window, and Help. `--smoke` checks the menu.

## [0.1.0] - 2026-11-10

### Added
- Pure-Rust 4-crate workspace architecture (`hirecraft-store`, `hirecraft-core`, `hirecraft-mcp`, `hirecraft-app`).
- Job requisition and 5-stage candidate pipeline (`applied` → `screening` → `interview` → `offer` → `hired`).
- Blind quantitative scorecard evaluation engine (1–5 ratings across technical skills and cultural alignment).
- Automated onboarding cascade: moving a candidate to `hired` simultaneously emits `candidate.hired` to:
  - `contractcraft\inbox` (employment agreement drafting)
  - `teamcraft\inbox` (Austrian AZG 38.5h weekly limit and leave entitlement profile)
  - `assetcraft\inbox` (primary workstation reservation)
- Stdio FastMCP JSON-RPC server and CLI with 8 tools (`hire_status`, `hire_req_create`, `hire_req_list`, `hire_candidate_create`, `hire_candidate_show`, `hire_candidate_stage`, `hire_candidate_hire`, `hire_scorecard_add`).
- Native egui 0.37 desktop workstation (`hirecraft-app`) with `--smoke` verification.
- Central fleet launcher `mcp-central-docs\starts\hirecraft-start.bat`.
