# Changelog

All notable changes to `hirecraft` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
