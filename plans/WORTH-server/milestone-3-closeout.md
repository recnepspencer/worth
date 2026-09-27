# Milestone 3 Closeout: External HTTP, Streaming, Binary, And Blob Surface

## Status

Milestone 3 is closed as of 2026-06-09 for the external compatibility HTTP,
streaming/export, multipart upload, binary download/range/resume, metadata
linkage, normalization/cacheability, operator evidence, and transfer-lifecycle
surface in `worth-server`.

This closeout covers:

- one typed compatibility HTTP root for read, mutation, streaming, upload, and
  download route families
- Query-first compatibility reads, state, inspection, and mutations with typed
  branch, basis, diagnostics, denial, and provenance posture
- streamed versus buffered export parity with explicit buffering-honesty
  counters and cancellation classification
- multipart upload admission, early rejection, chunked/compressed ingress
  bounds, staged upload cleanup, and integrity verification
- range transfer, admitted runtime-backed resumable download posture, and
  binary integrity verification
- explicit linkage between file metadata truth and binary transfer policy
- canonical filename and metadata normalization plus intermediary cache-safety
  posture
- separate external and binary certification bundles, counters, abuse-budget
  receipts, operator evidence, and transfer lifecycle evidence
- hostile closeout certification for parity, retry/precondition honesty,
  multipart/range miserable paths, normalization hostility, transfer hostility,
  and blob/truth separation

This closeout does not claim:

- durable restart-stable resume for downloads or any broader delivery surface
- lease registry, sync transport, active subscription delivery, shared-base
  reuse, or view-patch semantics
- integration/webhook/background-delivery milestones beyond the explicit
  background export fallback admitted inside the compatibility surface
- WebTransport, blind-server, cluster, or durable persistence topology work

Those remain later roadmap scope exactly as the milestone and vision allowed.

## Governing Source Summary

- `MENTALITY.md`: the milestone had to close the hostile "second server brain"
  problem rather than merely make HTTP routes return plausible bodies.
- `arch_laws.md`: compatibility HTTP, streaming, and binary transfer had to
  stay one facade over Query-first handoff and typed server-owned artifacts.
- `composition_laws.md`: reads, mutations, streaming, uploads, downloads,
  metadata linkage, normalization, evidence, and lifecycle accounting had to
  remain separate named responsibilities.
- `domain_structure_laws.md`: structured truth delivery, binary transport,
  metadata truth, denial/policy posture, and operator evidence had to remain
  physically distinct.
- `perf_laws.md`: buffering honesty, ingress/egress bounds, and exact counter
  contracts had to stay explicit rather than hiding broad materialization or
  route-local rediscovery.
- [milestone-3.md](./milestone-3.md): the shipped surface satisfies the merged
  compatibility/binary milestone and its hostile certification closure bar.
- [test-requirements.md](./test-requirements.md): the shipped tests satisfy the
  Milestones 1-3 compatibility-path honesty standard through narrow-artifact,
  typed-failure, and exact-zero-counter certification.

## Adversarial Constraint Closed

Milestone 3 had to survive the hostile case where the external server surface
would:

- redefine Query meaning at the route boundary
- treat streaming as a second read semantics
- route blob bytes through sync-style structured truth payloads
- flatten validation, policy, provenance, and support posture into status-code
  folklore
- let retries, ranges, multipart perturbations, proxy headers, or cache layers
  silently change canonical meaning
- certify overlap through broad response equality instead of narrow canonical
  artifacts

The closed surface now guarantees that:

- compatibility HTTP remains an interop projection over canonical server and
  Query artifacts rather than a second runtime
- streaming changes delivery mechanics only and preserves canonical read/export
  meaning where parity is claimed
- metadata truth and raw blob motion remain explicit, linked, and
  non-conflated
- range, resume, integrity, cleanup, and pacing hostility fail typed at the
  expected boundary
- operator reconstruction and transfer accounting remain possible through
  retained artifacts alone
- runtime-backed-now versus durable-later posture stays explicit anywhere
  resume-like semantics are visible

## Closure Summary

Milestone 3 closes as one merged external surface, not as a pile of unrelated
endpoint features.

What now ships:

- compatibility entry/root admission with canonical request-contract identity
- Query-first read/state/inspection routes with basis and validator posture
- Query-first mutation routes with idempotency and precondition closure
- incremental streaming and buffered export over one canonical meaning model
- multipart upload admission, early rejection, integrity, and cleanup
- range download, runtime-backed resume negotiation, and binary integrity
- metadata truth linkage, normalization, and cacheability policy
- external and binary evidence/counter bundles plus abuse/lifecycle accounting
- certification-grade hostile test closure across phases 1 through 13

What intentionally does not ship as part of this closeout:

- durable restart-stable resume or store-backed replay
- lease/session sync protocol families
- integration/webhook/CDC surfaces
- distributed or transport-upgrade capability families

That distinction matters. The shipped compatibility and binary surface is real,
but it stays honest about runtime-backed boundaries and does not market later
delivery or durability milestones as already solved.

## Public Surface Closed

