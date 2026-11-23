# moojot software factory 🏭🐷

An issue goes in; a merged PR (or a clear question for the owner) comes out.

```mermaid
flowchart TD
    A["New issue (bug / feature form)"] --> T{"Triage: categorize + validate"}
    T -->|question| Q["Answer, close"]
    T -->|"duplicate / spam"| X["Link or close"]
    T -->|security| S["Hide details, tag owner"]
    T -->|"can't tell"| O
    T -->|"missing info"| NI["status:needs-info (reporter)"]
    NI -->|reply| T
    NI -->|"14 days"| C["close: resolution:stale"]
    T -->|"valid bug / feature / docs"| SP["status:spec: bot writes spec, sets size + risk"]
    SP --> D{"Auto allowed? (config.yml)"}
    D -->|"yes: S + low risk, and not a community feature"| R["status:ready"]
    D -->|no| O["status:waiting-owner: tag @preamza02"]
    O -->|"/approve or status:ready"| R
    O -->|"/reject"| W["close: resolution:wontfix"]
    O -->|"/info"| NI
    R --> B["status:building: builder agent"]
    B --> PR["status:in-review: reviewer model + CI"]
    PR -->|"all gates pass + auto allowed"| M["auto-merge, issue closed"]
    PR -->|"failed twice / not allowed"| O
```

## Who does what

| Stage | Trigger | Runs on | Prompt |
|---|---|---|---|
| Intake (labels, source, re-triage on reply) | issue/comment events | GitHub Actions, no AI | `.github/workflows/factory-intake.yml` |
| Owner commands | owner comments `/…` | GitHub Actions, no AI | `.github/workflows/factory-commands.yml` |
| Label move | write access adds a `status:*` label | GitHub Actions, no AI | `.github/workflows/factory-labels.yml` |
| Stale needs-info | daily | GitHub Actions, no AI | `.github/workflows/factory-stale.yml` |
| Triage | `status:triage`, every hour at :41 | GitHub Actions + [opencode](https://opencode.ai) with a **free** model (no API key) | `prompts/triage.md`, `.github/workflows/factory-triage.yml` |
| Spec | `status:spec` | AI agent | `prompts/spec.md` |
| Build | `status:ready` | AI agent (own git checkout) | `prompts/build.md` |
| Review | `status:in-review` | AI agent (different model) | `prompts/review.md` |

## Watch it run

Triage is the stage anyone can watch, live, in this public repo:

1. Open an issue with the bug or feature form. Intake labels it `source:*` + `status:triage` within seconds.
2. Within the hour, [factory-triage](../../actions/workflows/factory-triage.yml) runs. To skip the wait, use
   **Run workflow** (optionally with the issue number, or with `dry_run` to see decisions without any changes).
3. The bot comments on the issue and swaps the labels: `status:spec`, `status:needs-info`, `status:waiting-owner`,
   or it closes the issue (question / duplicate / spam). Each run's summary lists every decision and the model that made it.

How it stays safe with a free model: the model can only **read** the repo (`.factory/opencode.json` denies shell,
edits and web) and never sees the GitHub token. It returns a JSON decision, and `.factory/runner/triage.mjs` checks it
and applies it. The runner only applies allowed labels, strips @-mentions, posts a fixed reply for security reports,
and never closes an issue the owner opened. If a free model is rate limited, it tries the next one in `TRIAGE_MODELS`.

## Owner commands (comment on an issue or PR)

| Command | Effect |
|---|---|
| `/approve` | Spec OK, so move to `status:ready` (the builder picks it up) |
| `/reject [reason]` | Close as `resolution:wontfix` |
| `/info [question]` | Ask the reporter, so move to `status:needs-info` |
| `/spec [notes]` | Back to `status:spec` so the bot rewrites the spec with your notes |
| `/triage` | Re-run triage |
| `/size S\|M\|L`, `/risk low\|high` | Override the bot's estimate |
| `/stop`, `/resume` | Add/remove `factory:hold` |
| `/merge` | (PR) enable auto-merge once checks pass |

Applying the `status:ready` label yourself does the same thing as `/approve`. That is the tag that sends an issue to implementation, including a community feature that stopped at spec. Anyone with write, maintain, or admin on the repo can do it. Other `status:*` labels you add become the only status, so the matching stage runs next. `type:security` never queues, and `factory:hold` still blocks the builder until `/resume`.

## Kill switch
Set the repo variable `FACTORY_PAUSED=true` (Settings → Secrets and variables → Actions → Variables).
