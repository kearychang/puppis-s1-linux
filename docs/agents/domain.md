# Domain docs

This is a single-context project.

## Before exploring

- Read root `CONTEXT.md` when it exists.
- Read the ADRs under `docs/adr/` that affect the area being changed.
- If either is absent, proceed silently; domain-modeling workflows create them lazily when decisions require them.

## Vocabulary

Use the canonical terms from `CONTEXT.md` in issue titles, specifications, implementation plans, tests, errors, and UI text. Do not substitute terms listed under `_Avoid_`.

If a needed domain concept is missing, reconsider whether the new term is necessary or note the gap for a domain-modeling session.

## ADR conflicts

Surface any contradiction with an existing ADR explicitly instead of silently overriding it. Identify the ADR and explain why reopening it may be warranted.
