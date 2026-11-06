# Funding merge and queue progress

Parents: P-A `25619ac5d1`, integration `14b16fa292`.
Validation budget so far: 1 Root Clippy run (failed at Foundation's retired
Bytes variant); 0 focused builds / 0 test runs. No Query/Store compile yet.
All emitted diagnostics and both source-review P1 cutovers are fixed together
before another compile; downstream Native diagnostics were masked by that
Foundation failure. Root compile 2 then reported five Native errors (retired
AspectShape::Content plus four content-denial matches) and two unused export
warnings. All are fixed as one family, including exact content-stop carriage
through the borrowed wire visitor and the entire unused reexport chain.
The third compile round runs Root, Query and Store sequentially and stops on
the first failing workspace rather than repeating the shared Native failure.
Current counts: 2 Clippy / 0 checks / 0 focused builds / 0 test runs.

Budget exception, recorded before Query/Store and focused runs: completing the
explicit three-workspace merge proof requires more than four individual
check/build/run commands. Two Root rounds exposed prerequisite-masked enum
cutovers; round 3 now compiles Native and continues dependent courts. Query
and Store each still require their FIRST compile, then one batched focused
selection per affected binary. No green unchanged workspace is being rechecked.
Current running count: 3 Root Clippy invocations; zero Query/Store compiles,
focused builds or tests yet. The 33 current Native warnings match the accepted
funding-landing-root-clippy.log by diagnostic text and path, with no additions.

Boundary: integration contributes master, Lifecycle and content/digest owner
contracts; P-A contributes backing-local funding, request preflight, selected
state capture and accepted queue evidence. No covered-history fold may return.
The actual captured roots retain integration's content directory through their
own owner custody. Original never-checkpointed recovery remains journal based.

Merge plan: preserve actual captured roots and selected capture; compose named
digest access with the funded cold read path; share the integration projection
validation engine while keeping request-owned and controlled-publication
allocation at their respective backing owners. Keep both exact caller lists.
Merge scheduling records by immutable sample identity and logical three-way
baseline records, then regenerate with the existing catalog owner. Do not
choose generated shard filenames as the merge unit.

Qualification: first batch is one Root, Query and Store all-target Clippy
compilation each, sequentially. Read/classify all diagnostics before the next
compile round. Focused proof names come from funding-landing-lanes.md,
funding-controls-native.log, funding-final-{host,bounded,store,cdc-recovery}.log
and queue-resume-lanes.md. Combine names per binary and keep ignored tests and
exhaustive suites unselected. Format, line caps, boundary and context at phase
completion. A fresh independent reviewer verifies the final merge and logs.

Then commit/report FUNDING MERGED and implement D13 (including authenticated
World originals and predecessor refusal), named digests and BC9002 in order.
BC9002 assertion equivalence is designed and independently reviewed before
editing the now granted root production seams. No pushing.

Review correction: source review01 REJECT P1 identifies new content refusals
from tail canonical readmission mapped to corruption. Preserve Content via the
existing NativeCaptureDenial conversion (Execution normalizes to Request).
An extracted exhaustive pure refusal map and its runtime-free regression are
the only source changes after the compile/source tree
c7e6413c053ef82c98dbccf8bfb3bdad930c6bde; it cannot affect the prior selected
scalar-only controls. Query Native codegen had completed before this correction;
Store has not yet begun. One post-review Root Clippy + focused Root regression
packet and phase gates follow, without repeating unchanged green workspace checks.
