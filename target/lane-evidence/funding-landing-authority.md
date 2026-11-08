# Coordinator ruling for this funding closeout

Latest user instruction, applied to the worktree at `5a49620b1d`:

> Standing lint-only rule (replaces per-file lint grants)
>
> To make Root and Store Clippy green, you may edit any file for a mechanical Clippy correction, when all of these hold:
> 1. Expression-equivalent only. No change to behavior, charging order, outcome, assertion strength, or a public signature. No allow attributes. Moving production helpers before a test module counts as mechanical.
> 2. Not Lifecycle's file. `git diff --quiet 7d0e5189bc codex/m3-p3-lifecycle -- <file>` must succeed, and the file must not be one of the ten exclusions.
> 3. Not already changed by master. If `git diff --quiet 7d0e5189bc origin/master -- <file>` fails, leave the file alone and list it: integration's master merge brings master's version, and the lint is rechecked there. If master has the file unchanged, fix it.
>
> Record every site you skip under rules 2 or 3 in your turn-end state, with its path and lint. Do not stop the turn for them: they're integration's to check after the master merge.
>
> This rule covers this lint fix and any lint you hit later in this session, and nothing else.
>
> Append `native-home-original-source .../checkpoint_admission/cold_materialization/selected_image.rs` exactly as in `proposed-funding-final-selected-reader-caller.patch`. Append only, as before.
>
> 1. Rerun full Root Clippy, then the boundary gate. The only boundary findings allowed are the three permitted BC9002 sites.
> 2. Run line caps and the agent-context gate.
> 3. Get a fresh independent review, commit, and report `FUNDING LANDED <sha>`.
>
> If Clippy stays red only at sites skipped under rule 3, you may still land. Report those sites. Anything else red blocks the landing.
>
> After that: do not merge integration until I name its head, then work the queue (decoder overflow, D13, named digests, BC9002).

Ten excluded files remain the exact set in the funding grants and scope-audit script. `branch/root_*` and the `checkpoint_complete/original_image.rs:69-70` digest fields remain excluded. No ignored tests, query-scale, exhaustive oracle, scenario suite, push, or unrequested integration merge is authorized.
