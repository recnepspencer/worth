# Milestone 3: External HTTP, Streaming, Binary, And Blob Surface

> **Shipped closeout:** [milestone-3-closeout.md](./milestone-3-closeout.md)

## Goal

Ship the full external compatibility surface for `worth-server` so non-WORTH
clients, export consumers, upload/download flows, and operational HTTP callers
can use one honest request/response, streaming, multipart, and range-transfer
boundary without creating a second meaning model beside Query or letting blob
transport leak into structured truth delivery.

## Why This Milestone Exists

After Milestone 1 and Milestone 2, the server already has one forced-entry
pipeline and one first-class WORTH-native direct-consumption surface. The next
naive failure would be to treat external HTTP and blob transfer as "just add
routes":

- rebuild reads, mutations, branch targeting, and denial shaping inside route
  handlers
- let streaming responses invent a second read meaning because the transport
  shape changed
- run multipart and binary transfer through ad hoc route code that bypasses
  the same auth, tenant, branch, and evidence pipeline
- couple blob transport to sync-style structured truth delivery
- flatten validation, support, and provenance into status-code folklore for
  external clients
- treat exports, uploads, downloads, and metadata truth as one untyped file
  feature

This milestone exists to prevent that failure by shipping the entire external
surface as one architectural unit.

## Governing Summaries

- `MENTALITY.md`: external usefulness does not justify route-local shortcuts;
  the milestone must solve the hostile "second server brain" problem first.
- `arch_laws.md`: compatibility routes, streaming surfaces, and binary lanes
  must stay one facade over Query-first handoff, typed denial, and typed
  boundary artifacts rather than each owning their own meaning.
- `composition_laws.md`: read HTTP, mutation HTTP, streaming, upload ingress,
  download egress, binary policy, and certification must remain separate named
  responsibilities instead of collapsing into a generic transport module.
- `domain_structure_laws.md`: structured truth delivery, binary transport,
  metadata truth, validation/policy denial, and operator evidence have
  distinct authority and lifecycle and must stay physically separate.
- `perf_laws.md`: buffering, broad scans, route-local replay, blob-in-memory
  materialization, and repeated basis/policy rediscovery are forbidden cost
  patterns for this surface.
- `worth_server_roadmap.md`: Milestone 3 is the merged external surface
  milestone and intentionally absorbs the earlier standalone blob milestone so
  request/response and binary transfer land together as one honest boundary.

## Adversarial Constraint

For the same authenticated principal, tenant/workspace target, branch/basis
posture, remask posture, canonical Query declaration identity, canonical Query
intent, file metadata truth, and diagnostics richness, the external
compatibility surface must admit, deny, read, inspect, mutate, stream, upload,
download, and explain results through the same canonical server-owned meaning
as the WORTH-native facade while keeping blob bytes structurally separate from
structured truth delivery.

This milestone fails if any compatibility or binary surface:

- widens or redefines Query meaning at the route boundary
- treats streaming as a semantic fork instead of a transport shape
- routes file bytes through sync-style structured truth delivery lanes
- hides branch, basis, support, or provenance posture because the caller is
  "just HTTP"
- reconstructs metadata truth or policy locally instead of consuming server
  and Query contracts
- buffers large uploads or downloads dishonestly when the surface claims
  streamed transfer
- or proves parity only through broad top-level response equality instead of
  narrow canonical artifacts

## Product Decision Lock

- Milestone 3 builds one external surface. HTTP reads, HTTP mutations,
  streaming responses, multipart upload, range transfer, resumable download,
  and binary policy closure ship as one architectural unit.
- Compatibility HTTP is an interop lane, not the center of gravity of the
  product. It consumes the same server and Query truth that the WORTH-native
  surface already uses.
- Streaming changes transport mechanics only. It does not create alternate
  read, mutation, or export semantics.
- Blob transport is not sync transport. Structured truth delivery and binary
  transfer remain separate lanes even when metadata truth changes because of
  the same product action.
- File metadata remains canonical truth linked through Query-facing semantics.
  Raw blob motion is a server transport responsibility, not a second truth
  model.
- Runtime-backed-now versus durable-later posture remains explicit anywhere
  resume-like or basis-bearing semantics are visible.

## Phase Plan

### Phase 1: Compatibility Surface Root And External Request Contract Boundary

Freeze the public compatibility HTTP root so all external request/response,
streaming, upload, and download surfaces enter through one typed external
contract family rather than ad hoc route-local DTOs.

**Relevant subsystems**
- compatibility surface root
- external request contract decoding
- route-family registration for compatibility and binary lanes
- external schema and parameter validation
- external header/query canonicalization
- compatibility version and negotiation policy

**Relevant APIs**
- `WORTHServerCompatibilitySurface`
- `WORTHServerCompatibilitySession`
- `WORTHServerCompatibilityRequest`
- `WORTHServerCompatibilityRouteFamily`
- `WORTHServerExternalRequestContract`
- `WORTHServerCompatibilityVersion`
- `WORTHServerNegotiatedRepresentation`
- `WORTHServerCanonicalHeaderSet`

