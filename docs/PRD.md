# HireCraft PRD

Status 12 November 2026.

## What it does

- Requisitions, candidates, and stages: applied, screening, interview, offer, hired, rejected.
- Scorecards with technical and culture ratings. The store averages them. One store test.
- `hire_candidate` writes `candidate.hired` to the contractcraft, teamcraft, and assetcraft inboxes. One core test checks that three files appear.

## What it does not do yet

- Refuse a score outside 1–5.
- Make the three downstream apps consume the event (no contract, no employee, no reserved asset).
- Replace Greenhouse, Lever, or Ashby.

## Done when

The score range is tested, and each downstream app has a test that drains `candidate.hired`. See `BACKOFFICE_THIN_RUNT_PLAN.md`.
