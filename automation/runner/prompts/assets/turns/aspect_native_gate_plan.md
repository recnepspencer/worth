Now create an implementation plan for phase {phase.id}: {phase.title} of the
Worth Store Aspect-Native Workspace Gate.

State file: {state_file}
Spec file: {spec_file}
Cursor: phase {current.phase}, turn {current.turn}

Phase scope:
{phase.scope}

Acceptance evidence:
{phase.acceptance}

This is a gate-closure phase, not an S.* physical database feature phase.
Plan only the current phase, and make the plan precise enough that the runner
can decide whether this phase is actually done before moving on.

Do the following:

- Follow our arch, perf, composition, domain-structure, and DX laws.
- Review relevant context, including the gate spec, roadmap, this phase's
  scope paths, the dedicated Store workspace, the legacy Store JSON/serde
  residue only as compatibility/residue inventory, and relevant
  worth-foundational aspect APIs.
- Read `docs/coding-guidelines/dx_laws.md` if it exists.
- Plan the directory skeleton explicitly.
- Plan the DX target via DX laws as an actual Rust code block target.
- Make implicit requirements explicit.
- Build the plan inline in chat, not in an md file.
- Be explicit about what this phase requires, changes, proves, and denies.

The plan must include:

1. Relevant context read and what it constrains.
2. The phase-specific adversarial constraint.
3. The DX target as a Rust code block.
4. Directory skeleton and each file/module's responsibility.
5. Native authority, terminal projection, compatibility residue, and forbidden
   JSON/serde boundaries for this phase.
6. Implementation sequence, with requirements, edits, proof, and blockers.
7. Done criteria for this phase, distinct from whole-gate closure.

Phase-specific instructions:
{phase.instructions}

After posting the plan in chat, update the JSON state only as lightweight
progress state: set this phase to `in_progress`, keep only a short `notes.plan`
marker, and advance the cursor to `implement`.

Do not put logs, artifacts, evidence dumps, long plans, or review findings into
the JSON. The JSON is only progress tracking.

{contract}