**Relevant Query surfaces**
- None. This phase freezes the external request contract family before
  operation-specific Query lowering begins.

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. External request entry must
  consume the Milestone 1 request-context and pipeline proof instead of adding
  a compatibility-only progression model.
- Use no new `worth-foundational` surfaces in this phase. Request decoding and
  route-family topology remain server-local entry structure here.

**Warnings**
- Do not let compatibility handlers accept anonymous parameter bags with no
  canonical request-contract identity.
- Do not make upload and download endpoints special bootstrap paths outside
  the ordinary compatibility surface root.
- Do not expose generic "raw HTTP handler" escape seams.
- Do not let duplicate headers, repeated query keys, forwarded-host ambiguity,
  or content-negotiation fallback remain route-local interpretation details.
- Do not treat `HEAD`, `OPTIONS`, or browser preflight handling as informal
  framework defaults if they affect auth, caching, or route admission.

**Test requirements**
- Add a compatibility-entry parity test proving equivalent external requests
  lower to the same canonical request-context artifacts as existing direct
  facade operations where overlap exists.
- Add a hostile request-contract test proving malformed path/query/body/header
  combinations fail at typed request-contract boundaries before Query-facing
  lowering begins.
- Add a route-family isolation test proving read, mutation, streaming, upload,
  and download routes register as distinct compatibility families under one
  external root instead of hidden handler-side forks.
- Add a header-canonicalization hostility test proving duplicate headers,
  repeated query keys, mixed casing, forwarded-proto/host ambiguity, and
  malformed negotiation headers normalize once into one canonical request
  contract or fail typed before semantic lowering.
- Add a compatibility-method test proving `HEAD`, `OPTIONS`, and admitted CORS
  preflight paths preserve route and auth classification without silently
  widening visible capability posture.

**Engineering decisions**
- The external surface owns typed compatibility request contracts, not
  endpoint-local parameter folklore.
- Route-family separation starts here so later streaming and binary work do
  not collapse into one generic HTTP bucket.
- Representation negotiation, API versioning, and header canonicalization are
  part of the external semantic boundary, not incidental web-framework glue.

**Open questions**
- None.

### Phase 2: Query-First HTTP Read, State, And Inspection Boundary

Freeze the ordinary compatibility read surface so external callers can perform
one-shot reads, admitted state access, inspection access, branch-aware reads,
and admitted historical/basis targeting without the server recreating meaning
at the transport edge.

**Relevant subsystems**
- compatibility read route family
- state and inspection route family
- branch/basis request lowering
- canonical response envelope shaping for reads
- conditional read admission
- cache and validator posture

**Relevant APIs**
- `WORTHServerCompatibilityRead`
- `WORTHServerCompatibilityState`
- `WORTHServerCompatibilityInspection`
- `WORTHServerExternalBasisRequest`
- `WORTHServerResponseEnvelope`
- `WORTHServerConditionalRead`
- `WORTHServerReadValidator`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`
- admitted branch-aware and basis-aware Query-facing read surfaces

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Compatibility reads must
  consume existing server request-context, admission, and Query-handoff proof
  artifacts.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` for external
  diagnostics richness posture when read/state/inspection responses expose
  adjustable detail.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
  and `FoundationalBoundaryEvidenceProvenanceFrontDoor` when compatibility read
  envelopes project server-owned provenance and evidence beyond raw payload
  bodies.

**Warnings**
- Do not merge read, state, and inspection into one ambiguous "fetch"
  endpoint family.
- Do not widen branch or basis semantics because the caller sent generic HTTP
  parameters instead of a WORTH-native declaration object.
- Do not teach support posture through undocumented route behavior.
- Do not let CDN or intermediary caching infer tenant, branch, remask, or auth
  safety from generic defaults.
- Do not let conditional read headers become best-effort hints; they are part
  of the read contract once admitted.

**Test requirements**
- Add a compatibility-read parity test proving equivalent direct-facade and
  compatibility read lanes compare equal on canonical declaration, basis,
  support, provenance, and response artifacts where overlap exists.
- Add a basis-localization test proving malformed, unsupported, or drifted
  branch/basis combinations fail typed before canonical read execution.
- Add a state-versus-inspection differentiation test proving admitted retained
  state and inspection routes remain mechanically distinct and do not collapse
  into one route-local payload family.
- Add a conditional-read parity test proving validator-bearing reads,
  `If-Match`/`If-None-Match`-style preconditions, and `HEAD` equivalents either
  preserve the same canonical basis/provenance story as the corresponding
  admitted read or fail typed at the precondition boundary.
- Add a cache-safety test proving branch-, tenant-, remask-, and auth-scoped
  responses emit explicit cacheability posture and cannot be misclassified as
  publicly reusable artifacts.

**Engineering decisions**
- External reads stay Query-first and declaration-first even when expressed
  through HTTP path/query/body vocabulary.
- State and inspection remain separate compatibility responsibilities because
  they differ semantically even when they reuse the same route family root.
- Conditional request semantics and cache validators are first-class external
  read contracts because they are how non-WORTH clients prove equivalence and
  freshness without semantic guesswork.

**Open questions**
- None.

