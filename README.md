# HireCraft 🎯

**Local-first sovereign Applicant Tracking System (ATS), blind scorecard interview pipeline, and automated employee onboarding engine.**

Part of the Sovereign Back-Office Suite replacing SaaS spend with zero-cloud-dependency, instant, offline-capable Rust native applications.

## Features

- **Candidate Pipeline**: Requisition tracking and structured 5-stage funnel (`applied` → `screening` → `interview` → `offer` → `hired`).
- **Blind Scorecards**: Structured quantitative evaluation (1-5 ratings across technical skills and cultural alignment) minimizing hiring bias.
- **Automated Fleet Onboarding Cascade**:
  - Emitting `candidate.hired` triggers automated simultaneous creation across the fleet:
    1. `contractcraft`: Generates Austrian employment agreement with statutory probation.
    2. `teamcraft`: Creates employee profile with Austrian AZG 38.5h weekly limit and 25-day vacation entitlement.
    3. `assetcraft`: Automatically reserves and assigns primary workstation hardware.
- **Tri-Modal Interface**:
  - **L1 CLI**: Fast candidate creation, stage progression, and scorecard rating.
  - **L2 MCP**: 8 FastMCP tools for AI agent orchestration.
  - **L3 GUI**: High-performance native desktop UI via `egui 0.37`.

## Architecture

- `crates/hirecraft-store`: Pure SQLite database layer with transactional integrity.
- `crates/hirecraft-core`: Stage transitions, blind scorecard aggregation, onboarding event dispatcher.
- `crates/hirecraft-mcp`: Stdio JSON-RPC MCP server & CLI binary (`hirecraft`).
- `crates/hirecraft-app`: Native desktop GUI executable (`hirecraft-app`).

## Quick Start

```powershell
# Run smoke tests
.\start.ps1 -Smoke

# Launch native desktop app
.\start.ps1

# Run MCP server on stdio
.\start.ps1 -Mcp
```

## License

MIT OR Apache-2.0
