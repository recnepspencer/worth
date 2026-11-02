# Phase6 controlled pinned derived read boundary review

Selected after typed Query recipe e589d6bd3a. The next root-path receiver requires
actual pinned derived acquisition that preserves its original caller stop and
observations even when no successful body escapes. Independent review of Native
pinned.rs and Query closure/ports/currentness confirmed the existing gap.

The actual Native pinned SourceRead owns immutable schema affinity, physical
entity/adjacency acquisition, event validation, original ExecutionRequest work and
retained memory. Its current derived_source accepts a work limit but no borrowed
caller control; failed evaluation drops the read/work observations. Query cannot
repair this with a timeout, synthetic zero counters, its own walker, or a second
request. Its established RootPathReadPorts stop may carry a richer original cause
than Native, so a borrowed narrowing callback must run throughout actual work.

This slice extends that same Native evaluator with a descriptive controlled scope
and failure envelope carrying original Native denial plus actual reads/work. The
scope grants no access and cannot supply a successful body. Existing public and
candidate-projection methods delegate unchanged through the new owner, preserving
legacy callers and typed denials. Read bodies keep their original reservation.
Control runs at every evaluator/physical checkpoint and final successful handoff;
the caller can retain its first richer stop externally and use NestedStopped.
Failure never returns partial endpoint data.

Complete Query observation/currentness/SourceFacts is the next receiver and remains
required. This Native seam is independently useful and verifiable over genuine
published event graphs, without changing schema/archive grammar or any build root.
The existing registered event probe fixture provides the runtime and transactions;
pinning uses its actual published observation/schema, not a rebuilt equal registry.
No new runtime factory, feature, test-only access or fabricated coordinates.