### Phase 3: Query-First HTTP Mutation Boundary

Freeze the external mutation surface so non-WORTH callers can issue
authoritative writes through typed server and Query contracts with validation,
policy denial, branch posture, and provenance preserved.

**Relevant subsystems**
- compatibility mutation route family
- mutation request schema validation
- mutation admission and Query lowering
- mutation response/provenance shaping
- idempotency-key admission and replay classification
- mutation precondition enforcement

**Relevant APIs**
- `WORTHServerCompatibilityMutation`
- `WORTHServerCompatibilityMutationRequest`
- `WORTHServerCompatibilityMutationEnvelope`
- `WORTHServerDenialEnvelope`
- `WORTHServerSuccessEnvelope`
- `WORTHServerIdempotencyKey`
- `WORTHServerMutationPrecondition`
- `WORTHServerIdempotentReplayReceipt`

**Relevant Query surfaces**
- `workspace.write_intent(command).review()?.admit()?.execute()`
- `workspace.write_batch_intent(commands).review()?.admit()?.execute()`
- `workspace.inspect(&receipt)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. External mutations must
  consume existing pipeline and handoff proofs instead of introducing an HTTP-
  local mutation-review ladder.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
  and `FoundationalBoundaryEvidenceProvenanceFrontDoor` for mutation result
  envelopes that must stay machine-checkable across clients.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` for mutation
  diagnostics richness posture where admitted.

**Warnings**
- Do not treat HTTP mutation as raw lower-runtime mechanics with nicer JSON.
- Do not collapse schema validation, capability denial, policy denial, and
  execution failure into one generic error surface.
- Do not let transport retries redefine mutation semantics.
- Do not let ambiguous timeout-or-retry scenarios create duplicate writes or
  duplicate side effects because the server lacks canonical idempotency shape.
- Do not admit mutation preconditions as advisory metadata; they must gate
  authoritative write behavior when present.

**Test requirements**
- Add a mutation-parity test proving equivalent direct-facade and
  compatibility mutations compare equal on canonical mutation result,
  provenance, branch, and support artifacts where overlap exists.
- Add a hostile schema-and-policy denial test proving malformed bodies,
  forbidden commands, and unsupported mutation families fail at the narrowest
  expected external boundary with distinct failure artifacts.
- Add a retry-honesty test proving semantically equivalent transport retries do
  not widen or mutate canonical result meaning.
- Add an idempotency-replay test proving repeated mutation requests with the
  same admitted idempotency key and canonical input produce one authoritative
  effect and one replay-classified canonical result family rather than two
  writes.
- Add a mutation-precondition test proving basis or validator mismatch fails
  typed before authority changes and cannot silently degrade into an
  unconditional mutation.

**Engineering decisions**
- Compatibility mutation is an interop projection of canonical server and
  Query mutation meaning, not a second mutation runtime.
- Typed denial and provenance are non-optional for external clients because
  they are part of the semantic contract, not debugging extras.
- Idempotency and mutation preconditions are part of the external authority
  boundary because hostile networks make ambiguous retries ordinary, not rare.

**Open questions**
- None.

### Phase 4: Streaming Response And Large Export Boundary

Freeze the streamed-response surface so large reads, exports, and initial
hydration paths can emit incrementally without redefining canonical read
meaning or forcing dishonest full buffering.

**Relevant subsystems**
- streaming response route family
- large export projection
- buffered-versus-streamed response selection
- streaming backpressure and pacing posture
- disconnect and cancellation classification
- background export fallback admission

**Relevant APIs**
- `WORTHServerStreamingResponse`
- `WORTHServerStreamingChunk`
- `WORTHServerCompatibilityExport`
- `WORTHServerResponseEnvelope`
- `WORTHServerStreamSelection`
- `WORTHServerStreamCancellationReceipt`
- `WORTHServerBackgroundExportRequest`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.inspect(...)`
- admitted export-oriented Query-facing read surfaces
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Streaming is a transport
  projection over already-admitted read meaning, not a new proof ladder.
- Use `worth-foundational::facade::FoundationalCounterBackedPerformanceReceipt`
  for streamed-response performance receipts.
- Use `worth-foundational::facade::FoundationalPerformanceCounterSpec` and
  `FoundationalPerformanceBundle` for exact chunking, buffering, and pacing
  counter contracts.

**Warnings**
- Do not treat streamed export as a second export semantics.
- Do not buffer whole payloads behind a streaming API unless the route admits
  that cost explicitly.
- Do not let chunk boundaries become semantic boundaries.
- Do not let client disconnect or cancellation blur the line between committed
  canonical result, partial transfer, and abandoned transport.
- Do not force export surfaces that are too large for honest request-lifetime
  streaming into fake synchronous success.

**Test requirements**
- Add a streamed-versus-buffered parity test proving equivalent reads or
  exports compare equal on canonical response, basis, provenance, and support
  artifacts.
- Add a hostile buffering-honesty test proving large streamed routes do not
  silently materialize full payloads before emission when the contract claims
  incremental delivery.
- Add a chunk-perturbation test proving different legal chunk boundaries do
  not alter canonical semantic artifacts.
- Add a disconnect-honesty test proving client disconnect, downstream backpressure,
  and explicit cancellation localize exactly what semantic work completed,
  what transport work aborted, and which counters remained zero.
- Add an async-export admission test proving too-large-or-too-slow export
  requests can transition into a typed background export posture instead of
  hanging inside a dishonest synchronous HTTP lane.

**Engineering decisions**
- Streaming is a delivery mechanic selected after canonical Query-facing
  meaning is fixed.
- Large export handling belongs in the compatibility surface because external
  clients need transport-level streaming without semantic drift.
- Cancellation and background export fallback are explicit external contracts,
  not accidental side effects of dropped connections or server timeouts.

**Open questions**
- None.

### Phase 5: Multipart Upload Admission And Early Rejection Boundary

Freeze the multipart admission boundary so external clients can declare upload
intent, metadata shape, and bulk-body expectations through one typed ingress
contract that can still reject hostile or unsupported requests before
meaningful body transfer begins.

**Relevant subsystems**
- multipart upload route family
- upload part parsing and validation
- structured upload metadata admission
- early upload admission and `100-continue` handling

**Relevant APIs**
- `WORTHServerMultipartUpload`
- `WORTHServerUploadPart`
- `WORTHServerUploadManifest`
- `WORTHServerCompatibilityMutationEnvelope`
- `WORTHServerUploadExpectation`

**Relevant Query surfaces**
- `workspace.write_intent(...)`
- `workspace.write_batch_intent(...)`
- `workspace.inspect(&receipt)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Multipart admission must
  consume the existing admission and mutation-handoff boundaries instead of
  defining upload-only proof progression.
