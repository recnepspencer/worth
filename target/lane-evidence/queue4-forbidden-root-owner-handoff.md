# BC9002: exact forbidden owner dependency

Status: IN PROGRESS. One external public-observation court is migrated and
passes; full adapter deletion and zero BC9002 are blocked by the explicitly
excluded branch/root_* owners. No excluded file was edited.

Required owners:
- `crates/worth-relational/src/branch/root_tests.rs`;
- `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs`.

The current instruction says branch/root_* stays untouched. These files are
therefore excluded even though their Lifecycle quiet-diff result is0. This is
not a claimed Lifecycle modification and not a blanket private-access blocker.

Actual dependencies:
- root_tests.rs::committed_root_and_envelope constructs the local durable
  runtime. Its callers exercise relink, completeness, visibility and the
  nested root_tests/request.rs cases. publication_cost_distinguishes_new_and_reused_schema_authority
  also constructs that runtime directly.
- root_authoritative_payload_tests.rs::strategy_artifacts_are_bound_into_the_committed_root
  passes local TestDurableMedium::new() to persisted_intent_runtime.
- root_authoritative_payload_tests.rs::schema_transition_descriptors_are_each_bound_into_the_committed_root
  calls durable_runtime_with_test_schema.

The latter tests exercise actual private root.relink_canonical_envelope and
require VisibilityCommitmentMismatch for strategy/schema authority edits.
A different public source-admission corruption refusal is not automatically
the same claim. They require their owning tests to migrate, or their owner to
split private value logic from public journeys, before the local provider can
be removed.

The support/runtime.rs durable builders instantiate
TestDurableMedium::new().attachment(). runtime_attachment.rs implements the
unit-test Native trait for that local attachment. The external testkit provider
implements the distinct normal-build Native trait. Keeping the forbidden
consumers unchanged therefore still requires a local identity adapter.
Renaming/reducing that adapter, adding a compatibility bridge, substituting
memory-only runtimes, hiding its request signatures, or growing a baseline
would evade the grant12 retirement instead of complete it.

The larger inventories deliberately distinguish262 direct medium/helper holder
files from104 direct adapter/provider-name references. These are not counts of
tests to move. The independent consumer audit traces mixed memory-only helpers
and public byte-tamper alternatives. Size alone, hidden Weak syntax and private
index counters are NOT blockers: public replay/funding and root-free local
fork/value units have valid existing solutions.

Completed partial migration:
`recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read` moves
from cold_envelope_lifetime.rs to the external durable_publication court. The
same real lazy member provider and exact source/receipt/failure/retry semantics
are retained. Public same-body shares and their intrinsic funding replace
private Arc/Weak syntax; a fixed weak-observer control bill correctly survives
until its runtime owner drops. No ceiling was raised. The name remains exactly
once; the four other original lifetime tests remain intact.

Compilation initially found an old private branch_head call in the migrated
court. That test integration error was corrected using the actual public
owner-admitted main-branch basis, without a facade change. Final focused check
and external court are green. This partial migration is not a zero-BC9002 or
full grant12 completion claim. The three local adapters and baseline remain
visible pending the forbidden-owner handoff.
