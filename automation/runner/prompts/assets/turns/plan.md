Now lets create an implementation plan for phase {phase.id}: {phase.title}.

Config file: {config_file}
Projection file: {projection_file}
Event log file: {event_log_file}
Spec file: {spec_file}
Run id: {run_id}
Cursor: phase {current.phase}, turn {current.turn}

Phase scope:
{phase.scope}

Acceptance evidence:
{phase.acceptance}

Do the following:

- Make sure to follow our arch and perf laws.
- Review relevant context, including the spec, this phase's scope paths, and the
  relevant APIs.
- If this run has a prior boundary review turn for the current phase, use that
  boundary brief as planning input.
- Read `docs/coding-guidelines/dx_laws.md` if it exists.
- Plan the directory skeleton explicitly.
- Plan the DX target via DX laws as an actual Rust code block target.
- Make implicit requirements explicit.
- Build the plan inline in the chat, not in an md file.
- Include the phase-relevant plan.
- Be explicit about what each step requires, changes, and proves. Do not give me
  a loose list of adhd bullet points.

The plan should cover:

1. Relevant context read and what it constrains.
2. The adversarial constraint this phase must survive.
3. The DX target as a Rust code block.
4. The directory skeleton and each file/module's responsibility.
5. Implicit requirements made explicit.
6. The implementation sequence, with requirements, edits, proof, and blockers.

Phase-specific instructions:
{phase.instructions}

After posting the plan in chat, finish with:

`RUNNER_EVENT: {"event_type":"plan_posted","payload":{"notes":{"plan":["..."]}}}`

{contract}
