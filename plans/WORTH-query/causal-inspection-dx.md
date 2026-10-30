# Causal Inspection Application DX

The ordinary application contract uses `worth-query-decl` and `worth-query-host`.
A genuine performed source receipt or completed query observation supplies original
evidence; it supplies no current permission. Fresh Host requests own READ, metadata,
diagnostics, currentness, funding and their consuming terminal.

The executable consumer is
[application_runtime_surface](../../workspaces/worth-query/crates/worth-query-host/tests/application_runtime_surface.rs),
including [inspection](../../workspaces/worth-query/crates/worth-query-host/tests/application_runtime_surface/inspection.rs)
and [rich signatures](../../workspaces/worth-query/crates/worth-query-host/tests/application_runtime_surface/inspection/rich.rs).
It installs the existing schema through public Host `in_memory` construction with
explicit resources and real build identity. It requires no internal Execution
factory. The
[integrated correction journey](../../workspaces/worth-query/crates/worth-query-certification/examples/application_correction_support/causal_inspection/output_equivalence/history.rs)
executes actual source/producer/query handoffs. Signature compilation alone is
not its runtime evidence.

## Retain the original and admit a new request

Keep `result.receipt().inspect().observation().unwrap().clone()` before dropping
the result. Query executions opt into `.retain_projection_for_inspection()` when
later comparison or original values are needed. Complete empty results retain a
real empty projection; absent capture is NotRetained. Clones share the original
payer and immutable owners, with no snapshot reader or current read permission.

Use `fresh.inspect::<Binding>(&original).why_observed().reference_only()` with
the binding's current metadata admission. This establishes Observed, not Changed.
A paired `.why_changed(&before)` independently admits both originals and their
diagnostics. Source-only inspection uses
`fresh.inspect_source::<SourceDisclosure>(&receipt).why_performed()`.

The [capability guide](../../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md)
owns exact signatures, declarations, supported shapes and refusal handling. The
[application guide](../../docs/build-an-application.md#inspect-original-source-and-query-evidence)
connects acquisition, admission, rich views and close.

## Select the intended causal meaning

| Intent | Public request and evidence |
| --- | --- |
| Inspect one original | `why_observed()`; genuine original binding, scope, parameters and basis |
| Compare original results | `why_changed(&before)`; both complete original projections, with current diagnostics |
| Establish creation | `caused_by::<SourceDisclosure>(&source)`; independent source/query admissions and both exact Native joins |
| Explain a performed output comparison | `evaluated_by::<SourceDisclosure>(&source)`; actual qualified Native/Signal pair, Changed or Equivalent |
| Materialize original query values | `materialize(Detail::family(QueryInspection))`; original Native disclosed tree and current diagnostics |
| Materialize canonical source effects | `Detail::source_effects()` or `.with_source_effects()`; original canonical owner and exact source scope |
| Include the actual connection/evaluation | `.with_producer_evidence()` on a connected rich request |
| Read optional supplier history | `Detail::conditional_history()` or `.with_conditional_history()` on connected rich inspection; exact owner association and both diagnostics |

The source input need not appear in the output query, and the two subjects can
differ. Neither binding lends governance to the other. Equal rewrites, no-op
clears, unrelated producers, replaced generations and current head values cannot
stand in for the actual source-created occurrence. Equivalent output preserves
its observed revision and cannot become a new creation connection. Stable input
reuse and checkpoint restoration do not recreate a performed evaluation.

Supported output equivalence is the qualified Required standard-u64 field on an
exactly-one Preserve role with installed `output_equivalence()`. Other supported
connection shapes and source-effect limits remain explicit in the guide; this
handoff does not expand them by rendering a generic narrative.

## Availability, loss and close

Availability is frozen once after current metadata admission and fed to both the
published payload and query anchor v3. Report getters do no later owner lookup.
Projection, committed/stable/restored supplier custody and conditional Native
observation remain distinct. Original immutable custody has no TTL.

Optional conditional history belongs to the actual Bridge diagnostics owner.
The owner issues it at successful execution, funds its fixed slot from that
request and applies its real recording/retention policy. Minimal tier alone does
not mean omission. Actual recording-disabled policy, finite capture denial,
retention eviction and owner closure have separate typed outcomes. A later rich
lookup independently rechecks the same original; earlier available metadata is
no lease. Rich materialization v3 binds its request, actual outcome and supplied
readonly facts. History describes supplier readiness, not the output-comparison
pair.

A new source execution can evict history while the original source, query and
performed evaluation survive. An earlier admitted rich snapshot remains usable.
Requested-but-unprovided modes and unrequested facets stay explicit, with no
empty-list inference or invented absence. Plain query/source-only mode cannot
acquire connected history; generic Signal forensic/replay and legacy Bridge
record families are not relabeled into this facet.

Revoked metadata refuses before protected discovery. Wrong diagnostics cannot
be borrowed from the other subject. Actual cancellation after private body
assembly prevents publication and releases fresh funding; it is not evidence
expiry. Close each report and the last original holder. Closing one sibling
leaves the survivor usable, and owner eviction detaches successor custody.

## Retained platform audience

The older `worth-query` engine keeps its separately authorized legacy
`QueryObservationReceipt` plus `ScopedInspectionBasis` orchestration. It is not
an application entrance for current original observations. Publication forwards
current observations through the narrow `application_inspection` contract; Host's
inspection namespace cannot import the legacy builder/basis. Query and Bridge
Native-materialization fences require the fresh application route.

Preserve that legacy receipt audience rather than creating a second application
engine or requiring raw Bridge/Signal/Relational handles. Durable causal archives,
restart-stable narrative reload and generic forensic reconstruction remain later
work. They are not implied by these in-memory original/reference/rich guarantees.
