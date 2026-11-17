# HireCraft PRD

Status 16 November 2026.

## What it does

- Requisitions, candidates, and stages: applied, screening, interview, offer, hired, rejected.
- A candidate pane: name, email, requisition, stage, scorecard average, notes, offered salary, Hire. This is a candidate record. It is not an employee card.
- Scorecards with technical and culture ratings. The store averages them. One store test.
- `hire_candidate` writes `candidate.hired` to the contractcraft, teamcraft, and assetcraft inboxes. Payload today: `candidate_id`, `name`, `email`, `req_id`, `salary_eur`, `azg_weekly_cap` 38.5, `annual_leave_days` 25. One core test checks that three files appear.
- Downstream drains that exist in those apps (checked 2026-11-16): ContractCraft opens a draft employment contract; TeamCraft opens a thin employee and drops the wage; AssetCraft records an equipment reminder and does not invent an asset or a cost.

## What it does not do yet

- Refuse a score outside 1-5.
- Carry start date, position, or work text on `candidate.hired`.
- An employee page. Personal data, photo, hire date, position, and current work live on the TeamCraft Mitarbeiterkarte. Founding staff are entered by SetupCraft and do not pass through this pipeline. Spec: `mcp-central-docs/projects/setupcraft/PRD.md`.
- Replace Greenhouse, Lever, or Ashby.

## Planned hire payload

When SetupCraft's later-hire path is built, Hire still means "this candidate is hired". The event adds a CEO-chosen start date, the requisition's position title, and a work-text draft. TeamCraft stores the offered gross as monthly gross and opens an incomplete card until photo, SVNR, and address are filled. HireCraft links to that card. It does not become the HR file.

## Done when

The score range is tested. The richer hire payload has a core test, and TeamCraft's drain test shows the card fields and the stored gross. See `BACKOFFICE_THIN_RUNT_PLAN.md` for the original thin-runt bar. The 2026-11-16 code already drains the thin event.