The public external surface now closes through the compatibility and binary
facade families in `worth-server`, including:

- compatibility request-contract entry
- compatibility read/state/inspection execution
- compatibility mutation execution
- compatibility streaming/export execution
- multipart upload admission and execution
- binary download/range/resume execution
- file metadata linkage, normalization, and cacheability policy
- operator evidence, certification bundles, and abuse/lifecycle accounting

Representative proof-facing test surfaces include:

- [compat_http_entry.rs](../../crates/worth-server/tests/compat_http_entry.rs)
- [compat_http_phase_two.rs](../../crates/worth-server/tests/compat_http_phase_two.rs)
- [compat_http_phase_three.rs](../../crates/worth-server/tests/compat_http_phase_three.rs)
- [compat_http_phase_four.rs](../../crates/worth-server/tests/compat_http_phase_four.rs)
- [compat_http_phase_five.rs](../../crates/worth-server/tests/compat_http_phase_five.rs)
- [compat_http_phase_six.rs](../../crates/worth-server/tests/compat_http_phase_six.rs)
- [compat_http_phase_seven.rs](../../crates/worth-server/tests/compat_http_phase_seven.rs)
- [compat_http_phase_seven_boundary.rs](../../crates/worth-server/tests/compat_http_phase_seven_boundary.rs)
- [compat_http_phase_eight.rs](../../crates/worth-server/tests/compat_http_phase_eight.rs)
- [compat_http_phase_eight_boundary.rs](../../crates/worth-server/tests/compat_http_phase_eight_boundary.rs)
- [compat_http_phase_nine.rs](../../crates/worth-server/tests/compat_http_phase_nine.rs)
- [compat_http_phase_nine_boundary.rs](../../crates/worth-server/tests/compat_http_phase_nine_boundary.rs)
- [compat_http_phase_ten.rs](../../crates/worth-server/tests/compat_http_phase_ten.rs)
- [compat_http_phase_ten_boundary.rs](../../crates/worth-server/tests/compat_http_phase_ten_boundary.rs)
- [compat_http_phase_eleven.rs](../../crates/worth-server/tests/compat_http_phase_eleven.rs)
- [compat_http_phase_eleven_boundary.rs](../../crates/worth-server/tests/compat_http_phase_eleven_boundary.rs)
- [compat_http_phase_twelve.rs](../../crates/worth-server/tests/compat_http_phase_twelve.rs)
- [compat_http_phase_twelve_boundary.rs](../../crates/worth-server/tests/compat_http_phase_twelve_boundary.rs)
- [compat_http_phase_thirteen.rs](../../crates/worth-server/tests/compat_http_phase_thirteen.rs)
- [compat_http_phase_thirteen_boundary.rs](../../crates/worth-server/tests/compat_http_phase_thirteen_boundary.rs)

## Verification Summary

The closeout state is grounded in the full Milestone 3 compatibility/binary
test stack, including:

- `cargo check -p worth-server --tests`
- `cargo test -p worth-server --test compat_http_entry --test compat_http_phase_two --test compat_http_phase_three --test compat_http_phase_four --test compat_http_phase_five --test compat_http_phase_six --test compat_http_phase_seven --test compat_http_phase_seven_boundary --test compat_http_phase_eight --test compat_http_phase_eight_boundary --test compat_http_phase_nine --test compat_http_phase_nine_boundary --test compat_http_phase_ten --test compat_http_phase_ten_boundary --test compat_http_phase_eleven --test compat_http_phase_eleven_boundary --test compat_http_phase_twelve --test compat_http_phase_twelve_boundary --test compat_http_phase_thirteen --test compat_http_phase_thirteen_boundary -- --nocapture`

That verification passed with 78 tests green across:

- compatibility entry and request-contract parity
- read/state/inspection parity and basis localization
- mutation parity, idempotency, and preconditions
- streaming parity, buffering honesty, and cancellation accounting
- multipart admission, malformed-part hostility, ingress bounds, and cleanup
- range/download/resume integrity and runtime-backed honesty
- metadata truth linkage and transfer divergence resistance
- normalization and intermediary cache-safety closure
- operator reconstruction and diagnostics-richness invariance
- abuse-budget and transfer lifecycle accounting
- Phase 13 hostile certification closure

## Residual Deferred Scope

The following are intentionally not part of Milestone 3 closeout:

- durable restart-stable resume or restart-stable replay for binary or sync
  delivery
- lease identity and reconnect contracts
- WebSocket/WebTransport sync protocol lanes
- remask/permission drift on active subscriptions
- shared subscription bases, view patches, or server-side materialization
- integration-facing CDC, webhook, outbox, or typed external-source work
- blind-server, topology, or cluster coordination capability families

Later milestones must consume the shipped compatibility/binary boundary as
closed instead of reopening request canonicalization, metadata linkage,
transfer accounting, or blob/truth separation at those later surfaces.

## No Open Blocker

No open Milestone 3 blocker remains at closeout.

The remaining work is later roadmap work, not unresolved debt inside this
milestone's compatibility/binary contract.
