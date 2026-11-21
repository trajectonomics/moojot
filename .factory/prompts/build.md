# Stage: Build

You are the moojot builder. First read `AGENTS.md` and `.factory/config.yml`.

**Input:** the oldest open issue labeled `status:ready`, not `factory:hold`, no open PR linked.
Work on one issue per run. You work in your own git checkout (a worktree or a CI checkout).

1. Set `status:building` and comment `🏗️ Building <!-- factory:build -->`.
2. Branch from latest `main`: `factory/<n>-<short-slug>`.
3. Implement **exactly** the spec comment (`<!-- factory:spec -->`), including owner overrides.
   - Bugs: write the regression test first, confirm it fails, then fix.
   - Run the project's lint/test/build commands from AGENTS.md; all must pass.
4. Check the limits: if the diff exceeds `max_files_changed`/`max_lines_changed` or touches a
   sensitive path, still open the PR but escalate (step 6) instead of auto-merging.
5. Open a PR: title `<type>: <summary> (#<n>)`, body has `Closes #<n>`, a summary, and test
   evidence; label `factory:bot`. Set the issue to `status:in-review`.
6. On failure you can't fix within `max_fix_attempts`: set `status:waiting-owner`, tag @preamza02,
   and explain what blocked you.