- Use no new `worth-foundational` surfaces in this phase beyond the mutation
  result and diagnostics artifacts already admitted in earlier phases.

**Warnings**
- Do not fuse raw bytes and metadata truth into one anonymous request payload.
- Do not require upload endpoints to bypass the compatibility request-contract
  family because multipart is mechanically different.
- Do not admit upload metadata without the same policy and branch posture as
  other authoritative writes.
- Do not wait until gigabytes of body data arrive before rejecting requests
  that auth, policy, manifest, or content negotiation could have denied early.

**Test requirements**
- Add a multipart-admission test proving structured upload metadata and the
  associated canonical mutation/result artifacts remain stable under varied
  legal part ordering and boundary placement.
- Add a hostile multipart-rejection test proving malformed part graphs, missing
  metadata, oversized parts, and unsupported content shapes fail typed before
  metadata truth commits.
- Add a blob-versus-metadata separation test proving upload byte ingestion does
  not appear as structured truth payloads in compatibility response artifacts.
- Add an early-rejection test proving `Expect: 100-continue` and equivalent
  early-admission posture reject unauthorized or malformed uploads before bulk
  body transfer begins.

**Engineering decisions**
- Multipart is one ingress transport, not a special semantics lane.
- Upload metadata truth is authoritative through the same mutation boundary as
  other writes; blob bytes remain transport data handled by the server.

**Open questions**
- None.

### Phase 6: Staged Upload Lifecycle, Chunked Ingress, And Upload Integrity Boundary

Freeze the bulk-ingress lifecycle so staged upload state, unknown-length
transfer posture, compression bounds, cleanup, and integrity verification stay
server-owned and evidence-bearing instead of becoming transport folklore.

**Relevant subsystems**
- staged blob ingress
- staged upload cleanup and expiry
- chunked-body and compression-bounded ingress
- upload integrity verification

**Relevant APIs**
- `WORTHServerBinaryIngressSession`
- `WORTHServerUploadCleanupReceipt`
- `WORTHServerIngressIntegrityDigest`
- `WORTHServerUploadExpectation`

