# Issue tracker: Local Markdown

Issues and specs (also known as PRDs) for this project live as Markdown files in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`.
- The spec is `.scratch/<feature-slug>/spec.md`.
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01`; never use one combined tickets file.
- Triage state is a `Status:` line near the top of each issue file. See `triage-labels.md` for the role strings.
- Blocking edges use a `Blocked by: NN, NN` line near the top. A ticket is unblocked when all listed tickets are complete.
- Comments and conversation history append under a `## Comments` heading.

## Publishing

When a skill says to publish to the issue tracker, create the appropriate file under `.scratch/<feature-slug>/`, creating the directory when needed.

When a skill says to fetch a ticket, read the referenced file. The user will normally provide its path or number.

## Wayfinding operations

- Map: `.scratch/<effort>/map.md`.
- Child ticket: `.scratch/<effort>/issues/<NN>-<slug>.md`.
- Child metadata uses `Type:`, `Status:`, and `Blocked by:` lines.
- The frontier is the lowest-numbered open, unblocked, and unclaimed ticket.
- Claim by setting `Status: claimed` before work.
- Resolve by appending the result under `## Answer`, setting `Status: resolved`, and adding a context pointer to the map.
