Now go implement phase {phase.id}: {phase.title} of the Worth Store
Aspect-Native Workspace Gate.

State file: {state_file}
Spec file: {spec_file}
Cursor: phase {current.phase}, turn {current.turn}

Phase scope:
{phase.scope}

Acceptance evidence:
{phase.acceptance}

Use the in-chat plan from the previous runner turn as the working plan. If the
plan is missing or stale, rebuild the missing part inline in chat before
editing.

Implementation rules:

- Work phase-relevant code only, but follow real API boundaries wherever the
  phase leads.
- Do not implement unrelated S0-S12 physical database work.
- Keep JSON out of production authority, evidence, digest, handoff, recovery,
  certification, and ordinary harness paths except explicitly named terminal
  projection or hostile/readmission test boundaries.
- Prefer principled production surfaces over adapters, shims, fixture-only
  proof, renamed debt, or compatibility facades.
- Make invalid states unrepresentable where the codebase gives you a
  reasonable way to do that.
- Keep directories and files shaped to the explicit skeleton from the plan.
- Keep touched code and test files under the workspace line cap unless an
  explicit exemption already exists.
- Verify enough to know whether this phase is ready for the required
  phase-done loop.

Do not put logs, command output tails, artifacts, long plans, or review
findings into the JSON state. Put substantive implementation explanation in
chat. The JSON state is only progress tracking.

When done, update the JSON state with only short progress markers:

- `status: complete` and `qa_status: needed` if implementation is ready for the
  phase-done check.
- `status: in_progress` if implementation remains, with a short
  `notes.remaining` marker.
- `status: blocked` only for a precise blocker.

Advance the cursor to `review` only when implementation is ready for the
phase-done check; otherwise stay on `implement`.

Phase-specific instructions:
{phase.instructions}

{contract}