**Relevant Query surfaces**
- `workspace.write_intent(...)`
- `workspace.write_batch_intent(...)`
- `workspace.inspect(&receipt)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Ingress lifecycle must
  consume the already-admitted upload request and mutation boundaries.
- Use `worth-foundational::facade::FoundationalCounterBackedPerformanceReceipt`
  for ingress lifecycle and cleanup receipts.
- Use `worth-foundational::facade::FoundationalPerformanceCounterSpec` and
  `FoundationalPerformanceBundle` for exact ingress pacing, cleanup, and
  decompression-bound counter contracts.

**Warnings**
- Do not let staged upload state survive interruption, expiry, or tenant/branch
  mismatch without explicit cleanup and evidence.
- Do not treat chunked transfer, unknown length, or compressed upload bodies as
  free-form transport trivia; they need explicit bounds and denial surfaces.
- Do not let digest verification degrade into best-effort logging after bytes
  already became semantically usable.

**Test requirements**
- Add a staged-cleanup test proving interrupted multipart sessions, expired
  uploads, tenant/branch-mismatched resumptions, and abandoned staged blobs
  produce explicit cleanup artifacts with exact zero authoritative-truth drift.
- Add a chunked-and-compressed ingress hostility test proving unknown-length,
  drip-fed, and compressed upload bodies respect exact size, pacing, and
  decompression-ratio limits with typed denial rather than resource fog.
- Add an upload-integrity test proving content digests and manifest digests are
  preserved exactly and mismatches fail before metadata truth commits.

**Engineering decisions**
- Staged upload lifecycle is server-owned state and must be explicitly cleaned
  up under interruption or expiry instead of relying on process folklore.
- Integrity verification belongs to the ingress boundary because external
  callers need byte-level trust before metadata truth can honestly finalize.

**Open questions**
- None.

### Phase 7: Binary Egress And Range Semantics Boundary

Freeze the ordinary binary egress surface so full downloads and range-shaped
downloads remain policy-safe, mechanically explicit, and separate from
structured truth delivery.

**Relevant subsystems**
- binary download route family
- range request parsing
- binary pacing and backpressure posture
- conditional range admission

**Relevant APIs**
- `WORTHServerBinaryDownload`
- `WORTHServerRangeRequest`
- `WORTHServerBinaryEgressSession`
- `WORTHServerStreamingResponse`
- `WORTHServerConditionalRangeRequest`

**Relevant Query surfaces**
- metadata lookup and policy-bearing read surfaces admitted through
  `workspace.read(...)` or `workspace.inspect(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Binary egress must consume
  the existing request, policy, and support posture artifacts.
- Use `worth-foundational::facade::FoundationalCounterBackedPerformanceReceipt`
  for binary egress counter receipts.
- Use `worth-foundational::facade::FoundationalPerformanceCounterSpec` and
  `FoundationalPerformanceBundle` for exact transfer, range, and zero-
  forbidden-fallback counter contracts.

**Warnings**
- Do not route binary egress through structured delivery classes or sync
  transport lanes.
- Do not let range semantics widen authorization or metadata visibility.
- Do not reduce range support to one happy-path single-range case while leaving
  multi-range, suffix-range, open-ended range, and `If-Range` ambiguity
  unspecified.

**Test requirements**
- Add a range-parity test proving equivalent full-download and legal
  range-assembled downloads compare equal on canonical metadata, provenance,
  and authorization artifacts.
- Add a hostile range-authorization test proving overlapping, out-of-bounds,
  and unauthorized range requests fail typed at the narrowest expected egress
  boundary.
- Add a range-shape hostility test proving multi-range, suffix-range,
  overlapping-range, open-ended-range, and `If-Range` requests either preserve
  one canonical egress policy or fail typed at the exact range boundary.

**Engineering decisions**
- Binary egress is a server transport responsibility layered over canonical
  metadata truth and policy.
- Range handling is an explicit binary-lane contract, not a hidden client
  convenience.

**Open questions**
- None.

### Phase 8: Resumable Download Posture And Download Integrity Boundary

Freeze the retry/resume and integrity boundary for binary egress so external
callers can distinguish ordinary full/ranged transfer from admitted retry or
resume posture and can verify byte correctness independently of transport
success.

**Relevant subsystems**
- resumable egress negotiation
- binary integrity projection
- retry and partial-transfer classification

**Relevant APIs**
- `WORTHServerBinaryResumeRequest`
- `WORTHServerBinaryIntegrityDigest`
- `WORTHServerBinaryEgressSession`
- `WORTHServerConditionalRangeRequest`

**Relevant Query surfaces**
- metadata lookup and policy-bearing read surfaces admitted through
  `workspace.read(...)` or `workspace.inspect(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase.
- Use `worth-foundational::facade::FoundationalCounterBackedPerformanceReceipt`
  for retry/resume and integrity receipts.
- Use `worth-foundational::facade::FoundationalPerformanceCounterSpec` and
  `FoundationalPerformanceBundle` for exact retry, restart-denial, digest, and
  zero-forbidden-fallback counter contracts.

**Warnings**
- Do not describe resumable download as durable restart-stable resume unless a
  durable contract really exists for that surface.
- Do not let download integrity live only in transport success codes; clients
  need explicit digest-bearing verification surfaces.
- Do not let partial transfer retry silently widen into a different canonical
  file artifact or policy story.

**Test requirements**
- Add a resumable-download honesty test proving admitted binary retry/resume
  posture stays distinct from durable restart-stable delivery claims.
- Add a binary-integrity test proving full, range-assembled, and retried
  downloads preserve explicit content digests and fail typed on digest or
  validator mismatch.
- Add a retry-boundary test proving interrupted egress and resumed egress
  compare exactly where claimed and fail typed where restart-grade guarantees
  are not admitted.

**Engineering decisions**
- Resume posture and integrity are separate from ordinary range semantics
  because they add a new proof family over the same binary lane.
- External callers need explicit retry and digest contracts, not implicit
  transport optimism.

**Open questions**
- None.

### Phase 9: File Metadata Truth Linkage Boundary

Freeze the explicit linkage between file metadata truth and raw blob transfer
so external clients can reason about what truth changed, what bytes moved, and
what policy authorized each step without conflating them.

**Relevant subsystems**
- file metadata truth linkage
- binary authorization and policy evaluation
- metadata-to-transfer provenance projection
- branch-aware file truth access

**Relevant APIs**
- `WORTHServerFileMetadataReceipt`
- `WORTHServerBinaryPolicyDecision`
- `WORTHServerFileTransferProvenance`
- `WORTHServerCompatibilityFileEnvelope`
- `WORTHServerExternalBasisRequest`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.inspect(...)`
- `workspace.write_intent(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Metadata truth linkage must
  stay a projection over already-proven request, policy, and mutation/read
  boundaries.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
  and `FoundationalBoundaryEvidenceProvenanceFrontDoor` for metadata-to-
  transfer evidence projection.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` for external
  visibility policy on metadata and transfer diagnostics.

**Warnings**
- Do not let raw blob presence imply metadata truth was committed.
- Do not let metadata truth updates imply that blob bytes flowed through the
  structured response body.
- Do not allow branch, tenant, or policy posture to diverge between metadata
  truth access and binary transfer authorization.

**Test requirements**
- Add a metadata-linkage parity test proving file metadata reads, writes, and
  associated transfer surfaces preserve one canonical metadata identity and one
  canonical provenance story across upload, download, and ordinary read lanes.
- Add a hostile truth-transfer divergence test proving interrupted blob motion,
  denied authorization, or malformed metadata cannot create partial truth/file
  linkage fog.
- Add a branch-and-policy linkage test proving metadata truth and binary policy
  decisions stay aligned under branch variation and diagnostics-richness
  variation.

**Engineering decisions**
- Metadata truth is authoritative; blob transfer is derived transport.
- External clients need both stories explicitly because regulated and ordinary
  operational consumers must be able to classify truth motion versus byte
  motion.

**Open questions**
- None.

### Phase 10: Filename Normalization And Intermediary Cache-Safety Boundary

Freeze the external portability and cache-safety boundary so filenames,
metadata keys, and intermediary reuse posture cannot drift into multiple
external identities or cross-scope leaks.

**Relevant subsystems**
- filename and metadata normalization
- external cacheability and intermediary-safety policy
- canonical external file identity shaping

**Relevant APIs**
- `WORTHServerCanonicalFilename`
- `WORTHServerMetadataNormalizationReceipt`
- `WORTHServerCacheabilityPolicy`
- `WORTHServerCompatibilityFileEnvelope`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.inspect(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` for visibility
  policy on normalization and cacheability artifacts.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
  and `FoundationalBoundaryEvidenceProvenanceFrontDoor` when normalization and
  cacheability decisions need boundary-visible evidence.

**Warnings**
- Do not let filename casing, Unicode normalization, path-like input, control
  characters, or metadata-key ambiguity produce multiple canonical identities
  for one file truth surface.
- Do not let intermediary caches or CDN layers infer public safety for file
  metadata or transfer responses that remain tenant-, branch-, or auth-scoped.

**Test requirements**
- Add a normalization-hostility test proving filename normalization,
  path-traversal-like names, duplicate canonical names, and metadata-key
  ambiguity collapse into one canonical metadata identity or fail typed before
  truth linkage.
- Add an intermediary-cache-safety test proving metadata and transfer surfaces
  emit exact cacheability policy that prevents cross-tenant, cross-branch, or
  remask-unsafe reuse by external caches.
- Add a canonical-file-identity test proving legal name and metadata encoding
  variation still lowers to one stable external file identity where equivalence
  is claimed.

**Engineering decisions**
- Canonical filename and metadata normalization are separate from truth linkage
  because portability and intermediary reuse are their own proof family.
- Cacheability posture is part of the external compatibility contract, not an
  infrastructure afterthought.

**Open questions**
- None.

### Phase 11: External Surface Counters, Diagnostics, And Operator Evidence Boundary

Freeze the external-surface evidence layer so compatibility, streaming, upload,
and download behavior can be reconstructed through typed counters and operator
artifacts rather than free-form route logs.

**Relevant subsystems**
- compatibility and binary counters
- external operator evidence records
- diagnostics richness policy for external surfaces
- certification bundle emission

**Relevant APIs**
- `WORTHServerExternalCounterSet`
- `WORTHServerBinaryCounterSet`
- `WORTHServerExternalEvidenceRecord`
- `WORTHServerCompatibilityCertificationBundle`
- `WORTHServerBinaryCertificationBundle`

**Relevant Query surfaces**
- None newly introduced. This phase certifies and exposes evidence for already
  admitted Query-backed external behavior.

**Shared crate usage**
- Use `worth-foundational::facade::FoundationalCounterBackedPerformanceReceipt`
  for compatibility and binary performance receipts.
- Use `worth-foundational::facade::FoundationalPerformanceCounterSpec` and
  `FoundationalPerformanceBundle` for exact external-surface counter contracts.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceAttachmentBundle`
  and `FoundationalBoundaryEvidenceSupportFrontDoor` for operator-facing
  evidence records that must survive outside the request call stack.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` for evidence
  richness policy.
- Use no new `worth-proof` surfaces in this phase.

**Warnings**
- Do not make binary counters a subset of broad HTTP counters only.
- Do not rely on free-form logs as the primary proof of parity, denial, or
  buffering honesty.
- Do not hide zero-valued forbidden counters; exact zero assertions are part of
  the contract.

**Test requirements**
- Add a counter-honesty test proving compatibility, streaming, upload, and
  download surfaces emit exact narrow counters, including exact zero assertions
  for forbidden buffering, forbidden sync-lane usage, and forbidden fallback.
- Add an operator-reconstruction test proving an external admitted or denied
  operation can be classified from canonical evidence artifacts alone without
  host-log archaeology.
- Add a diagnostics-richness invariance test proving reduced richness trims
  detail without changing support, policy, provenance, or counter truth.

**Engineering decisions**
- External evidence and counters are part of the product contract, not a later
  observability cleanup.
- Binary and structured external counters remain separate so transfer cost does
  not disappear into generic request accounting.

**Open questions**
- None.

### Phase 12: Abuse Budgets And Transfer Lifecycle Accounting Boundary

Freeze the external control-plane accounting boundary so pacing hostility,
slowloris/drip-feed behavior, byte-class budgets, retries, disconnects,
cancellations, and cleanup are all independently explainable with exact
artifacts instead of broad operational fog.

**Relevant subsystems**
- abuse-shaping and pacing-budget evidence
- disconnect/retry/cleanup accounting
- per-route-family and per-byte-class budget enforcement

**Relevant APIs**
- `WORTHServerAbuseBudgetReceipt`
- `WORTHServerTransferCleanupEvidence`
- `WORTHServerExternalCounterSet`
- `WORTHServerBinaryCounterSet`

**Relevant Query surfaces**
- None newly introduced. This phase closes control-plane accounting over the
  already-admitted external surface.

**Shared crate usage**
- Use `worth-foundational::facade::FoundationalCounterBackedPerformanceReceipt`
  for abuse-budget and lifecycle accounting receipts.
- Use `worth-foundational::facade::FoundationalPerformanceCounterSpec` and
  `FoundationalPerformanceBundle` for exact pacing, cutoff, retry, cleanup, and
  zero-hidden-fallback counter contracts.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceAttachmentBundle`
  and `FoundationalBoundaryEvidenceSupportFrontDoor` for retained lifecycle
  evidence.
- Use no new `worth-proof` surfaces in this phase.

**Warnings**
- Do not collapse cheap structured routes and expensive blob routes into one
  vague rate-limit story with no byte-class or route-family accountability.
- Do not hide slowloris, drip-feed, disconnect, retry, or staged-cleanup
  behavior inside aggregate error counts.
- Do not let transfer cutoff behavior become policy folklore with no retained
  reason artifact.

**Test requirements**
- Add an abuse-budget test proving per-route-family, per-byte-class,
  per-tenant, or equivalent external budget posture is exact, typed, and
  independently explainable for structured versus binary lanes.
- Add a transfer-lifecycle accounting test proving disconnects, retries,
  cancellations, expiries, and staged cleanup emit exact narrow counters and
  evidence artifacts instead of disappearing into broad failure aggregates.
- Add a slowloris-cutoff test proving hostile pacing and drip-feed behavior
  crosses explicit budget boundaries with exact denial and exact zero semantic
  truth drift.

**Engineering decisions**
- Abuse shaping is a control-plane contract, not just a metric family.
- Lifecycle accounting is separate from generic operator evidence because it
  explains why work was cut off, retried, or cleaned up under hostility.

**Open questions**
- None.

### Phase 13: Hostile Compatibility And Blob Certification Closure

Close Milestone 3 with certification that proves the merged external surface is
one honest interop boundary rather than a pile of routes that happen to work in
happy-path demos.

**Relevant subsystems**
- compatibility certification harness
- streamed-versus-buffered parity harness
- multipart and range hostility harness
- binary/structured separation certification

**Relevant APIs**
- canonical external certification bundles
- `surface_contract_digest`
- `declaration_digest`
- `response_digest`
- `basis_digest`
- `support_posture_digest`
- `provenance_digest`
- `failure_digest`
- `counter_snapshot`
- `audit_evidence_digest`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.write_intent(...)`
- `workspace.write_batch_intent(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use `worth-foundational::facade::FoundationalBoundaryArtifactCompileFailBoundary`,
  `FoundationalBoundaryEvidenceCompileFailBoundary`, and
  `FoundationalPerformanceCompileFailBoundary` to classify compile-fail and
  forbidden-surface certification boundaries.
- Use `worth-foundational::facade::FoundationalBoundaryArtifactProductionTestReadyArtifact`,
  `FoundationalBoundaryEvidenceProductionTestReadyArtifact`, and
  `FoundationalPerformanceProductionTestReadyArtifact` if the milestone emits
  persistent certification artifacts beyond local test bundles.
- Use no `worth-proof` surfaces unless the certification harness itself becomes
  a proof-bearing transition family.

**Warnings**
- Do not close the milestone on broad response equality.
- Do not certify request/response parity without also certifying streaming,
  multipart, range, and blob/truth separation hostility.
- Do not omit exact zero assertions for forbidden sync-lane bleed, forbidden
  fallback, or forbidden buffering residue.
- Do not certify happy-path idempotency or happy-path uploads while leaving
  ambiguous retries, disconnects, precondition mismatch, duplicate headers,
  and proxy normalization drift untested.

**Test requirements**
- Add one cross-surface parity matrix varying direct facade versus
  compatibility HTTP, buffered versus streamed delivery, branch/basis posture,
  diagnostics richness, and transport retry timing while asserting exact narrow
  canonical digests rather than one flattened payload equality check.
- Add one multipart and range miserable-path matrix varying malformed part
  graphs, interrupted upload/download, unauthorized ranges, and legal chunk or
  part perturbations while asserting exact failure localization and exact zero
  forbidden counters.
- Add one blob/truth separation certification lane proving metadata truth,
  structured read/mutation artifacts, and raw binary transport stay distinct
  under simultaneous upload/download and structured truth churn.
- Add one operator-evidence certification lane proving external compatibility
  and binary operations remain reconstructable from canonical evidence bundles
  alone.
- Add one external retry-and-precondition matrix combining idempotency keys,
  ambiguous timeouts, conditional reads, mutation preconditions, and retried
  uploads/downloads while asserting exact replay, denial, and zero-duplicate-
  authority artifacts.
- Add one edge-normalization matrix combining duplicate headers, repeated query
  keys, forwarded-host ambiguity, content negotiation variance, browser
  preflight, filename normalization hostility, and intermediary cache pressure
  while asserting exact canonical request, policy, cacheability, and failure
  digests.
- Add one transfer-hostility matrix combining slowloris pacing, unknown-length
  chunked ingress, compressed upload hostility, disconnect churn, staged-upload
  expiry, multi-range download variation, and integrity mismatch while
  asserting exact counters, cleanup evidence, and zero forbidden truth drift.

**Engineering decisions**
- Milestone 3 closes only when compatibility HTTP and blob transport are
  parity-safe, denial-honest, and structurally separate from sync truth
  delivery.
- The hostile proof bar is narrow-artifact certification with exact counters,
  not endpoint plausibility.
- External API correctness includes adversarial network and intermediary
  behavior, not just well-behaved clients.

**Open questions**
- None.

## Must Ship

- one typed external compatibility surface root for request/response,
  streaming, upload, and download lanes
- Query-first HTTP read, state, inspection, and mutation surfaces with typed
  branch, basis, support, denial, and provenance posture
- explicit request canonicalization, representation negotiation, API versioning,
  conditional read/mutation semantics, and idempotency-key replay posture
- streamed-response handling for large reads, exports, and hydration paths
- multipart upload admission and staged binary ingress
- early upload rejection, chunked/compressed ingress bounds, staged-upload
  cleanup, and upload integrity verification
- range transfer, admitted resumable binary egress, complex range-shape
  handling, and download integrity verification
- explicit linkage between file metadata truth and binary transfer policy
- filename and metadata normalization plus intermediary cache-safety posture
- separate compatibility and binary counters plus operator evidence artifacts
- explicit abuse-budget and transfer-lifecycle accounting
- hostile certification proving parity, denial, buffering honesty, and
  blob/truth separation

## Must Preserve

- Query remains the semantic authority for read, inspection, state, and
  mutation meaning
- compatibility HTTP remains an interop lane rather than a second semantic
  runtime
- streaming changes transport mechanics only
- blob bytes never become sync-truth payloads
- metadata truth remains authoritative and separately inspectable from raw byte
  movement
- ambiguous retries never create duplicate authority effects
- preconditions, validators, and cacheability posture remain explicit rather
  than best-effort HTTP folklore
- staged upload/download lifecycle remains cleanup-safe and evidence-bearing
- runtime-backed versus durable-later posture remains explicit wherever visible

## Acceptance Evidence

- direct-facade and compatibility overlap lanes compare equal on canonical
  declaration, response, basis, support, provenance, and mutation artifacts
  where they should
- streamed and buffered lanes compare equal on canonical meaning while exact
  counters prove buffering honesty
- multipart, range, and binary retry hostility localize failure to the
  expected narrow boundary with exact zero forbidden fallback and forbidden
  sync-lane counters
- idempotent retries, conditional requests, and precondition mismatches
  preserve exact replay or exact typed denial artifacts with zero duplicate
  authority effects
- metadata truth and raw blob transport remain distinct yet linked through
  canonical provenance and policy artifacts
- cleanup, integrity, normalization, and intermediary-cache hostility remain
  reconstructable through exact counters and evidence artifacts
- operator evidence bundles reconstruct admission, denial, and transfer posture
  without host-log archaeology

## Sequencing Notes

This milestone belongs immediately after Milestone 2 because the server now has
one honest direct-consumption surface and must next close the external interop
surface without letting compatibility HTTP become the product's second brain.

It deliberately absorbs the earlier standalone blob milestone because request/
response compatibility and binary transfer share the same external trust
boundary. Splitting them would invite duplicate auth/policy pipelines,
duplicate evidence models, and fake separation between metadata truth and file
transport that the code would have to undo later.

It belongs before lease/sync work because external request/response, streaming,
upload, and download do not require a full server-owned subscription runtime to
be honest. It belongs well before integration/webhook work because those later
surfaces should build on a finished external compatibility and binary boundary
rather than defining it piecemeal.
