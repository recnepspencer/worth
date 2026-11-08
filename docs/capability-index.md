# Capability Index

Find the API for an application task, follow its usage guide, and compare a
worked example. Each entry identifies the facade that owns the operation and
the inputs, results or retained values needed to use it.

An application imports two crates only:

| Facade | Role | Shorthand on this page |
|---|---|---|
| `worth-query-decl` | Declare meaning: schema, operations, queries, capabilities, programs, workflows | `decl::` = `worth_query_decl::facade::` |
| `worth-query-host` | Install and run: requests, mutations, outputs, branches, workflows, recovery | `host::` = `worth_query_host::facade::` |

`host::declaration` re-exports the declaration modules, so a host crate can
reach `host::declaration::application_program::*` without a second import. Some
declaration macros (`worth_query_mutation_binding!`, `worth_query_capability!`,
`worth_query_policy!`) are on `decl::` only.

---

<a id="before-you-conclude-a-capability-is-missing"></a>

## Find a capability

Use these entry points to find an API and its worked use.

1. **Open the facade's API documentation.** Generate local rustdoc for
   declarations and runtime calls:

   ```bash
   cargo doc --manifest-path workspaces/worth-query/Cargo.toml --no-deps -p worth-query-decl -p worth-query-host
   ```

   Browse the [Decl facade](../workspaces/worth-query/crates/worth-query-decl/src/facade.rs)
   and [Host facade](../workspaces/worth-query/crates/worth-query-host/src/facade.rs)
   to find the module that exports a name.
2. **Look up the task** in the [task index](#task-index) below.
3. **Search for a worked use** in the runnable examples and the reference
   application, which both use only the two facades:

   ```bash
   grep -rn "<name or concept>" workspaces/worth-query/crates/worth-query-certification/examples
   grep -rn "<name or concept>" workspaces/worth-query-bank-world
   ```

4. **Check the [known gaps](#known-gaps)** for the task's support posture.
5. **[Report a gap](#reporting-a-gap)** with the task, the facade names you
   checked and the specific behavior you need.

---

## Task index

Choose [native representations](native-representations.md) before a durable
value binding: flat scalar fields, named entities and actual typed relations.
The [candidate-only one-of pattern](native-representations.md#one-class-one-optional-aspect)
includes required class payloads, complete rule scope and atomic transitions;
its lawful at-pin refusal requires the repin. The
[source-subgraph contract and court](source-subgraphs.md) cover partial member
edit, pinned previous state, recorded undo and a generated-member witness.

For a new installed feature, use the
[complete feature definition](build-an-application.md#complete-feature-definitions)
first: `host::worth_query_application_contribution! { pub feature ... }` projects
schema members, action meaning, typed runtime configuration, required roles and
delivery roots. Supply actual handlers and domain configuration through the
generated runtime constructor. Use lower-level builders for specialized composition.
For tracked numerical work, use the
[managed/partitioned computation lifecycle](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation):
declare semantic membership, gather tracked inputs, compute with the original
checkpoint and complete on the owning operation. Canonical identities and
association preserve results across admitted placements; tracked changes refresh
affected partitions while unchanged partitions retain their results.
For application execution, compose
[one process execution service](build-an-application.md#one-process-execution-service):
retain an `Arc<host::runtime::ExecutionAuthority>` above applications and sessions,
pass clones through installation's `with_execution_authority`, and select
`ExecutionRequestPolicy::for_workers` with Serial as the ordinary default.
Choose Automatic placement for independent compute partitions when workload
measurements support it. Propagate typed cancellation, deadline, allocation and
domain failures from the original request.
For submission and recovery, retain the
[original owning action](build-an-application.md#53-execute-through-the-program).

Application installation and reopen require one
`host::application_installation::ApplicationBuildIdentity`, embedded by the
application build pipeline. Query binds it to every producer, including
`INPUT_REUSE=None`. Repeated launches of one artifact retain its embedded identity;
a new build causes derived output to regenerate while preserving authored source
and references.
See [build identity](build-an-application.md#application-build-identity).

Server protocol projections preserve the catalog's exact result schema identity
and numeric version. Generated TypeScript/Python decoders reject a successful
envelope when either differs; they validate protocol correspondence, not product
payload validity or native operation authority. Use `WorthServer::product_protocol_catalog()`
and `worth-server-client-generation` rather than a consumer-maintained route or
version table.

External-input capture and original mutation description are distinct custody.
For resumable provider-revision admission, use
`WorthQueryCapturedExternalInput::admit_preserving(&provider)` and retain its
returned `(original_capture, provider_denial)` on refusal. Retry that original
capture with the provider. The one-shot `admit` discards capture on refusal.
Pass the admitted capture into the authorized action. See
[external input on an action](build-an-application.md#32-attach-actions-to-features).

Signing provenance and supersession parts use the descriptive
`application_schema::CanonicalDigestId`, available through `worth-query-decl`.
Construct it from the actual 32 digest bytes; binary codecs emit a fixed byte
string and human-readable codecs emit 64 hexadecimal digits. Retired digest
integer arrays refuse decoding. The value grants no signing or runtime authority.

Program validation admits finite canonicalization work through
`ApplicationProgramDefinition::CANONICAL_REVISION_BUDGET` and the program facade's
`CanonicalDigestWorkBudget`. The default covers 4096 entries and 1 MiB; override
the declaration's finite ceiling for a larger program. Handle a validation
refusal before installation. See
[program validation](build-an-application.md#35-validate).

Installed graph-obligation identities separately admit 4,096 canonical entries
and 1 MiB. Distinct aspect contracts count once; references, projection masks,
field paths, bindings and structural scopes also count. Application runtime
resource profiles do not raise this ceiling. Operation installation denials
retain requested and maximum counts in `canonical_denial()`. See
[installation limits](build-an-application.md#4-install-or-reopen).

For mutations, prepare the typed decision through `DecisionReader`, then consume
its values and native targets through `CandidateWriter`. Propagate preparation
or publication denials; a successful commit publishes the validated candidate
atomically. An existing-field write without its exact decision observation
returns typed `UnobservedWrite`; its `unobserved_write()` cause identifies the
full entity occurrence and installed field locator without field contents.
Early writer refusals remain handler failures; defensive commit lowering
retains `EffectLowering` instead of a generic proposal-binding rejection.
See [prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects).

Committed-storage read failures retain `StorageUnavailable` with their subject
and cause through Query's execution denials. The Host `primary_graph` facade
exports `RelationalStorageUnavailable`, `RelationalStorageSubject`, and
`RelationalStorageUnavailableCause`. Preserve that refusal when handling a
provider-session failure; an unavailable record is not an absent record.

Native platform owners read exact snapshot kind/aspect and retirement metadata
through request-aware `read_*` operations. Post-publication optional proof
sealing uses `pin_resident_entity_metadata`: it requires already accepted Head
custody and this runtime's aspect-name meanings, performs no source read, and
returns a typed cold-pin refusal. `RelationalPinnedEntityMetadata` exposes current
kind, aspect revision and retirement only; historical reads require Full custody.

Native conditional observation owners borrow the genuine computed effect through
`BridgeConditionalCausalObservation::computed_evaluation()` and its observed
before/after Signal versions. The original Signal verdict, suppression reason
and comparison survive; passive decisions have no computed report. Scheduling
and readiness evidence describes those installed contracts and supplies no
application output-equivalence association. See [read-only Native observation custody](../crates/worth-runtime-bridge/README.md#read-only-native-observation-custody).
`compare_exact_output` supplies a fixed baseline/candidate computation in a fresh
private Signal partition; its paired observations do not independently establish
Native field provenance or application permission.
Producer bindings can declare the bounded field scope with
`output_equivalence()` returning
`WorthQueryProducerOutputEquivalence::exact_u64(Field::reference())`.
The required standard u64 field belongs to the binding's exactly-one Preserve
role. Declaration and installed Native/value contracts seal its meaning into the
producer edition. Keep the original settled-output query observation and authored
source receipt, then use a fresh independently admitted `evaluated_by` request to
inspect the owner-issued Changed or Equivalent decision for that bounded role.
See [performed producer output evaluation](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#performed-producer-output-evaluation).

Native generation payload inspection uses request-aware `read_latest_generation`,
`read_published_generation_for_commit`, `read_published_generation_for_observation`
and `read_generations_for_version`; a deferred index image is never reported as
an absent generation. The no-load `generation_residency_for_observation` observer
returns `DerivedIndexGenerationResidency::{DeferredImage, Absent, Published}`.
That cache status carries no payload or read authority and does not warm an image.

Applied Native aspect payloads retain their original-request reservation in the
actual entity/relation extra shared by current rows, historical metadata and
private mutation views. Replacement admits its own output; snapshot holders keep
their original owners. Successful field lowering and readmission also retain
the original request's bill with the actual private patch until consumption;
checked clone bounds exclude source spare capacity. Request-driven historical
metadata production also reserves each existing metadata allocation before
history edits; snapshots and moved values retain that fixed bill. Controlled
column/map nodes, fixed pages and scalar-directory writes independently retain
their actual backing bills; inline scalars add no fictitious value allocation.
Partial construction keeps admitted nodes until workspace destruction, while
escaped payloads keep only their own bills. Ordinary image/maintenance copies,
legacy adjacency/bitset backing, revision-map and independently constructed
Digest/default payloads remain open. Existing diagnostic/error allocation and
remaining replay/canonical-copy paths remain separate. These are specific backing
contracts with localized footprint costs, not whole replay or Complete funding.
See [applied aspect custody](../crates/worth-relational/README.md#retain-applied-aspect-payloads)
and [controlled node backing](../crates/worth-relational/README.md#retain-controlled-node-and-page-backing).

Native storage statistics, partition statistics, chunk summaries and chunk
diagnostics take the caller's execution request and return a fallible result.
They explicitly demand the whole mirror and retain that resident edition for
the computation; a cold-storage refusal must remain a typed read failure.

Platform owners can borrow the existing targets of a validated native mutation
with `ValidatedMutationFootprint::records()`. Iterate its borrowed records to
inspect each target, preserving multiplicity and the footprint's field and scope
checks.

Platform runtime owners use native partitions with `run_retained_run` or
`run_reduce_retained_native` to retain values accepted by canonical settlement.
Retain the returned owner for as long as its immutable values are needed. Reuse
checks a fresh request's controls; the caller qualifies input equality and
currentness. `accounted_cost()` is optional: `None` means cost was not measured.
For explicit payload and cost contracts, use lower-level methods
including `run_retained`, `run_retained_inputs` and
`run_reduce_retained_holding`.
Their genuine receipts carry original checkpoint prefixes; a value that lacks
the required cost qualification must execute again. See
[retained native results](../crates/worth-execution/README.md#retained-native-results).

Platform byte producers use `ExecutionRequestArrayBuilder<u8>::seal_immutable_bytes`
to retain immutable output with both payload and actual shared control storage
admitted through the original request. The byte owner keeps those admissions
through its final clone; its payload charge remains separately observable.
`ExecutionImmutableBytes::is_external_backing()` distinguishes external custody
from allocated fixed backing; it proves no admission or execution permission. See
[execution backing admission](../crates/worth-execution/README.md).

Platform native ordered-map owners move non-`Clone` payloads through
`ExecutionOrderedMapCell::prepare_owned_native_for_request`, then consume the
actual detached cell with `insert_owned_vacant_prepared` on their unique map.
`remove_owned_retaining_unique` preserves the actual retired allocation for
payload disposal outside the owner gate. Payload heap admission remains with
its concrete owner. See [execution backing admission](../crates/worth-execution/README.md).
Platform owners of exact contiguous request storage use
`ExecutionRequestArrayBuilder::allocate(original_request, count)`. Push initializes
its prefix; `replace_initialized(index, value)` moves within that prefix without
new allocation or custody. Both consult the original request. Seal consumes the
builder into the same immutable `ExecutionArray`; a stopped request cannot seal.
See [fixed payload custody](../crates/worth-execution/README.md).

Platform pre-effect entries use `ExecutionRequest::prepare_native` to prepare
dependent values under one supplied request: standalone entry establishes its
existing SDK scope once, while active same-lineage carriage reuses the actual
meter and its remaining allowance. Propagate both typed execution and domain
failures before publication. This port does not retain an already performed
effect through interruption. See
[Execution request entry](../crates/worth-execution/README.md).

Platform effectful owners use `ExecutionRequest::run_native_completion` to keep
their original business result and custody through subsequent maintenance on
the same native scope. Returned callback values survive final or nested stops;
pre-callback admission and interruption still refuse. This retains actual parent
accounting and latched stops without granting new allowance or imposing
accounting on worker-only requests. The physical report observes execution;
returning custody does not prove commitment. Accounted effectful owners use
`ExecutionRequest::run_completion(ceiling, operation)` for the same guarded
custody contract and the existing aggregate `ExecutionReport`. It reuses one
accounted scope, projects the actual descendant before admission, and retains
the original declared ceiling and remaining allowance. Neither completion port
invokes an effectful callback through an existing own or untaken joined stop.
See [Execution request entry](../crates/worth-execution/README.md).

Platform-owned ordered retained metadata uses `ExecutionOrderedMap<K, V>`:
shared-root cloning and canonical iteration preserve original custody.
Retain the returned root while reading or applying later edits. `accounted_bytes()`
returns the available byte measurement; handle `None` as unmeasured.
`rank_of`/`entry_at` use actual structural counts rather than prefix scans.
Private maps with `Copy` keys and values can prepare an
`ExecutionOrderedMapCell` with the original request, then use unique insertion,
replacement and retaining removal without allocating map backing. Actual touched
owners must be unique; alias refusal preserves the input. Removed cells retain
their original owners and grants for disposal outside caller locks. Ordinary
`prepare_in` remains explicit and creates no request.
Deliberately accounted reduction shape-cost inspection accepts removals/insertions through
`checked_full_build_cost_after_edits`, reusing native edit traversal without
combining values.
See [funded ordered storage](../crates/worth-execution/README.md#funded-ordered-storage).

For a platform owner's keyless map, `run_retained_run` issues an opaque
`ExecutionRetainedRun` after real native completion. `prior.advance_native`
accepts changed values and canonical removed identities only; native settlement
keeps qualified unchanged ranges. Qualify unchanged inputs before calling advance;
propagate stopped-prefix or missing-qualification refusals. Accounted reuse follows
the original receipt at its work boundary. See
[persistent native runs](../crates/worth-execution/README.md#persistent-native-runs).

Platform scalar owners carry complete canonical atomic domain values using
`ScalarAspectType::InlineBytes` and the immutable `InlineBytes` carrier. This
serves pure observations that receive the whole scalar without Native resolver
authority. The actual generic certification consumers are InlineCounterBinding
(the conditional counter), RatioBinding and the deliberately broken binary
write-admission binding; see the [complete consumer and route inventory](inline-atomic-content-boundary.md).
Native content owns ordered numerical data and splines at every size through
ContentRef, including admission, integrity, retention and reconstruction.
References occupy format-bound fields of a flat Struct aspect; there is no
whole-aspect content shape or standalone content-contract builder. Size
never chooses between these meanings; frames above 16 MiB refuse at admission
before effects. Inline bytes are not an alternative dataset or blob path.
Its native framing ceiling is
4,294,967,290 payload bytes; oversized lengths return `InlineBytesLengthDenial`
before admission. The carrier supplies no domain comparison or index hooks.
See [inline opaque values](build-an-application.md#inline-opaque-values).

Applications adopt a complete canonical atomic value through the existing
value-binding macro's `binary: OwnerDelegate` arm. Its Query-free pure owner
keeps normalization, canonical bytes, admission and terminal rendering; the entry
delegate supplies finite bounds, identity, unit and frame. Binary and explicitly
marked canonical domain atoms require the same per-binding source guard and have
distinct portable protocols from established structured canonical artifacts.
Ordinary domain field writes must pass the installed owner's admitted
reader and canonical byte comparison under the original request before provider
lowering or publication; an unreadable encoding returns a typed commit refusal.
See the [compiled entry delegate](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/domain_values/binding.rs)
and [representation and guard guidance](build-an-application.md#inline-opaque-values).

For independently edited persisted structure, use the
[native representation table](native-representations.md). For an intentional identity-free, reference-free canonical atom or
an explicit document boundary, use
`decl::worth_query_value_binding!`'s `scalar: String, canonical,` arm with
`decl::application_schema::WorthValue` derivation (or the pure owner's existing
value). Record the owner, representation policy, reason and reference guard.
The entire atom is one native field: its nested grammar grants no inner-field
read, patch, touch or invalidation. See the
[Store grain contract](../plans/worth-store/storage-foundation-aspect-native-gate.md#current-value-grain-contract).
The installed value-binding catalog retains the derived
shape and any explicit domain representation policy. Fixed arrays, checked
indices, `Box`, exact
pairs and tagged named variants use the checked seed path. Complete closure
admission checks every child, including unselected variants. Keep these pure
descriptions in the schema owner. Projection reads use the installed codec and
original request controls. Propagate decoder refusals: an ignored failure vetoes
completion.
Opaque owners supply admitted seeds.
Use an explicit field `worth_value(delegate = ...)` for a real domain-owned
constructor. For sealed original-text documents, `native_envelope: Envelope`
delegates the pure owner's `CanonicalDocumentCodec` readers and exact failures.
For derived readiness, use `WorthQueryOutputReadinessContractBuilder::for_field`
and typed `field_dependency` entries, then `build_typed`. The generated contract
is sealed against the actual installed field and complete aspect catalogs before
configuration. Its `delivery_ordinal` distinguishes delivery selection from
eligibility and uses the actual canonical node order. See
[canonical values and derived readiness](build-an-application.md#canonical-nested-values-and-derived-readiness).
Continuation and live delivery retain decode custody through their native final
handoff. For cumulative reads of a retained invariant snapshot, use
`snapshot.read_fields(&execution, |read| ...)`; its scoped `read.field(...)`
calls borrow the original pin and refuse the entire output after an ignored
decode failure or late cancellation.
The installed `value_meaning()` roster checks executable and native field
contracts; native canonical codec digests carry the same emitted meaning.
Use the retained catalogue's `same_installation_as(&other)` to compare its actual
immutable backing. Clones match; independent installation or restore does not,
even with equal field meaning. This grants no request, snapshot or settlement authority.
Producer readiness uses the actual output-role carrier and its emitted meaning.
Predecessor comparison requires explicit semantic aspect revision advancement
for changed meaning and grants no recovered-transition permission.
The MemoryOnly interim checkpoint transition supports whole-output derived-codec
retirement when authenticated native
Create/Preserve publication succession still covers every present field.
Complete membership is independent of individual data writes: unchanged
companions and genuinely executed no-data Preserve retain their original
identities, while no-contact reuse retains its performed provenance. Dynamic
families may allocate and retire members through actual native operations;
each member retains its own birth and exact predecessor write positions.
An intervening ordinary write, including equal bytes, revokes continuity.
Only a genuine complete native codec retirement can close that generation for
fresh target creation; ordinary deletion or producer Retire cannot free it.
It preserves authored bytes and identities on
the same carrier and requires the installed target producer to rebuild output.
A closed set of native producer-owned internal relations can retire with its
output records when both endpoints are independently proven in that same
publication. Related ordering can survive when its key codec stays unchanged.
A producer owns every relation it actually allocates/creates touching at least
one output, including new boundary edges during Preserve; external endpoints
remain external entities. Before/candidate/cold association authenticates the
complete relation roster and its birth/write succession. Independently committed
crossing references may survive in either direction without being adopted; exact
native creation/allocation distinguishes them from omitted producer ownership.
Internal and claimed-owned relations require complete native provenance.
Destructive retirement remains closed: external or rewritten relations, protected
or partly classified fields, affected invariants, changed ordering keys and join
consequences refuse before publication; ordinary restore remains exact. Surviving application relations
are selected with the same bounded native reads as entities and rejudged through
`RelationMutationIntent::Revalidate(RevalidateRelationIntent { relation_id })`.
This native read demand schedules installed rules against the actual preimage;
it changes no endpoint, field, record version or lineage and claims no write or
rollback restoration. Rules that overlap retired relations remain refused. Independently produced
surviving output requires complete native original association and exact present
identity/kind/payload plus compatible installed roles. It retains historical
correspondence without readiness facts, through normal capture and the same
funded deferred/acknowledged repair capsule. Every disposition first validates complete original native publication membership,
including present optional outputs. Mixed or incomplete outputs refuse;
only real installed producer publication can establish target currentness.
An existing continuation resumes
through its original runtime and retained index generation, while a new target
request reads only genuinely regenerated target output. See the
[historical query lifecycle](build-an-application.md#protected-continuation-pages).
See [canonical nested values and derived readiness](build-an-application.md#canonical-nested-values-and-derived-readiness)
for supported grammar, native checkpoint examples and the current package
archive refusal. Representation limits do not grant request resources,
currentness or codec-transition permission.

For managed and partitioned computation, declare domain values and use the
installed owner's `decision_plan` to prepare, compute and complete them under
the original request. See
[managed computation](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation).

`WorthQueryInvariantEntityIdentity` has its own native resident contract for the
shared entity-key allocation. Clones preserve that backing and its original
reader authority; resident description grants no read, currentness or mutation
authority and introduces no process-wide deduplication promise.

Platform storage owners use `ExecutionAllocationPolicy::reserve_memory(bytes)`
before copying when their contract requires admission. Accounted leases return
a reservation to retain with that payload. A worker-only lease checks request
status and returns `None`; the allocation owner still handles layout and allocator
failures. Retain an accounted reservation until the payload is released. See
[Execution payload custody](../crates/worth-execution/README.md)
for the ownership and refusal contract; this grants no graph or publication authority.

Choose a task and follow its guide for the exact calls and result handling.

| I want toâ€¦ | Use (facade path / call) | Guide |
|---|---|---|
| Declare and install a component set | Build `ApplicationComponentSetDefinition::new(set_ref).coupled_by(relation_ref)` and add it with `schema.components(definition)`. Different coupling markers share one declared member type; completed schemas and portable readmission normalize the roster and validate its actual homogeneous relations. Results use genuine Native persistent identities. Choose privileged inspection, scoped operation, ordinary invariant or declared Query reads below. | [Component declarations](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#component-set-declarations), [public topology example](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#public-topology-island-consumer) |
| Declare finite projected member meaning | Use `ApplicationProjectedMemberDomainRef`, `ApplicationProjectedMemberCaseRef` and `ApplicationProjectedMemberDomainDefinition`, then `schema.projected_member_domain(definition)`. This declares portable carrier/case meaning with fresh carrier closure; it creates no stored entity or live member. Genuine views bind observations to the actual registered Native domain and selected carrier generation. | [Projected member domain meaning](../workspaces/worth-query/crates/worth-query-decl/README.md#projected-member-domain-meaning) |
| Define components over carrier/case members | Use `ApplicationComponentSetRef::projected` and `schema.derived_components(ApplicationDerivedComponentSetDefinition::new(set, source))` with a homogeneous declared derived source. Case-restricted carrier lifts, composition, union and intersection express adjacency; self-edge exclusion compares complete logical keys. Domain context must come from actual source incidences. | [Finite projected sources and composition](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md#finite-projected-sources-and-composition) |
| Read components and enforce counts in an ordinary invariant | Resolve `typed_component_set(set_ref)` in the invariant factory, declaring its member entity and every physical source relation in both read closure and applicability. Proposed/committed views expose `component_of(&binding, &member)`, `components(&binding)` and `component_count(&binding)` with real Native identities/members. Preparation admits actual members/edges; evaluation cannot widen that footprint. Empty counts still schedule after real member creation/deletion. Complete census work may grow with the exact-view inventory; Of selects only its cohort. | [Ordinary invariant component reads](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#ordinary-invariant-component-reads) |
| Declare an operationâ€™s component visibility paths | `schema.operation_component_read(ApplicationOperationComponentReadDefinition::new(operation_ref, set_ref, path).also_path(other_path))` binds a canonical typed path union to the actual mutation root and installed set. The set requires `OperationReads<Operation>`. Portable descriptions undergo fresh schema closure and installed-spine checks; admitted readers consume only the actual installed scope and their original root, never an atomic relation grant. | [Operation component scopes](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#operation-component-scope-declarations) |
| Inspect a complete persistent component under installation authority | The installation-owned invariant projection reader exposes `component_of(set_ref, &member)` with `identity()`, `members()` and `len()`, and `component_lineage(set_ref, &identity)`. It selects the installed Native definition, admits every member and reads every coupling source in both directions at the same snapshot. This privileged inspection records no publication decision dependencies; ordinary scoped publication reads use the separate admitted-operation entrance below. | [Component inspection](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#privileged-component-inspection) |
| Inspect a logical component under installation authority | `entity_member(&entity)`, `derived_member_relations_from(relation, &member)` and `graph_component_of(set, relation, &member)` consume the exact installed Native domain/source at the selected stored snapshot. Results retain complete membership, genuine aggregate identity and original source/allocation custody. Members cannot cross projection authorities or snapshots, become entity mutation targets, or supply tracked operation facts. `component_lineage(set, identity)` reads actual stored lineage without requiring a live member. | [Privileged logical component inspection](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#privileged-logical-component-inspection) |
| Read and track complete components in an admitted operation | Operation projection and handler decision readers expose `component_of(set_ref, &member)`, `components(set_ref)` and `component_count(set_ref)`. Every returned Native cohort must be wholly visible through installed paths to the exact admitted root. Enumeration/count use the complete forward universe and retain negative/empty evidence; swallowed failures prevent sealing. Source revisions and original Native bodies govern reuse. | [Scoped component reads](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#scoped-component-decision-reads) |
| Declare a complete component query result | `ApplicationQueryComponentRef::of_scope_member` or `::census`, `.component_selection(ref)` and an explicit `.root_path(...)` union preserve one typed metadata slot for identity, membership and count. `.components_by(...)` governs the whole slot. Complete-schema and portable readmission validate its set, member and visibility paths. Installation and archives retain this meaning. One-shot results expose `.components(ref)`, `.len()`, `.iter()`, and each Native identity/member group. Of proves each complete cohort visible to the exact scope root; Census retains the complete forward universe, including empty/negative evidence. Closed slots refuse before Native acquisition, partial cohorts and result-limit truncation refuse, and original source bodies govern reuse. | [Declared component-query contracts](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#declared-component-query-contracts) |
| Declare a concrete operation's access once | `decl::worth_query_operation!(... input Input; access { ... })` generates Rust capabilities and the schema projection; `schema.declared_operation::<Operation>(definition)` installs it with explicit effect/aftermath policy. Declare the complete read/effect/precondition set. | [Operation access](build-an-application.md#21-declare-operation-access-once) |
| Fence a locator to one live Server assembly | Use `WorthServer::instance_identity()`, retain session discovery's `server_instance`, and install `WorthServerInstanceSelectionPolicy::RequiredForProductCalls` where product calls must carry it. Guard and generated catalog share the policy; session IDs share the same instance identity. | [Server instance selection](../crates/worth-server/docs/connect-another-backend.md#fence-a-live-server-instance) |
| Preserve product preflight/currentness refusals over HTTP | Server retains `WorthServerTransportDenial::product_operation_denial()` and projects `ProductOperationDenied` with `worth-server-product-surface-denial.v1` owner codes, basis facts and execution/recovery posture. | [Product-surface refusals](../crates/worth-server/docs/connect-another-backend.md#product-surface-refusals) |
| Attach a native program to Server readiness without extracting its runtime | Transport entry uses `worth-server`'s `WorthServerProductOperationDeclaration::with_program_application(program.clone())` and returns a finished adapter registration. Readiness reuses the authentic installed allocation and grants no operation authority; native requests still authorize execution. | [Connect another backend](../crates/worth-server/docs/connect-another-backend.md) |
| Re-enter an application's genuine operation custody on product retries | Runtime-backed product mutation declarations use `.with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody)`. Every command passes fresh Server admission/readiness and enters the adapter; Server neither caches responses nor issues retry receipts. The adapter must retain actual native custody and freshly authorize its advancement/delivery. Reads always enter fresh authorization, even with a retry key. Eligible default `ServerReceipt` mutations and durable-executor replay keep their existing contracts. | [Application-owned retries](../crates/worth-server/docs/connect-another-backend.md#application-owned-command-retries) |
| Complete an effectful business Back or global ending | Retain the original guarded owner; after real external completion, call `completed.accept_local_action(&fresh_request, workflow)`. The original binding's native admissionâ€”required abilities/policies or its exact installed capability, never Advanceâ€”admits exact route/custody settlement. Inspect returned progress and retain cleanup; exact replay never repeats business effects. Suppressed effects confirm their original joint settlement. | [Local action completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#complete-a-local-action-with-an-external-effect) |
| Attach a fresh record to the admitted creator | `creation.link_to_principal::<Binding, Relation>(relation, key)` binds the exact mutation principal and carries its observed target through publication. Root and peer creations only; fresh schema and typed installation validate the correspondence. | [Independent creation sites](../workspaces/worth-query/crates/worth-query/docs/authoring/independent-creation-sites.md) |
| Atomically create a retained audit beside a disposable selection | `creation.with_peer(entity, key, authoring_closure)` reuses typed initializers; `creation.link_from_path(path, relation, key)` binds a retained existing anchor. Reusing one site does not suppress independent peers. | [Independent creation sites](../workspaces/worth-query/crates/worth-query/docs/authoring/independent-creation-sites.md) |
| Assign or clear an input-selected existing entity reference | `body.replace_relation_from_input(relation, required_readonly_identity, member, key)` admits an original Option member, resolves through the native equality owner, and retains current/selected endpoints into one candidate. Nullable outgoing max-one relations only; same endpoint preserves its edge. | [Existing-reference inputs](../workspaces/worth-query/crates/worth-query/docs/authoring/existing-reference-inputs.md) |
| Replace a disposable current snapshot and retire its predecessor | `creation.replace_and_retire_from_scope(relation, key)` reuses the tracked native predecessor selection for atomic Unlink/Delete/Create/Link. Requires explicit Delete and one-to-one relation cardinality. Ordinary `replace_from_scope` still preserves the old entity. | [Retiring selection replacement](../workspaces/worth-query/crates/worth-query/docs/authoring/retiring-selection-replacement.md) |
| Preserve an unchanged disposable selection's actual identity | `creation.reuse_current_if_unchanged()` compares every declared field, including optional absence, through the native canonical-equivalence owner after input admission. Independent body effects still run. | [Complete-content selection reuse](../workspaces/worth-query/crates/worth-query/docs/authoring/complete-content-selection-reuse.md) |
| Create a record and its required child topology atomically | `creation.with_child(relation, key, authoring_closure)` declares flat prior-parent sites; `child.link_from_parent(relation, key)` adds an association from that same fresh parent. Existing native owners allocate all sites in the original admitted context and link retained created handles. | [Atomic child creation](../workspaces/worth-query/crates/worth-query/docs/authoring/atomic-child-creation.md) |
| Atomically write a computed value to a selected parent | `body.assign_value_at_path(path, required_field, &value)` retains the exact guard-free native path target and its destination-field dependency. Existing checked evaluation feeds the same candidate as scope/audit writes and signed creation. Captured AuditContext remains a preimage; certified Subject loci cannot be rewritten, including through aliases. | [Computed graph-target writes](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-path-writes.md) |
| Write an exact Decimal from captured input | Use input assignments/initializers with a readable `ScalarAspectType::Decimal` field codec and its explicit native input profile; a transparently serialized string is borrowed exactly and validated before effects. Required and optional members are supported. | [Exact Decimal inputs](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#exact-decimal-inputs) |
| Require captured input to match a parent's field | `body.expect_input_from_path(ApplicationQueryRootPath, field, member)` uses native forward/reverse/identity selection and tracked field comparison without changing the row's scope. Optional graph absence cannot match. | [Path-input guards](../workspaces/worth-query/crates/worth-query/docs/authoring/path-input-guards.md) |
| Retire dependent current snapshots when a source input changes | `body.retire_owned_selections_on_input_change(field, member, owners, selection, maximum_owners)` coupled to the exact body input write. Native observation retains every selection and publication unlinks/deletes atomically, preserving holders. | [Owned-selection retirement](../workspaces/worth-query/crates/worth-query/docs/authoring/owned-selection-retirement.md) |
| Require a stored field to equal captured input | `body.expect_input(field, "member")`, ordinary native body installation and owning execution. Required or optional field must be present; the original captured flat scalar is admitted through the field's installed codec. Query retains the comparison fact through publication. | [Captured-input guards](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#compare-a-field-with-captured-input) |
| Keep a business value unique | Schema `unique_field(entity, field)` with an equality-queryable field, then the normal authored body/owning action. Query retains intrinsic indexed dependencies for actual unique writes before signing/sealing; native lowering and publication enforce collisions and concurrent claims. | [Unique business values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#unique-business-values) |
| Conditionally emit the exact operation input | `body.emit_external_input(effect, Some(native_condition))`, the operation's `.external_effect(effect, correlation_family)`, and normal `setup.operation_body::<Binding>()`. Met emits; Unmet/NotApplicable suppresses only that effect. Guard facts remain publication obligations. | [Authored conditional effects](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md) |
| Emit graph data absent from the command or public DTO | `body.emit_external_query_context::<Binding, _, _, _, _, _>(effect, exact_query_definition, condition)`, a payload binding with Value `ApplicationQueryExpressionSnapshot`, `execute_with_expression_context()` and the actual `expect_source`. The query is parameter-free and ExactlyOne. Query funds/prepares once and publishes the ordinary outbox with all native source facts. | [Native query-backed effect payloads](../workspaces/worth-query/crates/worth-query/docs/authoring/conditional-body-effects.md#real-example) |
| Share one computed revision across business, audit and payload | `ApplicationOperationBodyValue::admit`, `body.assign_value`, `creation.initialize_value`, and `emit_external_evaluation` with `ApplicationOperationBodyEvaluationSnapshot`. Native Foundation evaluation prepares once and validates every installed destination codec. Unitless UInt64 results; at least one field sink. Computed scope bookkeeping may accompany signing, with old revision captured as AuditContext; computed signed-artifact initializers remain refused. | [Shared Computed Body Values](../workspaces/worth-query/crates/worth-query/docs/authoring/computed-body-values.md) |
| Index committed Public query rows | Declare an `ApplicationManagedIndexBinding` and exact Source/Probe contracts, install with `setup.managed_index`, then use authenticated `open_index`, explicit `rebuild_index` and ordinary `execute`. Native retains each selected public cache and publishes a cold successor; pending World reads retain their genuine predecessor. Cold selected images return `NotReady`; sibling handles share eviction/close. Source requires Public disclosure and either Public authorization or conventional READ ability. Ability-authorized Sources require a current-only Parent; original Source READ support remains independent through cache use, expectations and correction history. Governed Parent entrances retain independent capability and READ admission. Consume managed row/result-set observations with request-bearing `expect_source`/`expect_result_set`. Complete Source dependencies include nonmatches and empty membership. Handles retain their installed provider version, runtime, branch and full-generation scope. Prefix and closed-box providers use the same Decl/Host contract. Repeated required String reads use `SourceRow::with_text(selector, callback)` to borrow invocation/image-confined decoded text with per-use admission. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Declare newest-first event-chain membership | `ApplicationDerivedRelation::ordered_events(previous, introduces, retires)` accepts typed stored Event->Event and Event->Row references. The named recipe survives canonical identity, portable archive and actual Native registry installation. Native proves the physical chain and folds retirement before introduction; malformed chains retain a typed refusal. Native pinned acquisition preserves narrowing caller stops and failed-read work/read receipts. Named recipes can enter physical query root paths and maintained Sources through `forward_derived`/`reverse_derived`; private CAD adoption and its scale evidence remain unfinished. | [Derived relation meaning](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md#derived-relation-meaning), [typed recipe evidence](../plans/WORTH-query/milestone-9.19-phase6-query-ordered-event-evidence.md), [controlled acquisition evidence](../plans/WORTH-query/milestone-9.19-phase6-controlled-derived-read-evidence.md) |
| Select physical roots through a named derived source | `ApplicationQueryRootPath::from(scope).forward_derived(source)` and `reverse_derived(source)` preserve the Derived namespace, registered Native handle and complete positive/negative support. Compose ordinary query paths with physical hops and guards. Maintained Sources retain their existing guard/predicate restrictions. Ordinary queries, maintained Sources, watches and Source expectations consume the same acquisition. Equal rendered rows cannot erase changed physical history. Projected source domains and Derived closure steps refuse at this entrance. | [Derived query root paths](build-an-application.md#derived-query-root-paths), [root-path evidence](../plans/WORTH-query/milestone-9.19-phase6-derived-root-path-evidence.md) |
| Qualify an index's complete Source structure | Declare `ApplicationManagedIndexSourceStructure::BinarySourceTree` and implement the provider's `decompose` callback. Query checks actual node/Source bijection and connected acyclic topology before retaining the root, then carries it through local warm edits. Opaque providers remain conservative. This does not qualify candidate completeness, pruning, refined influence or live predicate routing. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Execute complete exhaustive provider membership | Declare `ApplicationManagedIndexMembership::ExhaustiveSourceTree` with `BinarySourceTree`; implement `validate_probe` and the pure `matches_source` predicate. Query visits every checked Source region, selects/emits genuine rows and refuses interrupted partial traversal. v1/v2 retain candidate-only membership. This explicit full traversal keeps complete Source dependencies and provides no pruning, locality, exact-contact or narrowed-footprint authority. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Prune checked managed Source subtrees | Declare `CheckedSummarySourceTree` with `BinarySourceTree` and provide the pure `Summary` algebra. Query recomputes Source/child summaries bottom-up, checks the actual cached Node value and propagates Native Source changes through all ancestors before root retention. Summary row lookup is confined to the actual region. Query owns visit/skip and genuine-row matching; mismatch or interrupted work refuses. Prefix extrema and closed-box unions use the same contract. Complete conservative Source dependencies remain; this supplies no exact-contact, retained predicate-routing or narrowed-footprint authority. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Retain checked managed traversal coverage | Exhaustive and checked-summary strategies record the actual visited cut. Query checks root, parent/child coverage, Source dispositions and the matched-slot bijection before its receivers release slots. The private executable issuer and actual borrowed Source/root, probe and read-session correspondence bind consumption. Public Source observations retain the verified frontier; Native committed/proposed correctness reads require complete evidence. Rebuild shape stays outside semantic Source equality. The supported Public maintained-watch profile adds inverse routing; explicit row-local exact refinement uses separate complete evidence; finer field influence remains unfinished. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Page a governed ordered child collection | Declare an ExactlyOne root with one ordered Many continuation collection and pinned support. `page_governed` / `resume_governed` (or `_approved`) freshly admit current capability and independent READ before all page metadata. Hidden ordering inputs need Ordering/Pagination influence and permitted hidden outputs stay omitted. Move-only tokens retain the exact old World/parameters/Native ordering generation; denial consumes the token. Requested width cannot widen the binding maximum. Managed root selection plus continuation remains a typed declaration refusal. | [Protected continuation pages](build-an-application.md#protected-continuation-pages) |
| Watch a complete maintained Parent | Implement the pure `WorthQueryManagedIndexPredicateProvider` on the forward provider; configure `setup.managed_index_with_predicate_routing`, declare Public Parent Live eligibility, then call `watch_index::<Index>()`. Initial full read and registration join the actual World image. Each fresh `next_watch` request admits the complete original Parent before metadata; Delivered carries separate notification B and current full read/receipt C, acknowledging only B. Query owns inverse routing, exact close and bounded queue custody. Projected root filters use conservative candidate routing; separately declared exact refinement uses the same row evaluator for each actual record mode. Unsupported/untracked, candidate-only and continuation profiles refuse. Whole-Source correctness dependencies remain. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Refine complete managed candidates exactly | Declare a separately versioned row-local `REFINEMENT`, implement `WorthQueryManagedIndexRefinementProvider`, and install the same provider through `managed_index_provider(...).with_refinement().install()`. Explicit Parent `managed_index_refined` and Native `refined_managed_index` preserve candidate access as a distinct meaning. Query verifies the full candidate cut, evaluates every genuine candidate once, and retains all decisions plus the accepted subset before release. Empty candidate sets still validate the probe; domain/controlled refusal cannot become empty success. Exact Parents require full Source/Parent projection correspondence. Governed exact inputs require membership influence even without Live. Qualified complete Native-maintained Parents may retain the verified candidate influence described below with actual passive audience correspondence; unsupported profiles remain conservative. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Watch a governed maintained Parent | Use `watch_index_governed` / `next_watch_governed` (or `_approved`) with the actual capability, operation and original purpose input. Every call freshly admits capability and independent Parent READ before metadata or ACK. Query retains exact passive request correspondence without an old grant/decision; revoke/regrant can retry the same queued notification. All projected provider inputs require explicit membership and Live influence, combined with the exact Source/Parent profile. Permitted hidden values remain omitted through normal disclosure projection. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Preserve qualified managed Source expectations across irrelevant edits | Complete Native-maintained Parents with the installed full projection profile and actual passive audience correspondence retain every actual candidate body, including exact rejects, plus complete negative coverage and exact decisions. The same installed forward provider verifies only the target cut; shared complete bodies provide the fast path and nonshared bodies require paid exact occurrence/dependency/full one-hop projection comparison. Commitments only shortlist equality; restored content never proves authored-history cancellation. Independent ordinary field/adjacency revision facts can still refuse populated observations. Ordinary source/result-set expectations pin the checked target through Native commit preflight. Typed provider/refusal/request causes survive comparison. Qualified governed captures retain full passive use/disclosure without old READ; unsupported profiles remain whole-Source. Qualified semantic identity and actual output reuse cover unchanged complete candidate inputs; qualified canonical input cutoff is documented below; qualified Public prefix and indirect box/contact correction uses actual canonical history, historical predicate relevance, genuine performed cancellation and strict paired support. | [Currentness evidence](../plans/WORTH-query/milestone-9.19-phase4-currentness-evidence.md) |
| Reuse outputs of a qualified managed Source | In the supported Public producer pipeline, full candidate content, including exact rejects, supplies descriptive semantic identity. Actual complete capture/cell/body comparison and retained output verification decide reuse under fresh READ/disclosure. Reused committed output retains its original receipt with zero producer contacts; successful regeneration still requires the declared lifecycle. Exact-current retained one-shot reads keep read-only custody; moved historical images refuse. Typed managed Source read errors remain available on output demand/availability denials. | [Semantic reuse plan](../plans/WORTH-query/milestone-9.19-phase4-semantic-reuse-plan.md) |
| Avoid a managed producer handler for equal canonical input | Qualified complete Public captures retain fresh invocation custody and compare complete candidate IDs/decisions plus outer field/relation membership and exact query affinity. The existing installed-input, decision-context, handler-prefix, consumed-output and Native witness gates decide StableReused. Conservative stable rows carry authentic output facts and retain full verification, with no clean equality links or downstream discharge. Changed inputs, generations, incomplete/restored or unqualified captures do not qualify. | [Selected-membership plan](../plans/WORTH-query/milestone-9.19-phase4-membership-plan.md) |
| Retain governed managed influence without retaining READ | The actual authorization/disclosure owner checks session, installed capability, query/parameters and principal/scope, then retains paid passive full capability-use and admitted disclosure meaning under the exact installed Parent. Tagged canonical Source material separates different audiences; full passive bodies remain compared. Hidden/rejected candidate inputs stay influence despite omitted DTO fields. Fresh public query/watch/disclosure still admits current capability and independent READ; descriptive mutation expectations do not replay a Query grant. Governed producer/output-demand readmission and canonical cutoff remain unqualified; their selected execution mode still refuses governance. | [Governed influence plan](../plans/WORTH-query/milestone-9.19-phase4-governed-plan.md) |
| Check managed membership qualification and limits | Both providers have complete coverage, exact-refinement, protected metadata and maintained-watch courts. Genuine empty-read/entrant interference refuses stale registration; equal-value commits preserve rows without promising notification suppression; completed recovery replay preserves World/provider/notice state. The Phase4 currentness row above qualifies complete maintained Parents with the supported projection/predicate profile and actual Public or governed passive audience. | [Phase 3 evidence and supported contracts](../plans/WORTH-query/milestone-9.19-phase3-closure.md) |
| Locate a checked predecessor Source node | In a BinarySourceTree warm edit, `nodes.before_source_node(slot)` uses the existing qualified Source correspondence and original request to locate that occurrence's node. Ordinary staged `get` still checks liveness; read its old key before editing the occurrence. Cold/opaque and missing/foreign slots refuse. This adds no Source projection, second provider locator, READ or publication authority. | [Construct committed provider indexes](build-an-application.md#construct-committed-provider-indexes) |
| Prepare structural candidate Source changes in an installed invariant | Resolve the typed derived relation binding; `planner.prepare_derived_relation_changes(&binding)` retains Native-issued paired Source custody in the rule Scope. `context.derived_relation_changes(&binding, &prepared)` reborrows only at that exact proposal/version/definition and current audience. Structural before/after pullback retains empty support. Declare the complete read and affected kinds; this structural loan does not admit scalar row reads or index-root publication. | [Candidate Source custody](build-an-application.md#prepare-candidate-structural-source-changes) |
| Maintain a provider Source across public and candidate reads | Declare the invariant's `managed_index_source` and cold policy; install the same provider and implement `update` over real before/after occurrences. Native prepares every affected scope before validation, including unopened scopes. Use `DecisionReader::index_query` for a freshly admitted committed read and `retain_managed_index`/`context.managed_index` for exact Native candidate observations. Committed opens attach maintained roots without a cold build. Whole-Source dependencies include nonmatches and empty membership; source availability and READ remain independent. Unsupported Source shapes refuse installation. | [Maintained invariant Sources](build-an-application.md#maintain-a-provider-source-for-an-invariant) |
| Project nested maintained Source rows | Declare matching finite Source/Parent relation forests with existing typed nested shape builders. Native preserves forward/reverse physical occurrences and complete descendant/empty dependencies; candidate influence, expectations and watches consume the full declared tree under original controls. Child predicates and OptionalAspect Source fields remain unsupported. No new Source READ, historical policy or CAD authority follows. | [Nested Source profile](build-an-application.md#construct-committed-provider-indexes) |
| Retain correlated native query values for expressions | `request.query(intent).execute_with_expression_context()`; inspect `observed_source.expression_context()`. Exactly-one one-shot result; values stay attached to recursive dependencies before any domain projector. The original row or `result_set_observation()` supplies them through `expect_source` or `expect_result_set`; these source contracts remain distinct. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md) |
| Inspect nested snapshot data in a pure payload codec | `ApplicationQueryExpressionSnapshot::field_path(&["owner", "name"])` follows actual nominal record types and schema field order, unwraps intermediate optional-one records, and preserves a final option. `ExpressionValue::record_fields()` distinguishes records from lists/entries. Lists are never implicitly indexed. | [Native expression contexts](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#inspection-and-debugging) |
| Require correlated native query information | `operation.requirement_operands().query_context::<Binding, _, _, _, _>(name, query_definition)` consumes the exact parameter-free, exactly-one query declaration. Supply its sealed source with `expect_source` to guidance and execution; native facts remain publication obligations. | [Authored context requirements](../workspaces/worth-query/crates/worth-query/docs/authoring/native-query-expression-contexts.md#real-example) |
| Require values across a related collection | `operation.requirement_operands().outgoing_field_values(name, relation, child_field)` binds a tracked `List<Value>` or `List<Option<Value>>`; use the existing Foundation expression for length and all/any. Same requirement drives guidance and mutation admission, with membership/liveness/field publication dependencies | [Related-item values](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#require-values-on-every-related-item) |
| Return the scope, one selected outgoing subject or a created row from an authored body | `decl::application_schema`: `body.preserve_scope_output::<Binding, Role>()`, `body.preserve_related_output::<Binding, Role>(relation)` (Preserve/ExactlyOne), `creation.create_output::<Binding, Role>()`; read the real receipt's `outputs_of::<Contract>()?.entity::<Role>()?`. Related selection is tracked once and retained through publication; replay preserves original identity. | [Scope-bound operation bodies](../workspaces/worth-query/crates/worth-query/docs/authoring/scope-field-operation-bodies.md#stable-entry-points) |
| Supply original typed inputs for native workflow needs | `WorthQueryWorkflowOperationInputs::new(program_runtime)`, then `.with::<InstalledAction, _>(description)` for a binding-wide entry or `.with_at::<InstalledAction, _>(&qualified_input_node, description)` for an exact node, and `select(workflow, &required)`. Distinct qualified nodes may retain the same binding; ambiguous overlap is refused. Native needs select order and existing owners retain all custody. Source-completion actions only. Accept an opaque completion with `prepared_advance.accept_selected_operation(completed_selection)`; the occupied native owner retains fresh admission, exact receipt checks and cleanup on every refusal. This roster does not supply heterogeneous proposal target inputs: ordinary `OperationInput` edges still require the same portable type and exact source-proposal input identity. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) |
| Declare features and a program | `decl::application_program`: `ApplicationFeatureSpec::root::<S, F>()â€¦finish()`, `impl ApplicationProgramDefinition`, `ApplicationProgramAuthoring::<S, P>::begin().validated_program()` | [Build Â§3](build-an-application.md#3-declare-features-and-the-program) |
| Select complete feature schema, runtime, contracts, spec and output projections once | Host `worth_query_application_contribution! { pub feature Definition<S: Binding> for Feature => Runtime { ... } }`; typed `action name(Binding, Delivery, HandlerType)` and `required_pair name(Roles, InitialHandler, PreserveHandler)` members drive every projection. `ApplicationFeatureDefinition` composes specs/schema/actual roots; `WorthQueryFeatureDefinition` composes explicit configuration and existing installation. The existing required pair and role runtimes preserve native lifecycle ownership. Repeated identical delivery roots deduplicate before flattening; distinct-root connection conflicts still refuse. | [Complete feature definitions and required roles](build-an-application.md#required-initial-and-preserve-output-roles), runnable `topology_entry/examples/required_feature.rs` |
| Declare distinct captured workflow target inputs | `graph.captured_operation_input(&proposal, &target)` and the component equivalent declare exact control/guarded correspondence. `prepare_workflow_proposal_with_inputs(workflow, instance, &inputs)` publishes complete original target contracts through native proposal/replay/first-effect owners. Archive profile 8 carries the new edge; unchanged definitions keep profile 7. Recorded-actor admission and recovery retain the original proposal correspondence and require fresh grants. In-memory owning execution, not a durable whole-command runner. | [Workflow operation inputs](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-operation-inputs.md) |
| Install or reopen an application Home | `host::WorthQueryApplicationHomeRequest::new(..)` supplies the location, validated program, roster, schema, configuration, build UUID, limits and initializer; `open_in_advancement(&phase)` returns the owning Home. Borrow `home.runtime()` to serve. A changed recorded program/schema requires a transition; inspect `refused.transition_required()` for the typed recorded and requested bindings. | [Build §4](build-an-application.md#4-install-or-reopen) |
| Edit a Directory Home's program or schema | Supply the actual old recipe with `successor.with_program_transition(predecessor, resources, author)`. The bounded read-only author returns the complete `WorthQueryHomeProgramTransitionPlan`; the Host records its intent, publishes positioned relational steps and acknowledges the binding before service. Fresh/current Homes skip the author; a completed transition closes and reopens with the successor alone. Inspect the owning refusal's `phase()`. Interrupted-transition recovery remains cleanup work. | [Directory development workflow](build-an-application.md#development-workflow-for-directory-homes), [Host transitions](../workspaces/worth-query/crates/worth-query-host/README.md#development-workflow-with-program-transitions) |
| Identify a rejected operation declaration dependency | `decl::application_schema::ApplicationSchemaDeclarationDenial` preserves `MissingOperationProgramDependency`, `MissingOperationDecisionReadDependency` and `MissingOperationMutationPreconditionDependency` with owned `operation` and `ApplicationOperationDependencyLocus`. Target loci carry the original typed field/relation/precondition; whole-body failures remain explicitly `Body`. | [Declaration failures](build-an-application.md#45-declaration-failures) |
| Configure invalidation retention | `host::runtime::WorthQueryInvalidationResources::new()` takes no policy. The subscription retains every delivered position, so no reader falls outside a window or reconstructs to catch up. | [Build: invalidation retention](build-an-application.md#invalidation-retention) |
| Reclaim retired physical storage | `runtime.try_reclaim_retired_storage(&phase)` borrows the same application's advancement and original request controls. The installed retention policy governs reclamation; typed stops preserve actual completed progress. Save the result outside the advancement callback to retain it across a final request refusal. | [Storage retention](../workspaces/worth-query/crates/worth-query/docs/execution/storage-retention.md) |
| Inspect retained owner footprints for certification | Import `WorthQueryCertificationCostRuntimeExt` from `worth-query-replay::facade`; call `runtime.observe_certification_retained_storage(&phase)` outside warm timing. It unions actual current/retired root allocations and reports resident catalog owner footprint separately. Busy, cold or unlinked owners return refusal. Catalog nested conventions and excluded transient/index/allocator storage prevent treating the result as heap or peak usage. | [Scope and usage](../crates/worth-relational/API_OVERVIEW.md#retained-owner-footprints) |
| Diagnose Native settlement-admission work | Capture `RelationalMvccCostScope` through Relational's inspection facade before the operation, then call `runtime.observe_mvcc_counters(&scope)`. Read `canonical_settlement_route_visits()` as actual runtime-wide candidate examinations, including other branches; retained-root maintenance is separate. | [Cost observation](../crates/worth-relational/API_OVERVIEW.md#inspect-what-happened) |
| Capture original operation meaning under native resources | `idempotency_owned(key)` returns a funded description or `(original_builder, owned_key, WorthQueryOperationInputCaptureDenial)`. Retry capture with those returned owners, or consume them into `into_capture_refusal` before the initiating scope ends and use `retry_capture` with fresh original-branch authority; success alone moves original input, conditions, source and branch. One shared typed payload charge follows native input custody until its last owner releases it; nested caller input heaps and allocator metadata follow their existing contracts | [Capture complete original meaning](build-an-application.md#capture-complete-original-meaning) |
| Compose one complete action | `ApplicationFeatureSpec::root::<S, F>()` or `at::<S, I, F>()`, then `.action::<Binding, Delivery>()`, optional `.locality_and_change::<Scope, Shape>()`, `.correspondence::<C>()`, `.evaluated_requirement::<Rule>()`, `.external_input::<Provider>()`, and `.finish_action()` before feature completion. Each clause is once-only; delivery is explicit. `ApplicationActionReference<S, I, F, B, D>` selects complete actual installed meaning without manufacturing it. | [Build Â§3.2](build-an-application.md#32-attach-actions-to-features), [Feature Capsule Authoring](../workspaces/worth-query/crates/worth-query/docs/authoring/feature-capsule-authoring.md) |
| Store fixed or discovered work beyond construction | `WorthQueryFixedOperation<S, Intent, Program, Root, Action>` and `WorthQueryDiscoveredOperation<S, Intent, Program, Root, Action>` retain the authentic runtime allocation and complete installed action. Keep an independent genuine runtime handle for fresh requests on `owner.branch()`. All refusals return actual custody; runtime close refuses `RuntimeKeepaliveRetained` while operation keepalive remains. | [Owning actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#retained-runtime-and-configuration-custody) |
| Observe a required output without driving its producer | Candidate `DecisionReader::output_availability(&native_producer, typed_demand)` and checked `WorthQueryComputationReader` equivalent retain the fresh source Query and native completion dependency in the same attempt; Current or positively qualified Pending, with typed refusal otherwise. The retained owner supplies native source/completion evidence and original-image retry; this route does not promise Fresh readiness after checkpoint restore | [Source-aligned availability](../workspaces/worth-query/crates/worth-query/docs/capabilities/source-aligned-output-availability.md), compiling `topology_entry/.../output_availability/collection.rs` |
| Run a mutation | Declare an exact `ApplicationActionReference` and delivery; `request.mutate(intent)â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. No-output actions use SourceOnly; fixed output actions own their required set. | [Build Â§5.3](build-an-application.md#53-execute-through-the-program) |
| Observe an effect-bearing released original | Explicit idempotency resolution returns `ReleasedOriginal`, preserving the original key, intent, Native meaning and release provenance. Ordinary retries refuse with `OriginalEffectReleased` before handler work; changed intent reports drift. The observation grants no receipt, adoption or delivery authority. | [Recovered custody](build-an-application.md#current-durable-home-composition-boundary) |
| Read the original receipt on an idempotent retry | Match `RecoveredCommitted` or use `recovered_receipt()`. The sealed original owner artifacts supply all recorded fields; the receipt cannot grant live output/publication authority or reconstruct the handler result. Keep typed unpublished/unresolved custody for recovery. An in-process original producer recovery keeps genuine delivery and settlement custody through Bridge/readiness; recovered settlement exposes output roles and provenance without a live `application_commit_receipt()`. | [Directory Home and retries](../workspaces/worth-query/crates/worth-query-host/README.md) |
| Bind keyed raw Server mutations to an installed Query application | Install `WorthServerQueryApplicationMutationBinding::for_program` in Query handoff configuration. Its selector returns genuine principal, scope, Query-issued branch and typed intent. Server binds the admitted stable scoped key and intent into Query's co-committed idempotency row, including concurrent/restarted retries. Missing application binding or unsupported legacy preconditions refuse before effects. | [Server application binding](../crates/worth-server/src/surfaces/compat_http/mutation_execution/application_handoff.rs) |
| Select history from an original recorded receipt | `request.at_original_receipt(&phase, &receipt, maximum_history_work)` admits the exact original through the current owner history; `at_recorded_commit` accepts its durable commit coordinate. Both require fresh authority and preserve typed history refusal. | [Directory Home and retries](../workspaces/worth-query/crates/worth-query-host/README.md) |
| Configure application compute placement | `host::runtime::ExecutionRequestPolicy::for_workers(posture, determinism, workers)` selects placement, determinism and width; start with Serial. Pass the policy through application resources. Platform stages use `ExecutionRequest::run_native`, `run_native_with_checkpoint` and `KernelContext::poll`; scoped visitors use `KernelContext::work_loan` to charge the original account. Propagate typed request and allocation failures. | [One process execution service](build-an-application.md#one-process-execution-service), [Original-scoped work](../crates/worth-execution/README.md#original-scoped-work) |
| Fund automatic Directory Home upkeep after application advancement | Pass `host::WorthQueryApplicationHomeCheckpointLimits::new(entries, payload_bytes).with_maintenance_allowance(work, memory_bytes, timeout)` through the Home request's `with_checkpoint_limits`. Defaults are 1 Gi work units per advancement, 256 MiB shared retained/scratch memory and 60 seconds. The Host owns this allowance; exhaustion leaves maintenance due or pending and preserves the business result. Explicit `checkpoint` and `recover_checkpoint` use their invoking request. | [Checkpoint limits](../workspaces/worth-store/crates/worth-store-application/src/checkpoint/limits.rs), [Host checkpoint calls](../workspaces/worth-query/crates/worth-query-host/src/application_home/checkpoint.rs) |
| Restore a captured interim application checkpoint | `host::application_installation::WorthQueryInterimCheckpointRestore` is the single interim door. Keep the capture-issued `WorthQueryApplicationCheckpoint`, or transport its bytes together with its opaque `capture_origin()` and rebind with `with_capture_origin`. The Native image and World-original registry are authenticated by the actual capture owners. Format14 refuses predecessors and byte-only operational restore. This interim mechanism is distinct from filesystem Home reopen. | [Interim restore](../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/program/checkpoint.rs) |
| Share workers across applications and checkpoint reopen | Construct `ExecutionAuthority::try_construct(ExecutionAuthorityConfig::for_workers(workers))` once at process startup, retain its `Arc` above sessions, and pass clones through installation's `with_execution_authority`. A final handle drop does not authorize constructing another physical authority in that process. | [Shared service and runnable example](build-an-application.md#one-process-execution-service) |
| Configure explicit request-memory accounting | `ExecutionBudget::new(workers, memory, work)` and `SerialMemoryBudget::new(memory)` accept optional memory ceilings. `Some(bytes)` selects a finite ceiling. Retain each reservation with its payload and handle finite-capacity or counter-overflow denials. | [Explicit request-memory policy](build-an-application.md#optional-request-memory-policy) |
| Retain a result whose heap size is not known before evaluation | Original `ExecutionRequest::run_measured_result(ceiling, known_bytes, &mut reservation, kernel)` reuses Scope admission and lends `ExecutionResultMemory` on that same reservation for before-growth claims. It measures opaque callback results/errors before framework retention and leaves their actual heap charged until release. This is post-materialization admission unless the source cooperates before allocation; declared Scan/Map maxima remain separate real contracts. | [Request-memory policy](build-an-application.md#optional-request-memory-policy) |
| Check invariant interruption and configure work policy | Call `Context::check_live()` or `ScopePlanner::check_live()` and propagate `CustomInvariantInterruption::{Cancelled, DeadlineElapsed}`. Configure optional domain work through `RelationalInvariantWorkBudget::new(limit)`; supply an explicit accounted execution budget when work measurement is needed. | [Optional invariant work and original interruption](build-an-application.md#optional-invariant-work-and-original-interruption) |
| Return a typed refusal from an installed invariant | Declare the rule's `Refusal` and canonical `RefusalBinding` with a finite wire maximum. Return `Violation(refusal)`; the original commit denial decodes through `decode_invariant_refusal::<Binding>()` and retains exact bytes. Installed binding order determines the first violation; encoding failure and cancellation remain distinct outcomes. | [Typed invariant refusals](build-an-application.md#typed-refusal-from-installed-invariants) |
| Enforce application relation laws | Declare `.acyclic()`, `.symmetric()`, `.unique(scope)`, `.same_partition_endpoints()`, `.reaches_at_least(n, kinds)` or `.orbit()` on the registered `ApplicationRelationIntegrity`; native commit validation judges proposed edges, physical multiplicity and distinct reachable targets, including creations and removals. The stored orbit law requires homogeneous endpoints and exactly one predecessor and successor per live member. It also enables private committed aggregate identity and lineage under one Native owner; ordinary orbit/lineage inspections, tracked orbit decisions and complete declared orbit queries are available; scoped and tracked lineage reads are unavailable. Generic Native branch reconciliation retains its separate typed limitation. A native failure that blocks the current execution point prevents custom invariant preparation; administrative admission still comes first. Inspect original typed law fields through the commit denial's `relation_law()` view. | [Relation laws](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Read a derived endpoint set in a rule or decision | Declare `ApplicationDerivedRelation`, its typed endpoint reference and the actual operation derived-read association. Ordinary invariant factories resolve `typed_derived_relation`; preparation and evaluation use separate proposed/committed frozen proofs. Decision readers require actual issued source scopes and record physical plus empty-adjacency facts. Finite projected endpoints compose through graph laws, walks, orbits and derived component sets. `setup.derived_relation_count` binds a framework owner to an actual `ApplicationDerivedCollection`; retained counts use the disposable derived-view lifecycle and can be discarded/rebuilt. This closed count input requires an Entity root. | [Derived relation meaning and worked examples](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md#derived-relation-meaning) |
| Retire an authored source and its separately produced realization publications atomically | Declare producer `OWNERSHIP = ProducerPublicationOwnership::SourceScopeCascade` at installation. Only an entity-specific source scope is eligible. The installed authored correction binding supplies the distinguished source; Native computes the complete transitive source-scope closure and checks the exact roster. Typed partial, extra, repeated, foreign-source and corrupt-origin refusals retain affected records. `restore_retired` restores every publication atomically with separate provenance and fresh generations. Owned publications refuse independent retirement (`OwnedPublicationRequiresCascade`), and restoration without their owner (`PublicationOwnerUnavailable`). References and shared input dependencies never imply ownership; independent producers retain their existing behavior. | [Source-owned producer publications](build-an-application.md#source-owned-producer-publications), [custody contract](../plans/WORTH-query/authored-retirement-custody.md) |
| Shrink a producer-generated tree whose children own their publications | The existing SourceScopeCascade declaration also governs producer succession. Native verifies deleted predecessor members as anchors, reuses its recorded source-scope closure, and requires the exact complete retirement in the same commit. Each route keeps its original provenance. Regrowth and demand following recorded size-edit undo create fresh generations/publications; redo uses the existing reapplication contract. Demand `authored_retirement_error()` borrows the typed Native cause and affected records. | [Source-owned producer publications](build-an-application.md#source-owned-producer-publications), [succession contract](../plans/WORTH-query/producer-succession-cascade.md) |
| Return a typed domain denial from an output producer | Declare the provider's `type DenialValue` as a canonical binding of the operation's `Denial` with a finite wire maximum, as `WorthQueryProjectedProducerDenial<P>` projecting each denial onto such a binding or an explicit absence, or as `WorthQueryUndeclaredProducerDenial`. Installation refuses an unbounded binding. A `ProducerDomainDenied` demand denial decodes through `decode_producer_denial::<Binding>()` and keeps exact bytes in diagnostic clones. Mismatched, undeclared and unencodable values are refused by type. `domain_reason()` stays diagnostic text. | [Typed producer domain denial](build-an-application.md#typed-producer-domain-denial) |
| Require an invariant of every producer publication | Declare `REQUIRED_INVARIANTS` on the producer binding. Each publication is admitted only when the rule passes or Relational proves the candidate's footprint touches no kind in the rule's declared closure; the receipt's `non_applicability()` is then `CandidateOutsideClosure`. A scoped-out rule (`NoApplicableTouch`, as for a raw delete), refused preparation, or a candidate with no intents (no receipt) refuses the publication, and the demand denial's `producer_requirement_denial()` names the receipt's verdict and basis. | [How it works Â§9.8](how-it-works.md#98-publication-live-reads-and-output-demand) |
| Observe reads by an installed invariant | `host::application_invariants::WorthQueryInvariantProjectionWork` describes reads. `ScopePlanner::reads()` observes preparation; `Context::reads()` includes preparation plus this evaluation, across every proposed and committed view. Counts describe Query reads and grant no access. | [Invariant reads and observations](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md) |
| Follow an invariant entity between the two views of one proposal | `target.reobserve_entity(&entity_binding, &issued_selector)` freshly observes the same physical entity under the target view's declared kind, original controls and prepared scope. The selector must belong to the same actual proposal and installation; it grants no read authority. A removed target returns `None`, while stale/foreign selectors or undeclared access refuse. The returned handle has the target posture; ordinary cross-view reads remain refused. | [Invariant views](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md) |
| Read complete reachability inside an installed invariant | `view.walk(&binding, RelationDirection::Forward, &root)`, `closure(.., &roots)` and `reaches(.., &from, &to)` consume one installed endorelation. Native-backed results retain the original payer and every visited entity/physical edge enters prepared admission. Cancellation and resource refusal return typed errors. | [Invariant walks](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md) |
| Read a persistent orbit inside an installed invariant | Use `view.orbit(&relation_binding, &member)` on the proposed or committed view. `members()` is successor order from the current anchor, `identity()` is the persistent typed name, and `len()` is the orbit size. `orbit_of(..)` preserves the same complete entity/edge footprint. Both calls obey installed binding, view affinity, declared access and prepared scope; a cyclic relation without the declared orbit law refuses. | [Orbit law and invariant reads](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Inspect a persistent orbit at a selected snapshot | Projection and raw operation readers expose `reader.orbit(Relation::reference(), &member)` and `orbit_of(..)`. The complete owned result exposes successor-ordered `members()`, physical `relations()`, `member(index)`, persistent `identity()` and `len()`. Operation readers require the relation's `OperationReads` permission; all members enter the original realized scope. These inspection reads record no publication decision dependencies. A retained identity grants no read, write or history-disclosure authority. | [Ordinary orbit inspection](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Track an orbit as a mutation or computation decision | `DecisionReader::orbit` / `orbit_of` and computation readers retain original Native aggregate identity/membership and complete successor-walk facts. Lower-level admitted operation readers expose `decision_orbit` / `decision_orbit_of`. Both original bodies are sealed under the actual admission, contracts and realized scope. Fresh comparison detects identity or topology changes; retained aggregate inputs currently require full verification and a fresh computation. Live bodies have no checkpoint reconstruction door. | [Tracked orbit reads](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Query one complete persistent orbit | Add a typed `ApplicationQueryOrbitRef` with `.orbit(slot)` to a `Many` query whose root and scope are the same kind. `result.orbit(slot)` exposes identity, full length and borrowed member rows in Native anchor-forward order, including after disclosed/output-demand conversion. Governed queries require one exact `orbit_by` metadata rule in addition to member-field rules; omission refuses before Native group selection. Filters, root-path unions, explicit ordering, continuation and live lanes are refused. | [Declared orbit queries](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Inspect aggregate lineage under privileged snapshot authority | `reader.lineage(Relation::reference(), &name)` on an installation-owned projection reader acquires real stored incident history and merge-only latest resolution at its exact snapshot; the operation inspection wrapper also requires `OperationReads`. The owned result retains actual Native values and typed related names. A retained name selects the read but grants no history disclosure, member scope or decision/publication authority. An absent name refuses; a retired terminal with no successor succeeds with `latest_identity() == None`. This API does not support scoped/governed lineage, tracked lineage decisions or restoring portable names into a fresh installation. | [Lineage inspection](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md) |
| Read complete reachability in a snapshot projection | In an invariant projection callback, call `reader.walk(Relation::reference(), direction, &root)`, `closure(.., &roots)` or `reaches(.., &from, &to)` with one endorelation and locally resolved identities. One pinned Native traversal retains result funding; the snapshot separately retains its reached scope. `reader.work()` reports actual reads so far. | [Invariant walks](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md#snapshot-projections) |
| Inspect complete reachability in a raw operation projection | The raw operation reader accepts `walk`, `closure` and `reaches` for a relation implementing `OperationReads<Operation>`, using the selected snapshot and installed work budget. These inspection calls do not record publication dependencies. | [Raw operation projections](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md#raw-operation-projections) |
| Record complete reachability as an operation decision dependency | Handler and computation readers accept `walk`, `closure` and `reaches`; the raw operation reader uses their `decision_*` forms. Original issued Native values pass through sealing and complete current-snapshot comparison. Structural computation inputs currently force a fresh run; live carried-value and checkpoint reuse remain unsupported. | [Recorded decision walks](../workspaces/worth-query/crates/worth-query/docs/foundations/invariant-walks.md#recorded-decision-walks) |
| Select complete reachability in a declared query | Compose `ApplicationQueryRootPath::forward_closure` or `reverse_closure` over an endorelation with ordinary hops and guards. Original Native complete bodies support row/set currentness; `receipt().root_path_steps()` reports actual per-step reads. | [Declared query closure](../workspaces/worth-query/crates/worth-query/docs/foundations/declared-query-closure.md) |
| Observe reads by a declared query | On an ordinary application result, use `result.receipt().inspect().read_work().reads()`; the lower Host query receipt exposes `receipt.reads()`. `WorthQueryInvariantProjectionWork` reports actual field, adjacency, endpoint and index reads separately from projection capacity and authorization work. These are Query boundary observations, not total CPU, heap or storage I/O. Reused computation facts add no historical reads; fallback preserves reads already performed. | [Read observations](../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md#inspection-and-debugging) |
| Declare source inspection for an authored operation | Implement `ApplicationSourceDisclosureBinding<Schema>` and add `inspection_disclosure Binding,` to its mutation binding. Declare who may inspect the operation separately from who may correct it. | [Source contracts](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#prepare-the-inspection-contracts) |
| Keep an original query for later explanation | Clone `result.receipt().inspect().observation()` before dropping the result. Inspect it with its original binding; each fresh inspection checks current permission. | [Keep originals](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#keep-the-evidence-you-will-inspect) |
| Preserve a settled producer when reading its output | Query through `request.at(settlement.observation()).query(...)`. Save the original query observation before closing the result. Keep the earlier authored source receipt separately; a plain head query does not infer a supplier. | [Read settled output](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#original-settled-producer-custody) |
| Diagnose a missing retained verification fact | Match `host::primary_graph::WorthQueryOutputDemandDenialKind::RetainedVerificationFactUnavailable(reason)` to distinguish native revision and exact indexed-selection denial categories when retained verification metadata names no retained fact. The actual currentness reader refuses before accepting a clean mark; internal fact positions are not public identities. | [Retained cause courts](../plans/WORTH-query/execution-retained-causes.md) |
| Close an admitted graph plan after operation custody transfer | `WorthQueryAdmittedGraphWorkPlan::release()` returns a release receipt only while the plan retains capacity. Handle `host::admission::graph_obligation::WorthQueryGraphWorkReleaseDenial::OperationCapacityTransferred` when operation execution already owns the reservation. The descriptive refusal grants no capacity or release receipt. | [Retained cause courts](../plans/WORTH-query/execution-retained-causes.md) |
| Diagnose retained settlement provenance | Match `host::primary_graph::WorthQueryOutputDemandDenialKind::SettlementProvenance(reason)` and `WorthQueryOutputSettlementProvenanceDenial` to distinguish absent original or receipt owners, present foreign owners, publication, reused settlement and restored checkpoint disagreement. Descriptions grant no settlement authority. | [Retained cause courts](../plans/WORTH-query/execution-retained-causes.md) |
| Identify the performed action and its fulfilled required root | `settlement.performed_source()` borrows the original action provenance; `source.receipt()` names the action before output production. `source.root_fulfillment()` retains the actual source and qualified root output occurrences with the original performed/reused/recovered posture. It grants no current permission and does not imply producer re-execution or current whole-tree readiness. `root_receipt()` describes the root producer. A read-origin tree returns `None`. | [Required output ownership](build-an-application.md#54-mutations-with-required-outputs) |
| Save query values for later inspection | Call `retain_projection_for_inspection()` before executing each query you will compare or materialize. Capture supports complete one-shot field/relation projections, including empty results. | [Retain original values](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#original-disclosed-projection-custody) |
| Declare who may inspect a query | Register `ApplicationQueryInspectionDefinition<Schema, Binding>`. Inspection metadata is separate from payload disclosure; comparison and materialization also require diagnostics. | [Inspection contracts](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#optional-query-inspection-metadata-meaning) |
| Compare original query results | Use `fresh.inspect::<Binding>(&after).why_changed(&before).reference_only()` with current metadata and diagnostic access. Both originals need saved complete projections. | [Compare results](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#paired-original-comparison) |
| Explain a query result or authored operation | Save the right originals, admit current inspection access, then choose reference, comparison, producer or materialized evidence. | [Inspection walkthrough](build-an-application.md#inspect-original-source-and-query-evidence) |
| Inspect an exact inspection admission refusal | Read `denial.owner_refusal()` for the borrowed original authorization, capability, scope, security, governance, principal, installation, diagnostic contract, graph admission, reporting input, session start, graph read, read terminal, diagnostic admission or producer capture cause. `Governance` identifies admission; `DiagnosticPreparationGovernance` identifies diagnostic preparation, each retaining its exact governance kind. Graph admission through `host::admission::graph_obligation` distinguishes `PlanIdentityExhausted` from `CapacityReservationIdentityExhausted`. `ForeignTarget` identifies an original-target binding mismatch. Handle `WorthQueryApplicationInspectionOwnerRefusal` through `host::causal_inspection`; producer capture diagnostics and `WorthQueryApplicationQueryInstallationDenialKind` are nameable there. Fixed cases also distinguish missing capabilities, source/comparison failures, parameter-binding mismatch and report/plan refusals. Common kernel and storage failures carry `WorthQueryApplicationInspectionStage`, including report identity versus bridge. `DiagnosticComparisonDenial`, `WorthQueryManagedGraphReadDenial` and `WorthQueryInspectionReadTerminalDenial` are nameable through the same facade. Resource and canonical details retain their existing projections. | [Inspection walkthrough](build-an-application.md#inspect-original-source-and-query-evidence) |
| Distinguish canonical inspection report stages | Match `host::causal_inspection::WorthQueryApplicationInspectionCanonicalRefusal` and its `WorthQueryApplicationInspectionStage` slot to retain `ReportIdentity` versus `ReportBridge` for basis, digest, sort, accounting or allocation refusals. | [Retained cause courts](../plans/WORTH-query/execution-retained-causes.md) |
| Read failed-rebase currentness | Match `own_effect.at_own_publication(original_is_selected)` as `FactlessCurrentness::{Current, Superseded, Undecidable}` through `host::causal_inspection`. `RebaseVerificationReason` keeps the exact categorical refusal and indexed denial; native fact positions remain with their private retained-fact reader. | [Inspection walkthrough](build-an-application.md#inspect-original-source-and-query-evidence) |
| Inspect Native decision-read preparation refusal | Read `HandlerExecutionDenial::native_decision_refusal()` for the exact retained Native stop from ordinary or managed-index handler decision preparation. The borrowed diagnostic grants no read authority. | [Application walkthrough](build-an-application.md) |
| Inspect an authored source without a query | Use `fresh.inspect_source::<SourceDisclosure>(&receipt).why_performed().reference_only()`. Public metadata uses `execute()`; governed metadata uses `execute_governed(source_metadata)`. | [Source-only inspection](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#source-only-inspection) |
| Connect a source to the output it created | Use `.why_observed().caused_by::<SourceDisclosure>(&source_receipt)` with independent query/source metadata and diagnostics. Pass the authored source occurrence, not an equal rewrite or the producer publication receipt. | [Creation connection](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#sourceproducer-connection) |
| Explain output when the query omits the authored input | Give the source and query separate disclosure bindings and admit each independently. Their subjects and parameter values may differ. | [Independent bindings](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#independently-admitted-source-and-query-connection) |
| Distinguish changed from equivalent producer output | Opt into `output_equivalence()` for one Required standard-u64 field on an exactly-one Preserve role. Inspect with `.evaluated_by::<SourceDisclosure>(&source_receipt)` and independent source/query admissions. | [Output evaluation](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#performed-producer-output-evaluation) |
| Check what evidence was retained | Read `report.evidence_availability()` or a comparison's before/after availability. These are snapshots; missing capture does not establish expiry or producer absence. | [Availability](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#original-evidence-availability) |
| Read optional conditional history | Select `WorthQueryApplicationMaterializationRequest::conditional_history()` or `.with_conditional_history()` on connected rich inspection. Handle eviction, omission and capacity denial explicitly. | [Conditional history](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#optional-conditional-execution-history) |
| Read an original query's saved values | Select the `QueryInspection` evidence family with `.why_observed().materialize(...)` with metadata and diagnostics. The view reads original fields and nested rows after later state changes. | [Materialize query values](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#rich-original-query-materialization) |
| Read source effects and the producer explanation together | On a `caused_by` or `evaluated_by` request, select `.with_source_effects().with_producer_evidence()` and admit both subjects. | [Connected values](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#rich-connected-source-and-producer-evidence) |
| Read source values without retaining a query | Use `.inspect_source::<SourceDisclosure>(&receipt).why_performed().materialize(...)` with `WorthQueryApplicationMaterializationRequest::source_effects()` with current source metadata and diagnostics. The owned result can outlive the receipt. | [Materialize source effects](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#rich-source-only-inspection) |
| Explain a cleared optional input | Keep the source receipt that performed the clear/removal and the producer's completed absence read. Equal None or a no-op clear does not create another absence occurrence. | [Absence evidence](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#canonical-absence-evidence) |
| Remove an Optional struct parent without weakening field authority | Declare `OperationRemovesAspects<Operation>`, all child decision reads and `operation_remove_aspect`; call `CandidateWriter::remove_aspect` on the genuine completed target. Complete installed read closure and Optional parent authority are required. Required siblings use `optional_aspect_field` only to interpret genuine parent absence; missing Required content in a present parent refuses. | [Causal inspection](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#public-optional-struct-removal) |
| Store ordered, identity-free numerical content at every size | Declare `ApplicationContentBinding`, register `.content_format::<Format>()`, and bind an existing ContentRef field with `content_field`. `DecisionReader::content` returns admitted immutable payload custody; `CandidateWriter::initialize_content` and `write_content` atomically stage Native bytes plus the reference. Managed Source rows lend the same installed content with `row.with_content(selector, Format, callback)`. Demanded preimages and declared correction content outputs retain genuine Native custody through the existing publication seal. Native owns format/version, integrity, retention, recovery and the 16 MiB complete-frame ceiling. Larger frames refuse before effects. | [Content boundary](../plans/WORTH-query/milestone-9.19-k1c-content-boundary.md) |
| Declare a foreign domain value through its entry facade | `decl::application_schema::ValueSeedDelegate` supplies the root grammar via `ApplicationCanonicalValueContract::delegated`, `encode_delegated_canonical_value`, and the bounded or admitted delegated decoder. The existing canonical codec borrows the foreign value without requiring a platform trait on its pure owner. | [Typed invariant refusals](build-an-application.md#typed-refusal-from-installed-invariants) |
| Configure World resources | Pass `RuntimeWorldBudgets`, a clock, invalidation resources and an execution policy to `host::runtime::WorthQueryProductWorldResources::new`. Use `RuntimeWorldBudgets::without_policy_limits()` for standard installation, or install deliberate finite policies. | [World resources](build-an-application.md#runtime-world-policy-ceilings) |
| Own admission time | The existing runtime-time source supplies `WorthQueryAdmissionClock`; pass that same observer to request scopes and authentication-adapter admission. The world retains `WorthQueryAdmissionTimeController` when deterministic hold/advance is required. Authentication, request deadlines and retained authentication events use that monotonic timeline; a foreign observer refuses before authentication work. | `host::admission::authenticated_principal`; `WorthQueryRuntimeTimeSource::admission_clock` |
| Configure Query reads and output settlement | Use `WorthQueryApplicationQueryResourceProfile::new(concurrency)` for graph-work concurrency and `WorthQueryOutputDemandResourceProfile::standard()` for settlement. Convenience settlement continues on `Progressed`, parks on `Pending`, and returns `Settled` when complete. Retain returned owners for later advancement or close. | [Query resource policies](build-an-application.md#ordinary-query-resource-policies) |
| Run a declared fixed action whose source feeds required outputs | Declare `.action::<Binding, Delivery>() ... .finish_action()` and its exact `ApplicationActionReference`; `â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)`, then `operation.advance(&fresh_request)`. Construction reuses the native governed root-artifact/source guard before effects; invalid locality/change/dependency metadata returns the original description with `WorthQueryOperationConstructionDenial::OutputSource`. Replay rejoins registered outputs without rerunning the handler; unavailable registration returns terminal historical evidence. A live fixed owner exposes `release_cleanup(&mut self, &fresh_request) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial>` and Initial-only `close(self, &fresh_request) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)>`; cleanup release advances no business work and does not make a partial operation closable. Settlement supports optionally bounded `read_current_with_policy`, original-basis `read_historical`, and refusal-preserving `close`; explicit `ApplicationNoProjection` uses `deliver_current` without a query. `ApplicationQueryBatchProjection<Read, N>` and `ApplicationFixedBatchDelivery` bind an ordered original-input selector array; `read_batch_current` and its policy form use one native aggregate loan and complete-only publication, while `read_batch_historical` retains the original basis; eligible Required fixed-leaf terminal `close` invokes native unavailable completion, preserving actual negative evidence across cleanup refusal in `output_unavailable()` on terminal or closed custody. Source saved with no current output is not positive settlement | [Build Â§5.4](build-an-application.md#54-mutations-with-required-outputs), executable `consumer_root/â€¦/application_program/fixed_delivery.rs` |
| Declare discovered output delivery meaning | `ApplicationDiscoveredActionDelivery` selects an actual discovered root and closed `ApplicationOutputProjection`; query and bounded batch traits bind pure original-input selectors. Declaration/installation alone grants no runtime admission or current delivery | [Declared output delivery shapes](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#declared-output-delivery-shapes) |
| Own a discovered action through recovery, fanout, delivery and cleanup | `retain_operation::<Program, Action>` selects `WorthQueryDiscoveredOperation` from installed discovered delivery meaning. Fresh-request advancement preserves original input/key/preconditions/source, native committed-retention failure and registered replay. Settled scalar/batch/no-projection delivery checks the complete original aggregate at the same observation and phase as its declared read; history and close preserve custody on refusal | [Owning discovered actions](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-discovered-actions) |
| Validate an explicitly driven discovered aggregate at one retained observation | `request.at(&observation).require_current_discovered_program_outputs(&settlement, maximum_work)` checks the original complete root and dependent discovery facts plus every retained output settlement with one native comparison meter. Empty original discovery still carries negative evidence; native retired roots keep original membership but require no invented output settlement. Refusal borrows and preserves the aggregate. This deliberate native guard alone does not run a delivery query; the owning discovered entry composes both | `consumer_root/.../application_program/discovered/currentness.rs`, host `discovered_currentness_contract` |
| Own a declaration-authorized no-output action | Declare `.action::<Binding, Delivery>() ... .finish_action()` with `ApplicationSourceCompletion`; `â€¦idempotency_owned(key)?.retain_operation::<Program, Action>(&runtime)` returns `WorthQuerySourceOnlyOperation`. The owner retains the exact installed runtime beyond the construction borrow; `into_runtime_for_cleanup()` explicitly abandons ordinary retry meaning and returns the exact runtime plus its actual cleanup collection. Retain both: failed managed-run cleanup has no publication recovery record. `release_cleanup(&fresh_request)` discharges the collection without settling uncertain business evidence. Each `advance(&fresh_request)` returns original custody, completion, or terminal evidence; completion and terminal custody expose refusal-preserving `close(&fresh_request)`. Unexecuted `Initial` custody also permits fresh close after a denied first advance; arbitrary unresolved advancement does not. Recovered completion carries native receipt/publication evidence, without an invented handler result. Retained mode uses `advance(&fresh_request, maximum_history_work: Option<NonZeroUsize>)`; `None` adds no history work ceiling, while a finite bound remains a caller choice. It delivers the exact native original World observation after replay/recovery; bounded history refusal retains evidence for retry | [Retained delivery guide](../workspaces/worth-query/crates/worth-query/docs/foundations/source-only-retained-delivery.md), executable [SourceOnly retention courts](../workspaces/worth-query/crates/worth-query-testkit-certification-operation/src/certification/application_graph/source_only.rs), [owning API](../workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs) |
| Retry a mutation safely | Keep the stable original key, typed intent and product affinity. A journal-only Directory reopen admits fresh authority; an exact keyed retry can return `RecoveredCommitted` and the original recorded receipt without handler replay. Changed intent refuses `IdempotencyIntentDrift`; stale/foreign selection refuses before effects. Keep pending/unpublished or indeterminate owners and follow their typed recovery/close route. Response loss alone does not permit a new key or another effect. Retained MemoryOnly operations still advance with fresh authorized requests on their original owner. | [Commit, retry and close](build-an-application.md#commit-retry-and-close-a-directory-home), [Host retry contract](../workspaces/worth-query/crates/worth-query-host/README.md#commitment-and-same-key-retry) |
| Diagnose an unclean Directory Home refusal | Call the Host opening refusal's `unclean_recovery_unsupported()`; track H refuses before World/service admission with `UncleanRecoveryUnsupported`, naming the durable owner and exact effect digest found after the selected checkpoint. Graceful close and quiescent kill remain supported. C5 retires this refusal when crash recovery is certified. | [Delivery tracks D18](../plans/database-foundation/milestone-3.md#the-happy-constraint-track-h) |

| Inspect a source admission refusal | Source bind, preparation and pending consumption check runtime ownership, installed seal and type, then current request authority before source reads or binding changes. `WorthQuerySourceExpectationDenialKind::AdmissionAuthorization` is a unit kind; `WorthQuerySourceExpectationDenial::authorization_denial()` retains the original authorization denial, including cancellation, expiry and native resource causes. These types are public through the Decl/Host reexports. Host's `WorthQueryRequiredOutputPreparationDenial::from(source_denial)` retains that cause as a dependency refusal, whose `recovery_posture()` describes retry. | [Source denial owner](../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/observed_source/denial.rs) |
| Correct a recorded source operation | Declare `ApplicationCorrectionBinding` with its recorded-inverse or compensation mechanism in the contribution's `corrections`. A recorded field inverse retains one unique mutated record common to every demanded locus, with exactly one admitted observation per locus on that record; collateral records missing another demanded locus do not make it ambiguous. Call `request.corrections().select(receipt, limits)`, then `reverse(&source).idempotency(key).limits(limits).prepare()` and publish the candidate. Handle equivalent replay, stale targets and native custody through the returned outcome. | [Recorded source correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md), [Compiled example](../workspaces/worth-query/crates/worth-query-certification/examples/application_correction.rs) |
| Reverse consecutive original source operations | Select each original receipt under a fresh request and reverse it after the exact later occurrences have their own performed reversals on the selected ancestry. Canonical authored writes, complete field/aspect/presence/live-adjacency support and compatible installed full-generation meaning must agree. Equal values or lists alone do not qualify. Keep original endpoint custody; current disclosure, inverse authority and stale-target checks remain mandatory. Audit-visible support, missing canonical material and unsupported shapes retain typed refusals. | [Consecutive original sources](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#reverse-consecutive-original-sources), [Compiled consecutive-undo journeys](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/consecutive_undo.rs) |
| Reapply retained source input from an explicit history choice | Enable `visit_reapplication` and `reapplication_key` on the correction binding. Read `corrections().alternatives(&source, limits, maximum_entries)` through its continuation, select the chosen reversal, then call `reapply(&alternative).idempotency(key).limits(limits)` with the source action's ordinary preparation and delivery. | [Explicit historical alternatives](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#reapply-an-explicit-historical-alternative), [Tree journey](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/tree_reapplication.rs) |
| Keep a correction chooser across authenticated requests | Retain the native page returned by `into_page()` in the server's session lifecycle. Complete traversal, then use a fresh request's `corrections().select_alternative_coordinates(&page, &chosen_source, limits)` before preparing reapplication. Selection validates the exact target and current disclosure; refresh a stale chooser and retain the same command key for an equivalent retry. | [Chooser request lifecycle](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#carry-a-chooser-across-requests) |
| Correct an authored conditional definition | Capture the authored parameters in the recorded inverse. The ordinary handler prepares the installed definition with `DecisionReader::prepare_conditional_definition`, carries its owner permit in the decision, and joins it to authored field writes with `CandidateWriter::advance_conditional_definition`. Publication exposes the complete Relational and Signal successor together. | [Conditional-definition correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#correct-an-authored-conditional-definition), [Ordinary handler](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/composite_definition_handler.rs) |
| Select a correction source after normal operation close | Keep the completed receipt's `correction_source()` locator, close its operation, then call `request.corrections().select_recorded(&locator, limits)` under the current caller. For transport, serialize `locator.coordinates()` and use `select_coordinates(&coordinates, limits)`. Native evidence lookup, World ancestry and disclosure validate the selection. | [Closed source selection](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#select-a-source-after-closing-its-operation) |
| Keep a corrective action's installed delivery owner | Use `prepare_operation::<Correction, Program, Action>(&application)`. Publish a prepared candidate or continue the returned original action, then advance, read and close its native owner under fresh requests. | [Corrective action delivery](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#keep-the-corrective-actions-delivery-owner) |
| Compensate a completed external workflow operation | Declare the actual source effect, compensation and exact `completed_external_reconciliation()` procedure. Preparation checks the original dispatch's authenticated inbound completion and originating workflow settlement; the new compensation preserves the original remote effect and workflow disposition. | [Completed external compensation](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#compensate-a-completed-external-workflow-operation) |
| Derive compensation from an original created output | Bind the source's actual exactly-one Create role, declare its fields with `ApplicationCorrectionOutput::<Source, Role>::field(...)` in `source_output_fields()`, and decode the borrowed original values from `ApplicationCorrectionSourceMaterial::Compensation(fields)`. | [Original created-output fields](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#read-original-fields-of-a-created-output), [Compiled binding](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/source_output_correction.rs) |
| Regenerate required outputs after a recorded correction | Use `prepare_performed::<Correction, Program, Root>(&application)` and retain its performed-source owner. Start its required outputs under a fresh request. An unpublished owner resumes through `recover_recorded_correction` and `promote_recorded_correction`; a retention failure resumes through `retry_recorded_correction_retention`. These owners retain the actual captured inverse and transformed key. | [Required outputs and recovery](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#required-outputs-and-recovery), [Compiled recovery](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/required_output_courts.rs) |
| Control who may do what | `decl::worth_query_capability!`, `decl::worth_query_policy!`, `decl::application_capability`; a binding declares `ApplicationCapabilityMutationBinding` and capability admission; its owning operation freshly admits the capability at execution and unpublished recovery | [Authorization and elevation](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md) |
| Observe an original elevation lifecycle result without minting phase custody | `request.observe_capability_lifecycle_idempotency_in_program(program, principal_binding, capability, operation, input, key, preconditions)` freshly authorizes the installed operation and returns immutable recorded meaning or a typed pending/refusal result. Keep actual predecessor owners for transitions. | [Lifecycle retry](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-authorization-and-emergency-elevation.md#lifecycle-retry-observation) |
| Require different people for different duties | `decl::application_capability::ApplicationCapabilityActorComposition::new(ApplicationCapabilitySeparationOfDutyRule::when(..), ApplicationCapabilityDistinctActorRule::when(..), ApplicationCapabilityRecordedActorRule::not_applicable())` inside `ApplicationCapabilityComposition::new(..)`. Each rule is a graph path from the principal | [AI README: capability authorization](../workspaces/worth-query/crates/worth-query/docs/AI_README.md#capability-authorization); worked use: `workspaces/worth-query-bank-world/crates/bank-domain/src/schema/estate/capability_composition.rs` |
| Require the original workflow actor to be present and compare it with the acting principal | `ApplicationCapabilityRecordedActorRule::required(ApplicationCapabilityRecordedActorRequirement::new(source, role, comparison))` in the capability actor composition, paired with `ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source)` or `for_operation` on the spec | [Workflows: recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) |
| Create a subject and enroll it atomically with owning recovery | Declare `creation.enroll_in_workflow::<Spec, Binding, Role>()`, install `birth_operation`, and prepare native `RequiredWorkflowBirth`. Capture one original descriptor, then `for_workflow_birth::<Spec, Role>(&workflow, &required)?.retain_operation::<Program, Action>(program)` selects the declared SourceOnly owner. Frozen input/key and actual one-shot birth authority survive refusal, keyed/unpublished recovery and cleanup; `into_parts` recovers effect-free bound custody. | [Workflow-enrolled creations](../workspaces/worth-query/crates/worth-query/docs/authoring/workflow-enrolled-creations.md) |
| Run a multi-step process | A workflow: `decl::application_program::ApplicationWorkflowSpec`, `ApplicationWorkflowDefinitionBuilder`; host lanes `prepare_workflow_publication`, `prepare_workflow_instance_start`, `prepare_workflow_proposal`, `prepare_workflow_advance` | [Build Â§6](build-an-application.md#6-author-publish-and-run-workflows), [Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md) |
| Start a fresh episode after a completed workflow | Scope/Preserve body `.succeed_workflow::<Spec, Binding, Role>()`; install `.succession_operation::<Binding, Role>()`, then native `prepare_workflow_succession` / `for_workflow_succession` with the same logical input/key. Preparation accepts the runtime-owned published reference or original pure start descriptor; after checkpoint restore, the latter requires fresh command admission and tracked native lookup. | [Completed-chain succession](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#completed-chain-succession) |
| Retain the actual outputs of a performed workflow operation | Native guarded settlement records the binding and actual role/entity/posture/reference inventory atomically with the transition; Existing, Created and Retire identities survive audit retention. No additional supplier flag or host write. | [Retained operation correspondence](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-operation-correspondence) |
| Require succession to continue the selected round or revision | Body `require_predecessor_output::<Binding, SelectedRole, Producer, ProducedRole>()` compares its sealed Preserve/ExactlyOne output with the latest performed occurrence of the declared guarded producer; both roles must name the same entity. Exact replay precedes fresh mismatch refusal. | [Succession output affinity](../workspaces/worth-query/crates/worth-query/docs/foundations/workflow-succession-output-affinity.md) |
| Run a workflow step that needs no signature | Guarded operation with `workflow_authority required, workflow_requirements ApplicationWorkflowExecutionRequirements::occurrence_only()` in `decl::worth_query_mutation_binding!`; on `AwaitingOperation`, `â€¦for_workflow_operation(&workflow, &required)?.retain_operation::<Program, Action>(program)` then `advance(&fresh_request)` | [Build Â§6.5](build-an-application.md#65-who-may-run-a-guarded-operation), [Workflows: Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations); example `workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/information.rs` |
| Certify shared and row-local facts without copying their source truth | A native signed creation uses `capture_from_path(ApplicationQueryRootPath, source, snapshot, Subject/AuditContext)` with v3 graph provenance. The existing lifecycle derives source-identity and selection-scoped supersession; `bound_subject_impacts` bounds the union of affected subjects. | [Graph-certified subjects](../workspaces/worth-query/crates/worth-query/docs/authoring/graph-certified-subjects.md) |
| Own capability delegation or revocation across fresh requests | `request.retain_capability_delegation_in_program(..)` / `retain_capability_revocation_in_program(..)` own original input, key, conditions and principal association. `advance(&fresh_request)` returns progress, same-owner refusal, native completion or terminal custody; `close_with_evidence` discharges cleanup before moving the receipt | [Owning capability commands](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#owning-capability-commands) |
| Own a guarded workflow effect across fresh requests | Effect-free bound construction refusal returns the original binding; consuming `into_parts()` extracts its original description and requirement beyond the runtime borrow. `WorthQueryWorkflowOperation` retains the original description and issued slot; explicit `advance(&fresh_request)` delegates native unpublished, keyed and dispatch recovery. `accept_owned_operation(completed)` returns the completed owner on refusal and retains it beside successful workflow progression; `close(&fresh_request)` discharges actual cleanup | [Workflow operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations), [Bank owning workflow](../workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs) |
| Use explicit workflow recovery with a held original | Advanced `accept_operation_from_owner`, `accept_recovered_operation_from_owner` and `prepare_workflow_operation_recovery_from_owner` require the actual `WorthQueryApplicationCommitReceipt` as well as the freshly admitted bound request. Recorded original data cannot satisfy that argument. Bank's advanced wrappers require its privately constructed performed operation. Ordinary callers retain the workflow operation itself. | [Workflow owner contract](../workspaces/worth-query/crates/worth-query-host/src/workflow_owner_contract.rs) |
| Inspect a workflow source independently of dispatch recovery | `WorthQueryWorkflowOperation::inspect_source(&fresh_request)` freshly admits the original binding and resolves native custody, then borrows its retained source evidence with `Committed`, `DispatchPending` or `ExternallySettled` posture. No recovery handle is minted and no delivery, settlement or cleanup occurs. `SourceEvidenceUnavailable` means completion has not been established, not that no effect happened. | [Source inspection](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#inspect-source-without-advancing-dispatch) |
| Own a separate workflow assessment observation | `into_assessment_demand(..)?.controls(controls).retain_operation(program)` transfers the actual admitted demand. `advance(&fresh_request)` obtains fresh disclosure; `close(&fresh_request)` releases failed producer cleanup and observer interest while preserving custody on refusal | [Assessments](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#assessments) |
| Run a workflow step that needs a signature | An `approval::<Capability>(id)` node wired with `approval_authority(&approval, &operation)`; declare `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` to make it mandatory; sign with `prepare_workflow_approval(..)` and a fresh authentication event. For a compiled signed body on that approval control, use `prepare_workflow_approval_with_body(..)` to co-publish its attestation and approval under one event; exact settled replay needs no new event. Its evidence join accepts one or more distinct assessment inputs; empty applicable evidence never grants approval. | [Workflows: Approvals](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#approvals), [Approval with attestation](../workspaces/worth-query/crates/worth-query/docs/authoring/signed-operation-creations.md#one-approval-action-with-a-business-attestation) |
| Prepare and publish an actual mutation | `OperationHandler::decide` / `prepare_decision` reads through `DecisionReader` and carries typed targets, owned keys and prepared values in its `Decision`; `build_candidate` consumes them through `CandidateWriter`. The framework opens the effect program directly and preserves checked sizes, cancellation, fallible growth, invariant validation and atomic commit. | [Build: prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects) |
| Check the runtime affinity of a stored entity reference in a handler | `reader.reader().runtime_instance_id()` describes the actual Native runtime owning the operation projection. Compare it before entity coordinates, then check the admitted scope's full identity and observed target membership. The opaque reader access retains its decision-reuse restriction; the value grants no authority. | [Build: prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects) |
| Execute a stored typed edit with independent mutation permission | Construct the ordinary installed source-completion action, then `advance` it with fresh request authority. A retained owner's `assess_current_authorization` uses its declared admission mode. Raw requests use `assess_current_authorization` for conventional policy or `assess_current_capability_authorization` for capability admission. These observations grant no later execution authority. For staged edits, assess on the source and construct the execution owner on the exact candidate branch. Installed commit-boundary invariants enforce value limits; retain the exact `Progress`, `Refused`, `Completed` or `Terminal` owner for retry or lawful close. | [Build: execute a stored typed edit](build-an-application.md#execute-a-stored-typed-edit) |
| Delete an authored entity | Declare the entity read and delete grant; observe `reader.entity(&identity, Entity::reference())`, carry its mutation target into the candidate, then delete and bind its actual Retire output role. | [Build: prepared decisions and typed effects](build-an-application.md#prepared-decisions-and-typed-effects), [Compiled handler](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/source_output_handlers.rs) |
| Publish typed fields, links and emissions | Use the original `CandidateWriter` under the operation's declared create/write/relation/`OperationEmits` authority, then bind its actual fixed output roles or family members. Payload validation and output cardinality remain enforced. | [Build: mutation bindings](build-an-application.md#52-mutations-intent-binding-handler) |
| Act on time or a signal | A conditional operation: `.conditional_operation::<Op>()` on the feature spec, `host::worth_query_conditional_node!`, and a host-supplied predicate and named clock | [Build Â§3.2](build-an-application.md#32-attach-actions-to-features), [Conditional installed operations](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md) |
| Complete work from an external system | An installed inbound source: `runtime.inbound_occurrences().receive(source, bytes, scope).execute()` (`host::application_entry::WorthQueryApplicationInboundOccurrencesExt`); in a workflow, `await_inbound::<Effect>(..)` | [Aftermath: authenticated inbound completion](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#authenticated-inbound-completion), [Workflows: waiting for an inbound completion](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#waiting-for-an-inbound-completion) |
| Survive a restart | For a supported Relational-only journal-only Home, open the same `Directory(path)` through the ordinary Host request. Reopen preserves authored history, does not invoke `initial_state` again, and admits original keyed receipts under fresh authority. Consuming Directory `close` owns teardown; a refusal retains its actual Home and lease for `try_close`. MemoryOnly has no restart guarantee. Checkpoint admission/recovery, program transitions and durable branch creation remain pending with the owners named in §4. | [Initialize and reopen](build-an-application.md#initialize-once-then-reopen-the-same-directory), [Directory close](../workspaces/worth-query/crates/worth-query-host/README.md#close-a-directory-home) |
| Close a Home with installed signing owners | `home.close(&request)` consumes the intact signing registry with its original Query owner. A runtime alias refuses `Busy(ProgramRuntime)`; refusal/resume keeps the registry. Final supplier destruction waits for `closed.reclaim()` or abandonment, and external authentication owners are not revoked. | [Directory Home close](../workspaces/worth-query/crates/worth-query-host/README.md#close-a-directory-home) |
| Discard a development Home | `host::discard_home(host::WorthQueryApplicationHomeLocation::Directory(path))` destroys the Home under Store's mutation lease. Close the owning Home first; a held lease returns `WorthQueryApplicationHomeDiscardDenial::MutationOwnerContended`. Retry interrupted disposal explicitly; a tombstone fences opening with `DiscardPending`. The next open initializes a new Home. | [Host disposal contract](../workspaces/worth-query/crates/worth-query-host/README.md#close-a-directory-home) |
| Open a Relational-only Directory Home | Phases 3/4 compose an InitialEmpty Signal root and Bridge baseline. The same supported program/schema opens under a new build UUID without initialization or eager regeneration. Producers, conditionals and resource pools return preparation `UnsupportedSeed` before configuration or physical Home creation; its typed D15 cause retains the required family and support phase. | [Directory seed contract](../workspaces/worth-query/crates/worth-query-host/README.md#relational-only-directory-installation) |
| Supply trusted authorization time to a Home | `WorthQueryApplicationHomeRequest::with_authorization_time_source(source)` carries the same trusted clock contract through MemoryOnly, fresh Directory and Directory reopen. Opening does not sample it; capability decisions preserve `TrustedTimeUnavailable`. | [Host trusted time](../workspaces/worth-query/crates/worth-query-host/README.md#relational-only-directory-installation) |
| Prepare an issued provider-session grant on its receiving application | `runtime.prepare_provider_session_grant(&phase, grant, receiver)` supplies the installed provider and actual Home witness. Stale and foreign grants refuse before the lazy receiver constructs any run, reads or supplier work. A current receiver must carry the exact original plan and provider. | [Host session preparation](../workspaces/worth-query/crates/worth-query-host/README.md#writer-contention-and-incarnation) |
| Retire unused program support across graceful reopen | `home.runtime().retire_program_support(&execution, &revision)` acknowledges durable Retired status. Keep that revision rostered on reopen; `supported_program::<P>()` stays unavailable and repeated retirement returns `AlreadyRetired`. Live branches, interpretations and mandatory custody refuse retirement. | [Directory support retirement](../workspaces/worth-query/crates/worth-query/docs/foundations/programs-and-adoption.md#directory-support-retirement) |
| Work on a branch or move it to a new program under its declared capability | `runtime.branches().fork(runtime.current_world()).components(..).create(&phase)`; `request(..).on_branch(branch)`; `.programs()` then `compare(&target)`, `adopt(&requirements)?.prepare(max)`, `programs.publish(prepared)`; `.workflow(WorthQueryWorkflowDispositions::retire_and_cancel_unperformed())` explicitly retires definitions and cancels only legal unperformed instances under the admitted adoption principal | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Build Â§7](build-an-application.md#7-adopt-a-new-program-on-a-branch) |
| Run isolated transaction steps on a fork | `runtime.branches().fork(head).components(..).create(&phase)`, then `request(..).on_branch(fork)`. The application's model-state owner selects the branch for subsequent requests; `runtime.on_branch(discarded).close(&phase)` retires discarded branches. The fork retains its parent's history for corrections. | [Branches](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#application-product-branches-host), [Fork transaction example](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/fork_head_adoption.rs) |
| Retry retired application product cleanup under a fresh admitted request | Keep the `WorthQueryApplicationProductBranchCleanup` from `OwnerCleanupPending`. Inside the original runtime's `with_application_advancement(scope, \|phase\| ...)`, call `cleanup.retry_in_advancement(&phase)`. Foreign or interrupted phases return the same cleanup owner; retained readers may still refuse. Release those readers and retry that owner. Keep the actual completed receipt even if the outer request later stops. | [Build: retired cleanup](build-an-application.md#register-attempts-on-a-stable-product), [Compiled control and release court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/proposal_acceptance/courts/acceptance/disposal.rs) |
| Fork the exact source already selected by the application | Pass the owned selection to `runtime.branches().fork_expected(selected)`, choose components, then `create(&phase)`. Keep the returned candidate and use `candidate.branch()` for ordinary requests. Adopt with `candidate.adopt(&mut model.head, &phase)` while the exact source remains current, or discard with `candidate.close(&phase)`. Refusals retain the candidate and typed denial. Close the previous branch separately after adoption. | [Fork from an admitted selection](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#fork-from-an-admitted-selection) |
| Own an application branch head | Consume a genuine expected-source candidate with `into_owned_head()`. Use `head.branch()` for requests and `head.adopt(candidate, &phase)` for guarded replacement. Close the returned previous head explicitly. A refused close retains either the live head or the actual retired cleanup owner. Inside an advancement, retry that cleanup with `retry_in_advancement(&phase)`; creation recovery uses `release_obligations_in_advancement(&phase, minimum_age_ticks)`. Both retain the original owner on refusal. | [Own an application branch head](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#own-an-application-branch-head) |
| Attach a model to its actual owned head | Declare `ApplicationHistoryBinding` and its ordinary READ/operation membership in the existing contribution. Call `request.attach_history::<Binding>(&phase, head, model_read)` on that branch. The move-only document freshly admits `read(&request)` and refuses model or head divergence. `assess_edit(&request, &operation)` observes current eligibility of an original no-output action using fresh READ, native action authorization and exact model/head correspondence; it grants no execution permission. Dispose with `document.close(&phase)`; preserve any live-document or retired-cleanup failure. | [Attach a model and read its owned head](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#attach-a-model-and-read-its-owned-head) |
| Carry a document-history declaration in a package archive | Export the package's typed schema record and use `encode_record_frame` and `WorthQueryPackageArchiveRecordDecoder`. Identity, full READ association, metadata policy, model domain and authored operation order/labels survive exactly. Installation resolves original typed recipes; live document attachment still uses its admitted model head. | [History declaration and attachment](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#attach-a-model-and-read-its-owned-head) |
| Carry a governed model payload purpose through history | `attach_history_with_payload::<History, _, _, _>(&phase, head, access, capability, operation, input)` returns a document whose typed payload recipe repeats original query admission on every fresh read, including retained preview reads and internal edit reads. It stores no grant or approval. Metadata purpose remains independent. | [Governed payload READ](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#use-a-governed-payload-read) |
| Read the selected document step under its metadata policy | Pass an explicit Public or Governed metadata disclosure contract to `ApplicationHistoryDefinition::for_binding(..)`. `current_step(&request)` uses public metadata; `current_step_governed(&request, capability, operation, input)` freshly admits the metadata purpose alongside the ordinary READ body and filters. Fresh governed edit enrollment and cached-attempt lookup use `advance_with_metadata(..)`. A payload or edit grant cannot expose step metadata. | [Selected step metadata](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#read-selected-step-metadata) |
| Refresh an attached record model after outside control writes | Declare `ScopeEntityRecord`, then call `refresh_context(&request, limits)` or `refresh_context_governed(..)` with fresh READ and metadata authority. Native checks bounded authentic transition evidence and rejects any model-record authorship, including equal-value and ABA writes. Success preserves recorded endpoints, the selected step and cursors. | [Refresh context after control changes](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#refresh-context-after-control-changes) |
| Page completed document steps | `history_page(&request, limits, after.as_ref())` and its governed counterpart bound rows, examined entries and metadata backing. Rows report Root, Retained(parent) or Truncated boundaries without excluded parent IDs. Foreign/stale cursors refuse, failed reads preserve position, and private/discarded attempts do not change the visible view. Rows describe source retention at assembly and hold no historical payload lease. | [Completed-step pages](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#page-completed-document-steps) |
| Drive a disposable history view through checked commands | Pages and redo child pages expose `context()` with format, actual installed schema binding and process-local document identity. `document.commands(&context)` rejects unsupported or foreign correspondence and borrows the same document; its preview, navigation, bookmark/window and maintenance entries still use the ordinary current admission and actual continuation owners. Preserve a queued navigation's original expected selection and key. Consuming edits/groups check `check_history_command_context` before transferring their original document. | [Executable document adapter](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/document_history.rs), [consumer guide](build-an-application.md#43-what-the-runtime-gives-you) |
| Read an exact historical document step | Open `document.preview(&request, node_id)`, then call `preview.read(&document, &request)`. The holder keeps the original composite source; each read freshly uses the actual current document's authority. Rows and source/basis descriptions expose no retained-selection conversion. Close results and holders independently. Governed metadata supplies its current capability through both governed calls. | [Historical step reads](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#read-a-historical-step) |
| Select a historical model state and edit a new future | Explicitly declare an entity-record history domain and `ApplicationHistoryNavigationBinding`. Use `document.selection(&request)`, then `navigate::<Navigation>().checkout(node).expected_selection(expected).idempotency(key).prepare(&request)`. Retain the original expected selection, target, key and pending/refused owner; `owner.posture()` reports candidate preparation, output work and selection progress. Fresh READ, metadata and NAV protect each call; current controls remain current. Selection creates a writable context without adding an authored node. Configure a compatible fixed required-output tree with `document.with_outputs::<Program, Root>(..)` and preserve its typed document mode through advancement and close. Unsupported output, workflow, external-effect or component meaning refuses explicitly. | [Historical checkout](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#select-a-historical-step-for-editing), [Required outputs](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#configure-required-outputs) |
| Navigate a parent or an explicit child step | Use `navigate::<Navigation>().undo_step()` or `.redo_to(child)` before the same expected-selection, idempotency and preparation stages. Undo resolves the recorded parent; redo validates the completed child's membership. Root and non-child targets refuse explicitly. Preserve the original command and expected selection for retries. Both commands retain current admission, required outputs and cleanup through the ordinary checkout owner. | [Historical navigation](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#select-a-historical-step-for-editing) |
| Discover completed redo alternatives without scanning the tree | `document.redo_choices(&request, limits, after)` returns descriptive `NoChild`, `Single(child)` or `ChoiceRequired` plus a bounded completed-child page. Classification is independent of page width; cursors bind document, parent, generation and listing scope. Use `redo_choices_governed` for capability-governed metadata. A chosen or remembered child still requires fresh explicit `redo_to(child)` admission. | [Historical navigation](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#select-a-historical-step-for-editing) |
| Configure a document-wide history window and bookmarks | Install separate `ApplicationHistoryWindowBinding` and `ApplicationHistoryBookmarkBinding` recipes. Use `set_history_window(target).advance::<Window>(&request)`, `set_history_bookmark::<Bookmark>(&request, node, true/false)`, `history_retention_status` and `maintain_history`. The default window is 200 unpinned steps; current and bounded bookmarks are separate. Excluded IDs refuse fresh opens/pins, existing holders retain original observations, and explicit close preserves refused cleanup. No byte or physical reclamation promise. | [Window and bookmarks](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#configure-a-navigation-window-and-bookmarks) |
| Bound unfinished document navigation | Set `ApplicationHistoryBinding::NAVIGATION_RESOURCES.maximum_unfinished_attempts` (default 8). A full document refuses a distinct key before target lookup or fork; same-key retry shares its original slot. Selection or completed disposal releases that slot. Dropped handles and refused cleanup preserve registered custody. Completed history and cleanup use their separate resource owners. | [Navigation ownership and limits](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#select-a-historical-step-for-editing) |
| Retain receipt evidence without pinning physical publication history | `receipt.descriptive_copy()` preserves original commit, intent and terminal evidence with identity-only publication custody. It carries no performed-change witness or full product basis. Ordinary `Clone` continues to retain publication custody; describing a copy does not retire the source or release a sibling holder. | [Receipt evidence API](../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_receipt/description.rs) |
| Record an isolated model edit | Consume the attached document with `document.edit(intent).idempotency_owned(&request, key)?.retain_in_program::<Program, Action>(host)`. Advance the owned edit, then `finish(&request)` to receive the document, sealed step and original action completion. Source-completion actions return `WorthQueryHistoryEditOutcome`; fixed required-output graphs return `WorthQueryFixedHistoryEditOutcome` with the original closed delivery or terminal evidence. Native qualifies the complete original output/discovery set at one retained basis before exact adoption. Both require no mutation source, workflow control or external effects. Reads use the selected head while candidate work remains private. Equivalent process-local retries return their original step without resetting selection; changed intent refuses. | [Record an edit and retain its original outcome](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#record-an-edit-and-retain-its-original-outcome) |
| Group ordinary source and fixed-output edits into one user step | Consume the document with `group_edits::<Intent, Program, Action, N>(host, &request, group_key, members)`. The homogeneous group preserves payload/output modes, each original operation and its complete required tree; `parts()` separates shared document access from execution. Complete once at the last member's genuine endpoint, then finish or explicitly abort. Fixed completions retain original closed deliveries; earlier historical settlements survive later members. Installed member/result limits apply before effects; nested, workflow and external-effect groups are refused. | [Grouped edits](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#group-ordinary-edits-into-one-step) |
| Store an expected basis and freshly select it | Serialize `selected.describe_basis()` as ordinary application data. After a fresh ordinary READ of that record, deserialize `WorthQueryExpectedProductBasisDescription` and call `runtime.branches().select_expected(&description, &phase)`. World and Native admit the original exact current basis; stale, retired, foreign and unsupported descriptions refuse. The data retains no lease and grants no READ or mutation permission. Consume the returned selection with `fork_expected`. | [Stored expected basis](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#store-an-expected-basis-and-select-it-again) |
| Record typed comparison data without moving the model basis | Create a separately owned storage branch with `parts.fork_relational()`. Write and read records through ordinary requests on that branch, with a canonical declared field codec for structured values. Retain its branch owner for explicit close. | [Storage and typed records](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md#store-an-expected-basis-and-select-it-again) |
| Bind a complete ordered attempt before checking its basis again | An ordinary registration action on a stable storage product gives native equivalent-outcome/intent-drift handling for the full selected input. Keep the original loaded meaning and issuing product, then freshly admit proposal/model/result READ and exact frozen mutation authorization before protected lookup. Registration grants no adoption authority. | [Build: register attempts on a stable product](build-an-application.md#register-attempts-on-a-stable-product) |
| Record and disclose an application's performed result | Keep weak canonical result data separate from a recording command constructible through actual sealed adoption custody. Observe genuine claim/result correspondence and complete absence in the mutation handler; read cached data through current governed policy. Preserve actual Fresh/Recovered provenance. | [Build: record an actual adoption result](build-an-application.md#record-an-actual-adoption-result) |
| Own Bank HTTP mutation and correction custody | Create/resume/read/close commands retain native operations across requests. Money actions supply retained activity reads; estate freeze, recorded restoration and capability revocation supply receipt completion. Retain `BankHttpMutationContinuation` before activating the listener; fresh authenticated commands advance and close each original owner | [Owning HTTP mutations](../workspaces/worth-query-bank-world/docs/http-owning-money.md), [Recorded estate restoration](../workspaces/worth-query-bank-world/docs/http-owning-money.md#correcting-an-estate-freeze), [Bank consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md) |
| Own plain Bank account, access or payment mutation | Bank builders for personal/business account creation, access grant/revocation, initiation and either rejection form use `retain_operation()` and native SourceCompletion; supply fresh authority to each `advance` and `close`. Fresh results and recovered receipt evidence remain distinct | [Bank owning completion guidance](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-account-access-and-payment-completion) |
| Own Bank estate source or retained funding/reversal | Typed Bank mutation builders use exact declared SourceCompletion or existing RetainedCommit; `close_with_evidence` transfers original completion only after native cleanup | [Bank source and publication handoff](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md#own-estate-sources-funding-reversal-and-deployment-revocation) |
| Retain Bank HTTP native cleanup and business evidence | The mutation continuation retains actual cleanup and closed source/capability owners. `release_cleanup` retries physical release on the owning thread. Local `take_released_cleanup` and `prepare_closed_mutation_handoff` transfer their entire native evidence with the installed runtime; a lost handoff restores exact custody. | [HTTP cleanup custody](../workspaces/worth-query-bank-world/docs/http-owning-money.md#estate-effects-and-native-cleanup-custody) |
| Issue or revoke Bank program deployment permission | Explicit `BankProgramDeploymentGrantSeed::for_program::<Source>(..)` in `BankWorldSeed`; recorded issuer calls `revoke_program_deployment_grant(..)` | [Build Â§7.1](build-an-application.md#71-the-call-chain) |
| Read current data | `request(..).query(intent).execute()`; exact reads with `retain_current_read()` and `at(&observation)`; live reads with `query(intent).subscribe(..)` | [Host README: ordinary typed query entry](../workspaces/worth-query/crates/worth-query-host/README.md#ordinary-typed-query-entry), [Host README: output demand and live reads](../workspaces/worth-query/crates/worth-query-host/README.md#output-demand-exact-observation-and-live-reads) |
| Read governed data at a retained observation | `request.retain_current_read()`, then `request.at(&observation).query(intent).execute_governed(capability, operation, input)`. Data stays at the observation; principal, capability and READ admission use current authority on the requested branch. Describe the same observation with `select_on(&phase, runtime)?.describe_basis()` after admission. | [Governed reads at one observation](build-an-application.md#read-governed-data-at-one-observation) |
| Read the original workflow instance at a retained observation | `request.at(&observation).read_original_workflow::<Spec, SubjectRead, AuditRead>(workflow.vocabulary(), &original_intent, subject_input, audit_input)` uses fresh installed read bindings before lookup. `Live`, `Historical` and authorized `Missing` retain exact basis data; audit is independently admitted and otherwise explicitly withheld. An ended instance's audit preserves its `WorkflowEndingActorProvenance` (principal or durable Home/World owner identity), without granting command authority. | [Governed original-intent reads](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#governed-original-intent-reads), [Recorded actors](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#recorded-workflow-actors) |
| Read bounded workflow history and reacquire proposal references | `request.at(&data).read_workflow_history::<Spec, SubjectRead, AuditRead>(vocabulary, target, subject_input, audit_input, width, indexes, after)` returns exact-basis pages with independently withheld audit. `subscribe_workflow_history` / `poll_workflow_history` use native envelope dependency marks and fresh disclosure. | [Retained history](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#retained-proposal-and-approval-history) |
| Read one workflow definition's metadata | Retained `discover_workflow_definition` with explicit typed `ApplicationWorkflowDefinitionReadTarget` and the separately installed `DefinitionMetadata` binding; fresh admission precedes selected-index lookup. | [Governed definition metadata](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#discover-and-retire-definitions) |
| Retry committed discovered-source retention | Ordinary `advance` retains and retries the native owner. For deliberate native control, retain the opaque `WorthQueryDiscoveredOutputRetentionFailure`; use a fresh original `advanced()` native request's `retry_discovered_output_retention(failure, application)`. Refusal returns the original owner; success promotes its actual carrier before starting native outputs | [Committed discovered retention](../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md#committed-discovered-retention) |
| Retain an application preview through readmission, discard or promotion refusal | `runtime.begin_preview(&phase, &observation, request)`; `preview.readmit(&phase, &runtime)`; `readmitted.promote()`; either phase's `discard()` returns its same owner on refusal. Fresh readmission validates the original source and runtime; promotion finalizes retained preview evidence, separately from source/output publication. Rejected preview remains explicitly discardable. Name its native result vocabulary through `worth_query_host::facade::application_installation::{BridgeSpeculationError, BridgeSpeculationErrorKind, BridgeSpeculativePromotionOutcome}` | [Preview custody](build-an-application.md#56-retain-preview-custody-across-refusal) |
| Own a complete no-source output tree | `request.demand(original).controls(controls).retain_program_outputs::<Program, Root>(&program)` retains the actual installed full tree. Owned construction refusal supports fresh `retry` without replacing an issued basis. `advance` returns pending/refused/settled/terminal custody. Settled `read_current` checks all original outputs and discovery before reading at the same native observation and phase. Every phase can `close`; an owned close refusal supports `retry_close`, including after source supersession. Native unpublished and uncertain obligations remain owned; close delegates lawful original native cleanup/recovery and otherwise refuses | [Complete no-source trees](build-an-application.md#55-own-a-complete-no-source-output-tree) |
| Close original complete output custody before fresh final delivery | Settled Fixed and complete no-source parents expose `close_for_delivery(&fresh_request)`. Actual native cleanup/interest close precedes sealed `WorthQueryClosedFixedDelivery` or `WorthQueryClosedProgramOutputDelivery`. Their existing-shape `deliver_current`/`read_current` methods validate the original full tree and installed meaning at a fresh native observation. To show several closed carriers current at one common basis, each no-projection Fixed and no-source carrier's `deliver_current_at(&request.at(&observation))` runs that same installed-action or selected-root gate and full lineage at the caller's observation. Fixed close refusal returns the same parent; no-source `WorthQueryProgramOutputDeliveryCloseRefusal` supports close-only `retry_close` | [Delivery after native close](build-an-application.md#delivery-after-native-close), Host `closed_delivery_contract` construction/phase/shape family |
| Retain complete read-tree qualification at a pinned observation | After native close, `WorthQueryClosedProgramOutputDelivery::qualify_at(&request.at(&observation))` checks the selected typed root and all original outputs/discovery once at that exact basis. `WorthQueryCompletedReadOutputs` retains original read-source provenance, root posture and the qualified common output basis, including receiptless reuse. It grants no query or mutation authority; drop facts and carriers to release readonly history | [Delivery after native close](build-an-application.md#delivery-after-native-close) |
| Qualify the original read tree's endpoint | `WorthQueryClosedProgramOutputDelivery::qualify_original(&request)` derives the endpoint from canonical World publication continuity and an actual original retained endpoint read, then checks the selected typed root and complete original output/discovery set. `WorthQueryCompletedOriginalReadOutputs` keeps genuine source, root and endpoint custody distinct. Later unrelated head movement is never captured as the endpoint; a disconnected prefix cannot stand in for a qualified complete tree. Receiptless reuse can keep the endpoint at the source | [Delivery after native close](build-an-application.md#delivery-after-native-close) |
| Dispose of an original complete tree inside a runtime phase | Live, terminal and construction-refused read-tree owners expose `close_in_advancement(&phase)`; settled owners also expose `close_for_delivery_in_advancement`. Close-only refusals support `retry_close_in_advancement`. The original runtime's live phase suffices without an application request, payload READ, or nested advancement. Refusal retains the exact original native remainder; a closed delivery keeps readonly pins until dropped | [Delivery after native close](build-an-application.md#delivery-after-native-close) |
| Inspect original root output roles without a producer receipt | `host::application_entry::WorthQueryApplicationProgramOutputSettlement::root_settlement()` borrows the actual immutable root demand settlement; use its `outputs_of::<Contract>()` typed correspondence even when `root_receipt()` is `None`. Parent operation retains all tree/close custody; this root-only evidence does not establish fresh complete-tree currentness | [Original root output evidence](build-an-application.md#original-root-output-evidence) |
| Poll cancellation/deadline in a decision handler | `DecisionReader::checkpoint()` polls the real request. Preparation retains its first failure and refuses before candidate authoring even if caught. Successful polls preserve completed-decision input-reuse eligibility under the existing producer/context contract. | [Build Â§5.1](build-an-application.md#51-who-is-asking-principal-and-request-scope) |
| Bind a derived collection to checked computation | `ApplicationDerivedCollection` requires `Computation` and `Completeness`; declare the computation beside the collection and install with `setup.derived_collection::<Feature, Collection, _>(owner)`. The owner's output and stop types must match the completeness contract. Initial/Create and Preserve/Preserve producers may share that installed owner and operation with a common no-intent input: implement `ApplicationMutationInputScope<Schema, Binding>` for each selected mutation binding. Ordinary intent inputs delegate their existing single-binding scope. Scope selection grants no execution authority. Complete installed operation/workflow-stage target coverage is checked before initialization, separately from exact per-binding route inventory; `installed_conditional_targets_for_schema` exposes read-only full locations, and conditional declarations use `in_workflow_stage` for stage selection. Missing or misbound installation refuses before bootstrap effects; the existing keyed computation checks its completed value before publication. Native partition reuse selects retained facts from the exact settlement frontier and consumes validation at the same pinned reader snapshot; applications provide no dirty hints. Successful native reduction retains its Execution-owned run; a fresh descendant request advances only changed inputs and actual removals, sharing unchanged native results and fitting cost ranges. Fixed-membership tree updates need no full shape plan or deletion scan. Execution owns first-stop and tight-prefix charging; applications cannot supply a result handle or checkpoint tape. Context-sensitive or nested kernels execute rather than retaining unsupported custody. Fixed membership with unchanged keys also carries checked reader-call ranges and immutable item/call bounds; ordinary preflight/charges and exact reached-entity scope still apply. Qualified native postcommit rebase consumes paired original/postcondition custody, the complete snapshot-aligned frontier and actual candidate touches; fresh suffixes and changed leaf sizing remain charged. Missing association or continuity reconstructs in full. Qualified completed original physical reads share checked locator/effect routing through a genuinely fresh registered session. Both comparisons retain native source and registration checks; changed or unqualified source uses funded full original-fact comparison. Principal and source-role observations remain individual. Posting registration and membership processing remain separate obligations; key/membership changes retain full preparation. | [Build Â§3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) |
| Inspect original producer publication or admission stops | Borrow the native demand denial inside the original advance result. `publication_companion_stop()` identifies a commit resource stop; `producer_admission_stop()` identifies native preparation/readmission stops, including structural causes. Both retain the original `host::primary_graph::CompanionPreflightStop` and supplied quote. `output_verification_capacity_stop()` retains the original `VerificationCapacityStop` for output verification, including precommit consumed edges and preparation/readmission: the meter stop, scratch-size overflow or allocation refusal remains distinct. Demand capacity evidence may overlap producer admission; the commit getter specifically identifies precommit verification. All three `None` leaves the cause unknown. Keep the original operation/cleanup custody | [Producer publication diagnostics](build-an-application.md#producer-publication-diagnostics), [Host borrowing example](../workspaces/worth-query/crates/worth-query-host/src/facade.rs) |
| Produce derived or computed outputs | Feature spec `provides::<Port>()`, `derived_artifact`, `managed_computation`; demand with `request.demand(demand).controls(controls).start()`. The handle or retained continuation owns actual failed producer cleanup; `release_managed_cleanup(&fresh_request)` on the handle releases it without output settlement. Diagnostic denial clones carry no cleanup authority | [Build Â§3.4](build-an-application.md#34-ports-connections-and-the-output-graph), [Build Â§3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation) |
| Produce from authentic sources and declare artifact lifecycle | `WorthQueryApplicationProducerProvider` supplies semantic identity, operation input and idempotency from the original source, and declares its typed domain denial value. `ApplicationDerivedArtifact` declares output/locality, retention, succession, required posture, producer, dependencies, reuse and stopped outcome. The installed mutation path enforces source/currentness and publication contracts. | [Producer source and publication](build-an-application.md#producer-source-and-publication) |
| Reverse a business change | Declare a domain action with its required source, authorization and effects, then execute it through the ordinary owning request path. | [Owning actions](build-an-application.md#53-execute-through-the-program) |

Use the committed construction and explicit cold-refusal contract above.
Structural candidate Source loans do not grant scalar or managed-index reads.
Declare an invariant's managed Source requirement separately: supported scalar
and direct-endpoint Sources use the same maintained resource through committed
DecisionReader and candidate invariant reads. Optional parent-aspect result fields
are not supported by maintained Sources. Unsupported projection, predicate
and ordering shapes return `UnsupportedInvariantSource` at installation. See
[maintained Source authoring](build-an-application.md#maintain-a-provider-source-for-an-invariant).
Native `ForkExact` creates a branch-specific managed Source association while
sharing immutable material; `ReuseExact` reads its actual selected component.
Sibling edits remain isolated, and cross-branch Source expectations refuse
before provider execution.
Use the freshly admitted Parent's `evict_index(&handle)` (or its governed/approved
variant) for selected maintained cache cleanup. Cold content returns typed
`RootUnavailable`; admitted candidate reconstruction preserves unchanged Source
conditions. Retained historical images remain valid shares. The request-free
handle eviction retains `InvariantOwned`; fresh `rebuild_index(&handle)` refills
the selected maintained Native cache after independent Source READ and provider
purpose admission. Cancellation before install preserves the previous cache; an intervening
cache edit refuses the stale build. `ReuseExact` rebuilds its selected older
component while newer main and fork material remain unchanged. The explicit cold
lane pays for the ordinary Source query plus Native occurrence projection.
After its Native component deletion completes, closing the owning product
retires its live Source lookup; an existing
`ReuseExact` alias can still read, evict and refill its retained selected image.
Retained immutable custody does not permit fresh Native branch admission or writes.
For default `Refuse`, fresh unpublished bootstrap explicitly calls
`graph.initialize_managed_index::<Index>(phase)` for new scopes in its next seed
attempt. Normal rules still judge it; selection is consumed even by an empty
(refused) or unrelated batch. Installed/recovered runtime cannot borrow that
permission, and ordinary new/cold scopes keep their declared refusal.
A retained logical Source during
generated-output suspension does not make a current public physical read Ready.
The generated Catalog prefix journey uses actual producer/reconstruction custody:
logical rules preserve suspended rows, an exact manifest cannot bypass the
reconstructed proposed-domain check, and valid restoration serves fresh reads.
Exact authoritative-record correspondence can share the selected suspended
Source payload without provider construction or maintenance, while publication
mints a fresh available association. Any authoritative field difference declines
this conservative proof and uses the declared cold policy. Original-request
cancellation after a real rule query returns suspension custody for retry.
Its generated geometry counterpart declares the Edge and both Vertex roles in
one real output publication, reconstructs actual endpoint relations and both
memberships, and rejects B's third enclosure. Exact geometry restoration performs
no build/update. Original suspension cancellation retains the available basis and
fresh physical reads; `execution_control_stop()` exposes the typed stop while
preserving the owning Native preparation denial.
Generated cold-B coverage evicts a previously attached B and closes its public
registration. Cold construction cancellation leaves it cold; retry builds it
before either rule. Fresh pinned ordinary Source queries keep current principal
admission: they refuse during suspension, then read the original endpoint after
a changed lawful restoration. Retained managed Parent mode remains unsupported;
old data cannot supply fresh security or select a current managed root.
Generated suspension and restoration also qualify real Native-ahead/World-pending
recovery. Live publication preserves typed activation exclusion. After its actual
unpublished carrier returns, fresh reads select the predecessor World image;
cancelled continuation returns custody and fresh continuation adopts the settled
Native image with no provider/rule replay. This is pending recovery evidence,
not competing-winner interference or crash persistence.
The existing unpublished application's `attempt_identity()` and
`native_semantic_result()` expose its original World correlation and retained
Native meaning when available. These borrowed observations grant no publication,
release or replay authority; retain the original owning recovery posture.
See the
[current limits](build-an-application.md#prepare-candidate-structural-source-changes)
and [Native audience](../workspaces/worth-query/crates/worth-query/docs/modeling/schema-validation.md).

For managed or partitioned handler computation, use the installed
`decision_plan` entrance described in [Build Â§3.6](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation).
It owns prepare/compute/complete and propagates the original typed failures.
Use `checkpoint.poll()` for request controls and
`HandlerResult::from_result`/`into_result` with `HandlerFailure<Domain>` to
propagate failures. Supply domain inputs, partition keys and computed values;
return the completed decision to the installed handler lifecycle.

Action metadata retains its existing capability scope: locality/change and
delivery belong to the exact occurrence; correspondence lookup is binding-type
scoped; rule/provider lookup is operation-type plus rule/provider-type scoped.
Use the installed evaluator for guidance and submission enforcement. Resolve
external values and admit their provider revisions before passing them to the
authorized source action.

Deliberately specialized native declarations use `.native_action::<Binding>()`
and the same optional clauses followed by `.finish_action()`. Declare delivery
for product mutations. For a concrete binding issued
only by an installed conditional, use `.conditional_native_action::<Binding>()`
and `.finish_action()` (DSL `conditional_native_action(Binding) {};`). It carries
binding membership with conditional ownership;
installation still checks the exact conditional operation set and real binding,
handler and producer contracts. Configure initial workflow
vocabulary before cloning runtime handles or retaining operations. A refused
`retain_workflow_spec` preserves runtime, installed spec and signing owner in
`WorthQueryWorkflowRuntimeBindingRefusal::into_parts()`; refused
`support_workflow_spec` returns the original spec with its denial.
`WorthQueryWorkflowApplicationRuntime::close_conditional_runtime(&mut self)`
delegates native close while retaining the configured wrapper and its installed
workflow/signing companions on refusal. Close the actual operation owners and
retry the same wrapper.

Before disclosing retained SourceOnly status, active, completed and terminal
owners expose `assess_current_authorization(&fresh_request)`. It checks the exact
installed action through the binding-selected conventional/capability owner,
using frozen input and original workflow context without staging, claiming a key
or advancing custody. Success is descriptive, not execution permission. See the
[owning entry guide](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions)
and the native SourceOnly authorization tests in `native_workflow_operations`.
Authentication or a cached command-owner match is
not sufficient for current disclosure authorization.

Completed source owners also expose `original_input()` as a read-only borrow of
the exact frozen encoded input, including recovered completion. It grants no
read, publication or reconstruction authority. Server's typed `CompletionData`
projection uses this only after completion and fresh authorization; active and
terminal custody cannot claim a created locator through this metadata. See
[source command custody](../crates/worth-server/docs/query-source-custody.md).

Generic hosts constrain bindings with `WorthQuerySourceOperationBinding<Schema>`
and call those ordinary owners directly. The sealed interface carries native
dispatch, not an authorization witness or a consumer-replaceable callback.
For bounded HTTP custody, `worth-server::mount_query_source(adapter, limits)`
retains real Query owners, rejects duplicate/key/capacity admission before
preparation, stores returns before replies and freshly governs inspection.
See [Query source custody](../crates/worth-server/docs/query-source-custody.md).
This process-local transport selects SourceCompletion bindings. Retain the live
Query owners in the mounted adapter through inspection, recovery and close.

---

## Use requests and retained operations

Declare schema and program meaning through `worth-query-decl`, then install and
execute it through `worth-query-host`. Bind workflow vocabulary before retaining
operations or cloning the configured runtime.

Use the branch's selected program and an authenticated application request to
prepare an action. Retain its original input, key and operation across
`Progress`, `Pending` and `Refused` results. Supply a fresh authorized request for
the next advance, inspect its typed outcome, and close the returned owner when
its cleanup obligations are complete.

Drive workflows by matching their returned needs. For
`AwaitingProposal(RequiredWorkflowProposal)`, inspect the node, operation and
input type, then use fresh `prepare_workflow_proposal` admission. Use admitted
principal identity for actor checks and reacquire live workflow references from
the installed runtime after reopen.

Declare time- and signal-driven work with `conditional_operation`. Submit clock
or predicate observations through the installed conditional runtime and advance
the eligible operation through Query.

For business reversal, declare a domain operation with the required source,
authorization and effects. For branch adoption, retain the target branch's
native adoption result and handle its typed preparation and publication outcomes.

For fresh program installation inside an already admitted original execution request,
use `host::application_contribution::with_bootstrap_advancement_from_request` and pass
its phase first to `host::application_installation::in_memory_rostered_program_in_advancement`,
then the validated program, roster, declaration, configuration, build identity, World
limits and seed callback. To install a host-owned authorization clock in that same
bootstrap, use `in_memory_rostered_program_with_authorization_time_source_in_advancement`
and supply the clock after the callback. Setup spends the supplied request; the installed World keeps
its declared runtime request policy. Default installation entries still open setup
from that policy. See [installation funding](build-an-application.md#4-install-or-reopen).

For ordinary checkpoint restoration under a separately funded opening request,
pass that same bootstrap phase first to
`host::application_installation::WorthQueryInterimCheckpointRestore::rostered_program_in_advancement`,
then the validated program, roster, declaration, configuration, build identity,
World limits and checkpoint. The checkpoint is the sole model source; no seed or
transition callback is accepted. The installed World keeps its declared runtime
request policy. Default checkpoint restore still opens from that policy.

For installation and checkpoint-transition work inside an already admitted original
execution request, use `host::application_contribution::with_bootstrap_advancement_from_request`.
Its bootstrap phase borrows the same cancellation/deadline and cumulative native work.
Call `host::application_installation::WorthQueryInterimCheckpointRestore::rostered_program_with_transition_in_advancement`
with that phase and an explicit capture policy from
`phase.execution_request().allocation_policy()` or a descendant of that funding lease.
An unrelated lease refuses before effects with
`ExecutionDenied(ExecutionResource { NestedLeaseMisuse })`. Transition resources
can further narrow capture alone with `WorthQueryCheckpointTransitionResources::with_capture_memory_limit(bytes)`;
the runtime derives a descendant lease, preserving original controls and preparation funding.
The capture-bearing path
requires leased backing and refuses unsupported serial backing with the existing
`ExecutionDenied(ResourceEnvelopeMismatch)` vocabulary, before effects. Bootstrap
custody cannot admit installed-runtime execution. For a prepared ordinary mutation,
`prepare_in_program_report_in_advancement` and `commit_report_in_advancement` borrow
the installed application's phase and accept allocation from that request or a
bounded descendant, without opening a foreign request. A performed recovery capsule's
`repair_to_checkpoint_with_request(original)` returns that same capsule on opening,
settlement or capture refusal. Existing convenience constructors still own their
explicit bootstrap request and capture policy. See [original-request transitions](build-an-application.md#44-checkpoint-program-transitions).

## Known gaps

The K1c Native content owner admits immutable typed frames and closes Native
transaction references before publication. Native checkpoint and journal recovery
reconstruct their retained closure through the existing physical member boundary.
The field-bound Decl/Host content door, managed Source acquisition and recorded
correction closure use that same Native owner. Application checkpoints recover
current Native content. A restored maintained Source without its live World image
returns RootUnavailable until an opted-in authored cold staging transition; bytes
do not mint that image. Checkpoints do not persist
live correction occurrences or preimage carriers. Missing live history remains an
explicit refusal. A decoded or cold Native envelope without its admitted historical
content directory returns HistoryNotResident. A dangling identity in an admitted
directory still returns MissingContent. K1c's bounded-cost courts measure complete frames at 1/2/4 MiB
and local reads/edits against 100/1,000/10,000 unrelated retained entries;
[qualification evidence](../plans/WORTH-query/milestone-9.19-k1c-content-slice5-evidence.md)
records exact invoices and typed bounds. Ordered numerical content uses ContentRef
at every size; values above the complete-frame ceiling refuse at admission.
Applications must not introduce a separate byte store. See the
[content boundary and slice order](../plans/WORTH-query/milestone-9.19-k1c-content-boundary.md).

For implementation identity, use the whole-application
[`ApplicationBuildIdentity`](build-an-application.md#application-build-identity).
The build pipeline embeds one UUID; installation binds it to every producer.
A changed build UUID admits the same supported Relational-only Directory program/schema
without eager regeneration. Directory accepted-output reuse and demand regeneration
remain Phase 6; MemoryOnly keeps its existing output-lineage contract.

Use the capability table to find a supported task. Describe additional domain
meaning in the application:

- The meaning of a signature is its approval capability. Declare one
  capability per meaning.
- A reason for a change is domain data in the proposed input. The signature
  covers it through its input identity.
- Worklists and queues are derived views of domain state.

---

## Reporting a gap

If the five steps above leave you without a way, report it. Write:

1. **The task**, in domain-neutral terms (for example "a document must be
   signed by two different reviewers").
2. **What you searched**: the facade modules, rustdoc, examples, and this page.
3. **The nearest existing API**, with its facade path.
4. **Why it does not fit**, in one or two sentences.
5. **The known-gap row** it matches, if any.

Open an issue in this repository with that content, or give it to the platform
owners.

---

<a id="which-source-wins"></a>

## API reference and examples

- Read the facade's rustdoc for function signatures, type bounds and failures.
- Use [Build an Application](build-an-application.md) for the declaration,
  installation and execution sequence.
- Browse [runnable examples](../workspaces/worth-query/crates/worth-query-certification/examples)
  for complete installations and request handling.
- Use the [Glossary](glossary.md) for platform terms.

---

<a id="keeping-this-page-current"></a>

## Workflow and value reads

Use `read_workflow_instances` with an explicit Subject/Definition scope to read
retained workflow pages. Supply opaque-reference choices or native
carry-compatible intent to adoption, and handle its typed workflow denials.
Request instance-enumeration authorization separately from definition metadata.

### Explicit optional aspects

Applications declare whole-aspect absence with
`ApplicationSchemaDeclarationBuilder::optional_aspect` through `worth-query-decl`.
`aspect` remains Required independently of each field's presence. Installation,
value meaning, predecessor comparison and current package archive record protocol8
retain this law. Native producer births with absent payload require every actual
binding to be a supported Optional EntityField struct; Required and Defaulted
aspects do not acquire absence permission. Tracked reads retain Optional whole-aspect absence independently of field absence; another field materializing that aspect invalidates the read. See the compiled `optional_aspects.rs`
example in [build guidance](build-an-application.md#whole-aspect-absence).

For explicit scalar storage, pair `aspect` or `optional_aspect` with
`scalar_aspect_field(entity, field)`. The aspect must contain exactly that Required
primitive scalar field; installation refuses extra or optional children and
structured canonical carriers. `field` continues to declare Struct storage.
Ordinary field readers/writers and installed query projections use the exact
scalar binding. Optional-parent reads use `ApplicationQueryOptionalAspectFieldRef`.
For a source-connected `u64` input, use `U64ApplicationValueBinding`, an ordinary
producer input query and a Struct output on the same full-generation subject.
Admit the source-disclosure and output-query bindings independently; scalar
storage alone supplies no producer connection.
For a Scalar producer output, use the same explicit storage declaration and
ordinary field writer/output role. Build its readiness with `for_field` and
`field_dependency` on the actual output field, then project that Scalar field
from the settled observation. Readiness seals the physical binding and consumes
the Scalar `WholeAspectSet` event; a changed Struct sibling cannot supply the
queried output's missing revision or effect.
`producer_commit()` names current settlement; `output_publication_commit()` names
the actual field-producing publication. A selected own-publication continuation
can carry one publication candidate across the same admitted source occurrence.
A canonical mutation of its output records resets the candidate; a successor's
own eligible publication can replace it. Mixed field origins are not a history
lookup. Connection still requires the candidate's original
source/input occurrence, current full-generation output role and exact observed
revision/contract/canonical effect. Currentness and admissions remain fresh.
For an output-join refusal, `producer_connection_refusal()` preserves the coarse
`OutputUnconnected`; `producer_output_refusal()` describes the furthest disclosed
output-join candidate's failed revision, binding, contract or canonical-effect
condition. It exposes no values or identifiers and returns `None` before
diagnostic admission or for source/input and request refusals.
See [scalar storage and source connections](../workspaces/worth-query/crates/worth-query/docs/capabilities/cross-runtime-causal-inspection.md#explicit-scalar-aspect-storage).

Ordinary Query reads use installed result cardinality and fresh per-read admission;
`request.query(intent).limit_results(n).execute()` narrows that cardinality.
`request.query_batch(intents).execute(&phase)` runs installed read slots under
fresh per-read authorization. Supply intents in the batch's declared order.
Live declarations name cause/binding/effect/scope/target meaning without a resource
contract. Request Live controls retain channel buffer capacity and materialized-record
cardinality; their managed queue occupancy is released on delivery or normal close.
For ordinary derived readiness, use the
[typed field builder](build-an-application.md#canonical-nested-values-and-derived-readiness)
against the completed declaration. The lower-level
`WorthQueryOutputReadinessContractBuilder::new` accepts a native projection for
families that own that projection directly. Supply an effect-free synchronous
predicate over the declared field dependencies.


### Managed asynchronous resources

`decl::application_resource` declares process-local resource input/result/failure
codecs, provider role, finite capacities, retry/timeout profile, four independent
action requirements and explicit disclosure slots. `schema.installed_resource`
resolves this meaning against actual typed recipes and the installed principal
and scope binding. A transported codec description grants no provider, action or
completion authority. For Host entry, provider progression and cleanup, see
[application resources](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-resources.md).

Installed resource access purposes carry only their actual action and requested
disclosure facets. Close has no Inspect or result obligation; detailed close adds
only cleanup detail. ResourceRead admission binds that exact installed contract
to genuine provider capacity, without borrowing the Access query's payload plan.

The internal engine's owned async entry returns a move-owned custodian. Entry
refusals return any admitted custodian in `WorthQueryOwnedAsyncRuntimeStop`;
timeout/retry/revalidation borrow that same owner mutably. Immutable retained
witnesses select their original occurrence and do not become successor custody.
The UI opening adapter preserves and cleans up the actual refused-entry owner.
This internal integration remains separate from the application's installed
provider and fresh action/disclosure path.

The lower Bridge resource declaration can explicitly opt a queued producer into
process-local retention. Its move-owned release consumes only that producer's
genuine terminal/queue/completion custody, preserving both owners on refusal and
returning distinct released evidence on success. Host uses it to release actual row custody before returning its
Native slot under fresh action and disclosure admission.

For asynchronous resources, the lower World/Bridge owner binds each admitted
request to the exact committed Relational target, including populated forks
whose branch version is zero. Empty targets return typed `NoCommittedSource`
on that committed-source entrance before Signal admission. The explicit internal
`admit_owned_bridge_async_request_at_branch_head` route instead retains a genuine
observed head snapshot without a commit or fresh-currentness claim; the read-only
UI presentation backend uses that posture. The separate managed resource
entry through Decl/Host uses the declared provider and fresh action/disclosure
path in [application resources](../workspaces/worth-query/crates/worth-query/docs/capabilities/application-resources.md).
See [native async source custody](../crates/worth-runtime-world/README.md#owned-async-source-binding)
and the [internal engine guide](../workspaces/worth-query/crates/worth-query/docs/capabilities/async-resources-and-result-state.md).

### Typed relation initialization

Declare relation-owned scalar fields with `ApplicationRelationAspectDefinition`
and `ApplicationRelationFieldRef`. An operation needs both Link and explicit
field initialization targets; its typed writer retains fields only on a newly
created relation with a same-program handle. Native touch admission enforces the
same contract for raw candidates. Bootstrap invariant readers can inspect those
actual physical payloads through `relation_field`; ordinary operation and query
readers do not acquire that privileged door. See
[relation payload creation](../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#relation-owned-payload-at-creation).

Native derived-index definitions retain their original owner-issued IDs through
both checkpoint and canonical journal recovery. Register with `index_authority()`;
the next actual authoritative commit carries the pending definition prefix.
Checkpoint CHK18 captures the same full catalog/frontier cut, and artifact v4
requires the delta in every envelope projection. Recovery accepts exact overlap
without reallocating IDs; derived generations still use existing build/demand
owners. Preparation and uncertain/refused publication never acknowledge catalog
coverage. See [durable index definitions](../crates/worth-relational/README.md#durable-index-definitions).

Native callers can retain opaque caller meaning with
`facade::mvcc::RelationalCallerAssociation::from_bytes` (nonempty, at most 4096
bytes), then consume a validated proposal through `bind_caller_association`
before its one existing canonical preparation. Candidates and canonical
envelopes expose immutable association observations; prepared candidates cannot
be rebound. Artifact v4 and checkpoint CHK18 preserve the bytes through Native
recovery and refuse older versions. Native interprets no World meaning, and the
bytes confer no completion or durable-effect authority. See
[opaque caller association](../crates/worth-relational/README.md#opaque-caller-association).

The same canonical envelope and live result share one original immutable
`facade::transactions::RelationalCommitSemanticResult`: exact create text and
assigned identities, changed records, structural and summary meaning, actual
publication counts, and original validation execution/check/decision meaning.
The image excludes live snapshots, timings, work measurements and publication
claims. Required per-commit `facade::durability::RelationalAssociationDiscovery`
allows associated subjects to be selected without reading unrelated cold bodies.
Missing images/discovery and older artifact v1/v2/v3 or checkpoint CHK13/CHK14/CHK15/CHK16
refuse; decoded descriptions do not grant recovered publication authority.

To record an exact current Native component, call the concrete basis service's
`export_recorded_current_component(execution, &basis)`. Its Native-owned serde
reference distinguishes `Empty` from a canonical commit's actual receipt,
position, artifact digest and roots, and retains canonical-frame reference axes.
Stale and foreign admitted bases refuse. The description confers neither durable
source qualification nor recovery authority; see
[record a current Native component](../crates/worth-relational/README.md#record-a-current-native-component).

The same basis service's `export_recorded_retained_component` exports the exact
original admitted and pinned subject after head advancement, preserving its
original receipt/position/digest. Historical Journal and checkpoint subjects use
request-driven catalog materialization; independent body readmission checks the
complete authoritative payload, while resident canonical identity avoids that
reconstructive comparison. Failed source demands refuse explicitly. Native
recovery binds root images to the original source envelope before subsequent
commits make them historical, preserving original validation without executor
replay. Recorded-component admission checks the complete authoritative payload.
The durability authority provides
`discover_associated_effects`, `discover_all_associated_effects` and
`readmit_associated_effect` for selected or all opaque associations: discovery leaves unrelated bodies cold, and readmission verifies
the exact retained Journal source/index/body rather than a resident re-encoding.
Associated Memory refuses. Matching performed route flags distinguish pending
from settled; `CheckpointReadmitted` describes directory evidence only, and
missing registration never implies settled. These descriptions confer no
recovered publication capability; see
[readmit an associated Native effect](../crates/worth-relational/README.md#readmit-an-associated-native-effect).

For actual reconstruction, `durability_recovery().recover_with_authority` returns
the existing Native recovery authority. Its `admit_recorded_component` admits
exact source-produced Empty or canonical images with actual observation pins;
canonical Memory and missing/foreign/mismatched images refuse.
Its closed-readmission `discover_recovered_associated_effects(execution, &runtime)`
checks exact recovery/runtime affinity before the same complete associated
inventory, even when empty. It grants no live publication fence or absence proof.
Its descriptive
`recorded_publication_successor` selects the original after-reference from the
same source-backed effect and this recovery's captured observations when a crash
preceded the caller's occurrence; missing/ambiguous images refuse, and it grants
no adoption. Its `admit_recorded_publication` consumes the actual readmitted associated effect and
original before/after references, verifies matching settled/recovered owner state,
and acquires both real component leases before issuing the concrete
`worth_proof::RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>`.
Readonly observations use weak owner/source identities; the concrete phase owns
real pins and transfers them once. Refusal retains source data and partial custody.

For one provider-issued placement registration, consume
`RelationalDurableMemberCustody::into_retirement()` and retain the actual
`RelationalDurableMemberRetirement` across Busy, Stopped and typed refusal.
Advance borrows the original issuing source, whose weak allocation identity was
captured at acquisition; unrelated callbacks cannot release the payload.
Complete releases only this local share. Unknown opaque/composite providers
refuse. Retired payload reclamation and generic Drop remain unbudgeted; this
does not qualify whole cold close. See [provider custody retirement](../crates/worth-relational/README.md#retire-one-provider-member-custody-share).
Native never fabricates a live result. CHK18 preserves required exact canonical
component headers and explicit Empty/schema headers from current cells and live
external component leases. One brief Native retention-owner cut linearizes external acquire/release/Drop
with capture. Capture owns the selected actual bases through complete actual
selection acknowledgement or last-capture abandonment after original lease
release; reopened historical artifact bodies stay cold until exact source/root
admission acquires real pins. Plain observations and released leases add no durable
component obligation. Missing selected material refuses. Store/World recovered
adoption and the coherent Home checkpoint cut remain integration obligations; see
[admit an original recovered publication](../crates/worth-relational/README.md#admit-an-original-recovered-publication).

The same weak basis service's `release_commit_result_snapshot(&CommitResult)`
releases only that immutable Native-issued result's original published snapshot.
It validates same-owner registration identity and result/snapshot axes. Released
means actual removal; AlreadyReleased requires that original registration's
actual released fact, not absence. Foreign, mismatched or unknown bindings refuse;
independent reads survive and result Drop does not auto-release. See
[original result snapshot cleanup](../crates/worth-relational/README.md#release-an-original-commit-result-snapshot).

For request-bounded original-result retirement, that same service exposes
`begin_commit_result_snapshot_cleanup(Arc<CommitResult>)`. Its owning task's
`advance(request, &actual_native_owner)` checks real owner/cell affinity and
returns owning pending/refused progress or the original result on completion.
The original cell retains actual removed custody until observation/lane/registry
retirement finishes; removal alone never means cleanup complete. Home/World must
retain the task and original result across stops and lost responses. Independent
readers survive. Generic final Drop and whole Native/Home teardown remain outside
this request-bounded advancement contract.

An owning platform lifecycle can consume its actual immutable Memory Native
handle with `into_memory_close`, retaining the opaque phase outside callbacks
and advancing it with the original request. Actual source/shared affinity,
accepted work and explicit registrations refuse; sealed steps retire genuine
cache observations and head tokens while independent readers retain custody.
Completed private allocations expose only explicit unbudgeted reclamation;
incomplete Drop is unbudgeted abandonment, never a successful close. This is no
durable Home or arbitrary-destructor deadline guarantee. See
[Memory Native retirement](../crates/worth-relational/README.md#retire-an-immutable-memory-runtime).

### Native record and aspect history pages

The Relational owner provides a retained cursor inside one exact selected commit:
`retain_record_history_subject` resolves an exact entity/relation generation in
its admitted root, including retained tombstones; `record_aspect_history_cursor`
requires independently owner-issued exact commit selection. Each `next_page`
bounds canonical record visits, returned entries and newly allocated page backing
under the original request. A finite `retain_record_history_selection` can be
reused by exact per-commit cursors; each cursor keeps independent source custody.
Serial and inherited accounting stays with the retained owner until its final
drop. It retains the canonical envelope without copying
its patches. This native cursor does not establish World product ancestry or
application disclosure. World's `open_exact_product_history` and staged one-parent
pages supply genuine retained product ancestry, fork inheritance and exact
component evidence, with typed refusal at retired gaps. The existing
`select_product_branch_ancestor` also requires an unbroken raw-parent path.
For endpoint ancestry without missing intermediate contents, use World's explicit
`select_retained_product_branch_ancestor`: the same owner follows only its recorded
retirement splices, retains the actual source, rejects siblings/foreign owners and
honors the original request and finite selection bound. Query's recorded-source
correction uses this endpoint proof alongside independent Native evidence and
disclosure checks. Exact selectors and pages keep refusing retired gaps.
They retain the pinned source instead of reading a newer live head; see
[exact product history](../crates/worth-runtime-world/README.md#exact-retained-product-history).
The installed Decl/Host route declares an
`ApplicationHistoryQueryDefinition` over a real `SubjectRead`, captures only its
disclosed exact generations, and opens/continues a bounded cursor at a pinned
product with fresh SubjectRead and independent metadata admission.
See [installed capture and paging](../workspaces/worth-query/crates/worth-query/docs/capabilities/record-and-aspect-history.md)
for the API, separate capability decisions, typed gaps and final release.
See [native paging and retention](../crates/worth-relational/API_OVERVIEW.md#bounded-record-and-aspect-history).

### Create independent component forks from retained truth

Platform owners with a memory-only Native runtime can admit an original basis
through `fork_port().admit_retained_fork_source(&basis)` and consume its typed
source with `fork_retained_branch`. World's branch port admits an original
`ProductBranchObservation` through `admit_retained_branch_source`, then consumes
it with `create_product_branch_from_retained_source`. Supply real `ForkExact`
destinations for both components; Signal must still match its original exact
source. Handle profile refusal and retain any partial effect owner through
cleanup. See [Native retained forks](../crates/worth-relational/README.md#fork-an-actually-retained-root)
and [World retained creation](../crates/worth-runtime-world/README.md#create-a-branch-from-retained-component-truth).

### Select retained entity payload while preserving current surrounding truth

Platform owners with a memory-only Native runtime can transfer a pristine
current branch transaction, an actual retained basis and a full entity ID to
`preparation_port().admit_retained_entity_selection`, then consume the selection
with `prepare_retained_entity_selection`. Publish or discard the returned
candidate through the existing owner. Other records and intervening history
remain current; installed invariants still judge the proposed payload.
Handle identity/schema/currentness refusals and retain application permission
and model-scope admission separately. See
[Native retained entity selection](../crates/worth-relational/README.md#select-a-retained-entity-payload-on-current-truth)
for lifecycle and persistence limits.

### Graph-read admission in the default Query workspace

`worth-query-admission::integration::review_graph_read_access(requirements, inventory)`
checks read requirements against the available native index inventory. Handle
typed async, persistent-index, capability-registration or unsupported refusals.
The execution receipt retains the admitted plan and executor observations.
Use inline execution for supported synchronous reads; an asynchronous requirement
returns its typed refusal when native asynchronous support is unavailable.

### Retain privileged read authority when restoring a program

The composition root calls `application_installation::WorthQueryInterimCheckpointRestore::program_with_projection`
on `worth-query-host` to receive the restored runtime and its
`WorthQueryApplicationInvariantProjectionAuthority`. This is the same read-only
capability retained during bootstrap; it grants no initial-state authoring on
restore. Ordinary runtime holders cannot mint it. Restoration and admission use
the existing checkpoint owner and resource limits.

### Inspect checkpoints and rejected-proposal reservations

`WorthQueryPrimaryGraphApplicationRuntime::inspect_application_checkpoint`
accepts an advancement phase and checkpoint bytes. It returns a typed
`WorthQueryApplicationCheckpointInspection`: exact non-allocator model value
bytes, typed allocator frontiers/high-waters and the live entity/relation census.
Query framing and native fields/arenas have their single codec owners. This is
reconstructive diagnostic inspection, not recovery admission or mutation
permission. Unknown fields fail closed; codec projections exhaustively classify
their actual decoded types. Ordinary reads should use query/projection APIs.

`observe_record_reservations` on that runtime starts an owned, bounded observer
for actual Native reservations in this runtime on the current thread. Drop ends
it, including on unwind. Its explicit diagnostic allocation policy funds fixed
backing; `report(policy)` seals a funded immutable prefix. Reports retain entity
and relation coordinates even when the proposal refuses. Capacity, request and
physical allocation failures return typed incomplete-evidence failures; no
partial report is certified as complete. Other runtimes/threads are excluded.
Observer recording costs O(active observers) per reservation; starting reserves
O(maximum records), and sealing a report copies O(observed records). The observer
never changes allocation or publication semantics. Scope it around the proposal
being diagnosed; concurrent proposals on other threads are independent.

`CheckpointAllocatorInspection::verify_reported_advance(after, reservations)`
owns the allocator comparison for both Native and Query consumers. Pass the
before/after decoded frontier and generation slices and the complete observer
report, converting records with `CheckpointRecordIdentity::from`. It accepts
only contiguous appends and one generation step per reservation; duplicate,
missing, backward or unreported coordinates return a typed
`CheckpointReservationAdvanceDenial`. It is reconstructive diagnostic work,
O((A+R) log(A+R)) time and O(A+R) scratch. Applications separately compare their
model bytes; they must not reimplement allocator arithmetic.

Checkpoint restore deliberately refuses unknown fields in the Native checkpoint,
allocator, partition, arena, frontier and generation rows as `CorruptCheckpoint`
before publishing any state. Inspection uses that same closed grammar. Current
and supported legacy writers emit only their declared fields, so their bytes
remain accepted; no checkpoint version was reinterpreted or migrated. A payload
with extra fields that an older reader happened to ignore is outside this
compatibility window and now refuses. Future field additions require an explicit
protocol/version admission decision rather than silent forward compatibility.

### Whole-aspect restored support

Recorded correction supports genuine `SourceAspectRevision` witnesses. Native
resolves the complete installed scalar or struct field roster at both exact
full-generation roots and compares complete aspect presence/content under
compatible entity kind, binding and contract. Query selects Lifecycle, WholeAspect
and every actual Field coordinate, including authored no-ops. Strict original
revision verification and canonical performed cancellation remain required;
matching values alone do not authorize restoration. Unsupported shapes and
unavailable canonical evidence retain typed refusals under the original request.


### Native public-only Source maintenance

The Native compiler substrate can register an exact schema-bound program with
`RelationalRuntimeBuilder::managed_index_source` or the initial installer's
`install_with_managed_sources`, without inventing an invariant. The receiving
registry validates actual membership/projection owners and derives structural
access; original request controls and the existing Source participant own all
maintenance. Live installation receipts bind the complete Source roster through
checkpoint readmission; Source-affected codec transitions remain unsupported.
Query bindings can request `ApplicationManagedIndexMaintenance::Native { cold }`
through their defaulted `MAINTENANCE` constant and existing provider installation
entrance. One combined roster supplies public and invariant audiences; Refuse
wins. Explicit portable v5 supports optional refinement; default v3/v4 meanings
remain unchanged. One-shot Parents can use `ApplicationQueryBasisSupport::current_only()`;
explicit retained reads refuse before provider execution. See the
[public maintenance plan](../plans/WORTH-query/milestone-9.19-phase5-query-maintenance-plan.md)
and [authoring guide](build-an-application.md#construct-committed-provider-indexes).
Bank's current-account Parent is qualified on its existing v1 Native Source.
Actual topology face discovery remains unfinished. The topology entry's existing
positive-turn invariant now uses the shared exact integer orientation predicate
over its full u64 metre/model-local domain; opposite and zero turns retain typed
invariant refusal. This numerical prerequisite does not supply spatial discovery.
See the [orientation evidence](../plans/WORTH-query/milestone-9.19-phase5-topology-integer-orientation-evidence.md).

The existing Query-free box/tree/segment-contact owner supports the sealed signed
fixture and full-u64 Metre/model-local coordinate classes through one typed
`IntegerBox2<Space>` implementation. Cross-domain operations are type errors;
a unit/frame class alone does not establish model-instance affinity. The concrete
`TopologyLengthBinding` also retains its original UInt64/nonzero wire meaning and
supplies bounded admitted decoding with the original refusal. These generic fixes
remain qualified independently of the removed public Scene/Face application.

Phase5 qualifies Bank and the existing bounded box/prefix provider contracts.
Real CAD geometry/topology/contact integration and actual-face scale acceptance
belong to Phase6 in the private application. See the
[public cleanup evidence](../plans/WORTH-query/milestone-9.19-phase5-public-cad-cleanup-evidence.md).
The [Phase5 contract matrix](../plans/WORTH-query/milestone-9.19-phase5-closure.md)
records the qualified public boundary, evidence provenance and remaining private gate.

### Native managed Source impact history

Native `authored_managed_source_selector` accepts the actual installed
Source program and full-generation scope; use
`authored_managed_source_history_cursor` with genuine original/selected bases
and the existing staged page lifecycle. This is complete conservative canonical
impact-candidate history, including actual field/whole-aspect no-ops, endpoint
writes and transient membership, unioned with actual physical publication changes.
Existing scalar-valued Struct fields and direct/Reverse endpoint routes are
supported. Shared-row neighbors and disjoint fields on declared field kinds can
remain conservative candidates; membership-only programs omit entity scalar-only
commands. It supplies no exact Query
relevance, READ grant or correction cancellation. Arbitrary scalar-aspect/deeper Source reads, merges and schema/materialization
transitions are not covered. Live envelope
custody is required: deep clone, decoded recovery and journal settlement cannot
recreate sidecar issuance, and missing metadata refuses as
`IncompleteManagedSourceHistory`. See the [Native API contract](../crates/worth-relational/API_OVERVIEW.md#managed-source-authored-impact-candidates)
and [slice plan](../plans/WORTH-query/milestone-9.19-phase4-membership-history-plan.md).

### Qualified managed Source correction

Qualified retained managed Source facts join ordinary recorded correction through
existing receipt selection and preparation. Actual Native scope candidates for
Public Parents can be narrowed by the same installed forward predicate on both
historical affected row images. Opaque acquisition/use/disposal happens outside
runtime/attempt gates under the original request. Exact refinement never excludes
matching history. Missing/expired material and governed Parents stay conservative;
wrong association refuses. Ordinary history, performed cancellation, strict
original and complete target support remain independent requirements. No historical
READ is granted. Prefix and indirect box/contact correction use the same generic
history consumer; governed historical relevance stays conservative. See
the [correction contract](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#qualified-managed-source-support).

### Native historical Source material receipts

An actual Source observation may call `authored_history_impact` on a genuine
canonical occurrence. The sealed view validates actual Native ownership and
carries affected rows plus funded weak before/after material receipts. Opaque
payload acquisition/use/release occurs outside all runtime/attempt gates; expiry
is conservative. This admits no historical READ. Query's qualified Public
correction consumer interprets this metadata at its outside-guard boundary. See the [Native API](../crates/worth-relational/API_OVERVIEW.md#historical-source-material-metadata)
and [foundation plan](../plans/WORTH-query/milestone-9.19-phase4-historical-candidate-plan.md).

### Retain an accepted physical Store close

Platform workers already inside an admitted kernel can retain its immutable
original cancellation lineage and absolute deadline through
`MapKernelContext::retain_interruption_view(original_lease)`. Use the original
borrowed lease for leased execution or `None` for serial execution. The move-only
`ExecutionInterruptionView` supports `Send + Sync + 'static` ownership, including
sharing the same view through `Arc`; `check_live()` grants no fresh execution,
memory or work allowance. Its admitted backing stays charged until its final
owner drops. See [Execution retained controls](../crates/worth-execution/README.md#original-request-retained-storage).

The Store physical owner can start `PhysicalStoreCloseTask::start(serving)`.
The one owned termination worker consumes the actual serving runtime and runs
the existing close plan. Spawn refusal returns that runtime unstarted.
`poll(self)` returns the same pending task, an actual close outcome, or owning
inspect/reopen-only failure evidence. Borrowed readiness/progress is descriptive.
Consult the original execution request before start and only after a pending
poll; an observed terminal result wins over a late stop. Keep a pending task in
recovery custody rather than claiming the Home remains serving.

Dropping observation does not cancel accepted close. The worker retains its
owners through normal close, including blocking drains and media release.
Spawn, allocation and a finished-worker join have no hard deadline bound. Panic
or lost outcome never proves drainage, lease fate or successful close; retain
any actual outcome and progress for inspection and normal recovery.

### Own an anchored publication one physical step at a time

`AnchoredRecords::start_publication(publication, interruption_view)` admits an
owned publication without starting a worker. Retain its recovery custody
outside a disposable execution result before calling consuming `step`.
`Pending` returns the same operation; `Ready` permits caller preparation of the
next existing batch. Only the final actual receipt describes the selected
anchored value. Admission hashes input and may read the current value; locks,
allocation, I/O and physical scheduling remain cooperative safe points.

Each accepted step registers its actual identity before worker start and holds
its owner's selection/input-memory protection through terminal delivery. The
caller transfers the real existing input reservations into that guard; it does
not issue another budget. Ordinary synchronous `publish` drives these same
phases using its existing Signal deadline. A genuine Execution interruption
view additionally observes the original cancellation lineage and absolute
clock at pre-effect and group seal, without converting to Signal time.

Consuming `stop` permanently removes further advancement. Stopped custody may
inspect, explicitly cancel or settle the accepted step, preserving its actual
prefix/fate. Recovery cannot revive stopped or completed publication authority.
Observer loss does not cancel accepted mutation authority or certify a result.
Terminal descriptions retain facts, not live views, selection guards or Serving.


### Retain an ordered journal publication across an interrupted caller

`physical_runtime::OrderedJournal::start_publication(publication, original_view)`
returns a move-only operation and strong recovery. Move the encoder's existing
`ExecutionMemoryReservation` with `OrderedJournalPublication::with_input_memory`
when present. Store the recovery outside disposable execution results before
calling consuming `step`. `resume` returns the same recovery alongside its unique
Ready or Stopped endpoint; reinstall that recovery before lending the endpoint.
`Recovery::stop` permanently fences advancement. Consuming recovery `resume`
readmits a clean, fully visible actual terminal as `Committed`; descriptive
stopped settlement alone cannot mint that carrier. Poll actual settlement before
consulting a late caller stop. Only Store-issued `OrderedJournalCommitted` carries
an actual selected entry; source/terminal observations are descriptive.
The journal keeps its existing frame, spine, ordering and Signal policy. Reads,
encoding, allocation and short substrate locks remain cooperative rather than
hard real-time. Accepted current-step custody survives observer loss through the
existing physical mutation worker; no World acknowledgment runs in that worker.

### Compose World with an admitted application Home

Applications enter through `WorthQueryApplicationHomeRequest`; Store privately
composes the exact Native owner, InitialEmpty Signal root, Bridge baseline and
World. Fresh opening invokes the initializer once. Journal-only reopen admits
original completion, activation, published World basis and Native history without
seed replay. The same supported program/schema admits a new build UUID without
eager regeneration. See [install or reopen](build-an-application.md#4-install-or-reopen).

One real Store Application mutation lease permits one Directory writer.
Competing Host opens refuse `MutationOwnerContended`; process death frees that
lease for a new opener. The new incarnation refuses prior-open selectors,
retained capability access and provider-session grants with typed
`StaleIncarnation` before disclosure or provider effects. Obtain fresh authority
from the new runtime. An original recorded receipt is data for owner admission,
not a stale live handle or permission to replay a handler.

Fresh Memory and Directory Query publication use the same Native completion
commit after all seed batches and index construction. World genesis selects it.
Installed package/schema digests, the terminal seed receipt and aggregate counts
describe completion; they grant no readmission authority or Memory durable
association. The activation row separately binds the program. Failed Directory
completion retains the original preparation and Home without retrying accepted
seeds. Retired development completion formats return a typed Query recovery
refusal before service, without migration.

#### Checkpoint admission — pending (owner P-A, D12)

#### Recovery after a selected checkpoint — pending (owner P-A, D12)

#### Development workflow with program transitions — pending (owner P-C, D16)

Store Application's `readmit_world` consumes the concrete Native-readmitted
Home and an `ApplicationWorldDefinition`. The private attachment injects the
same actual Native runtime into World and Bridge. The resulting
`WorldReadmittedApplicationHome::world()` borrows existing publication,
branch/history observation, inspection and recovery ports; only Home owns
World's lifecycle. Use that Home's `close` and retain its actual refusal or
recovery custody for retry. Accepted World journal tasks stay in the same
Home custody through physical teardown. Close stops and settles accepted work;
unsettled work refuses close. Terminal indeterminate evidence permits teardown
and reopen, while keeping runtime publication admission fenced.

This lower composition seam requires the actual Native recovery authority and
its complete associated-effect inventory. Failed admission returns
`WorldHomeReadmissionRefused`, retaining the same concrete Home, lease and World
slot; `into_home` explicitly cleans up refused component custody and returns
that Native-readmitted Home. Attached memory bootstrap, legacy publication and
branch mutation doors refuse before effects. Query's closed
`query_publication_home_binding(expected)` selector admits the actual live owner
and exact process identity, then checks its retained Home binding. Only genuine
memory ownership returns `None`; unavailable, foreign or inconsistent ownership
returns its typed refusal before any I/O. Publication still authenticates through
its original effectful preparation doors.
Live directory preparation abandonment retains its original World and Query
custody and offers `Inspect` plus `ReopenRequired`. Its live release/continuation
also returns `RuntimeWorldRecoveryDenial::ReopenRequired`; neither a zero effect
count nor an empty live inventory proves no effect. End current Home/process
ownership and reopen under Store's mutation lease. The closed pre-service D8
recipe supplies the actual Native recovery authority to World's complete
associated inventory admission before reconciliation, including empty World
journals. That authority checks its exact runtime; it is historical recovery
evidence, not a live publication fence. Recovered custody keeps its distinct
durable-before-live disposition operations. Query admits its actual original
key/intent/affinity before service; retry returns distinct recovered
`ProductUnpublished` custody. Inspect its original Native meaning and predecessor,
then use the fresh recovery claim's `continue_original_in_request` for the
original occurrence, or the existing release route for durable release. Neither
runs the handler again. Pending/refused outcomes retain that same owner. A
committed `Continued` permits original continuation and forbids release after
reopen; it is never appended twice.

World lifecycle `close(request, &actual_owner)` uses the caller's original request, nonwaiting
admission, and the existing publication frontier. Before sealing, contention or
interruption preserves Open. After sealing, the same retained task owns each
extracted head, its populated history slot, retirement phases and report;
interruption preserves Closing. Ordinary weak ports stay unavailable while the
close-only weak port can retry that same task. Its authentic same-owner borrow
keeps the actual root alive through return; a foreign owner is refused without
weak upgrade or root cloning. Unresolved business, pending
bootstrap and owner-created component retirement custody refuse before sealing.
This minimum also refuses Directory custody without a qualified durable handoff;
it does not claim full request-aware Home or Native snapshot cleanup.

The consuming `seal_world` retains the actual World close report and advances
the same World-owned result cleanup before dropping World. Initialization uses
original-request checks and trylocked constant-size Closed/catalog admission.
Each cleanup advance visits one sealed history slot or advances Native's genuine
original-result retirement task with the actual Home Native owner borrowed.
Busy, stop, Native refusal and observer loss retain the same cursor, result and
original release-cell custody; counts describe observed Native Complete outcomes.
Recovered publication capabilities never become synthetic live result tasks.
Final World/Bridge/Native destruction, materialized Native cache custody and cold
capability retirement retain their separate contracts; this is not full Home D11. A refused seal
retries through `try_seal_world`. Host can then release its Query observations
before `finish_close` attempts Native teardown. Shared conditional assembly
custody returns `WorldCompositionRetained` in the same Home recovery object;
retry uses actual ownership admission and runs conditional cleanup outside its
assembly lock. A sealed World never resumes Serving.
`publication_port().begin_durable_bootstrap_root(request, creation, native_basis,
signal_basis)` validates the actual current Signal owner and supplies the same
private Bridge's baseline. Its retained bootstrap can be advanced through
`advance_durable_bootstrap_root(request)`. The existing bootstrap state keeps the
original pre-install history, pins and branch cell across pending calls. World
artifact v3 requires Native's explicit recorded component, Signal's owner-proven
InitialEmpty recipe, Bridge's versioned registered-source baseline and the actual
program revision; v1/v2 and missing seed fields refuse. General authored Signal
or installed Bridge state refuses before this narrow bootstrap. Root installation follows the actual
sealed journal acknowledgement. The filesystem Home court covers this bootstrap,
ordinary Relational-only publication and exact World-owned result snapshot
cleanup. Undelivered publication refuses Home close before teardown; recover
its original delivery through `recovery_port().recover_performed` and consume it
before retrying. Independent Native reads retain their own close obligations.

World's durable open retains its original identities and one Native preparation,
and its prepared wrapper executes through `execute_durable_without_signal`.
The original occurrence backing is admitted before Native effects, then filled
only with the original settled receipt, position and artifact digest. Actual
journal selection authorizes the same cell's live CAS; pending retains the
original ready custody and capacities. Attached checkpoint
refuses `WorldSectionRequired`; close reports that checkpoint failure. Native-only
reopened Home checkpoint also refuses before capture when actual World journal
heads exist. This presence check grants no World proof from empty streams;
already-accepted checkpoint recovery still settles and actually acknowledges
its original task before Native close admission.

Cold readmission supports one initial product branch and its ordinary
Relational-only occurrence chain. Each occurrence admits its exact original
Native predecessor and successor from actual recovery; later Native advancement
does not replace either historical root. World validates the actual journal
positions, payload digests, association and recorded Native evidence, restores
the original durable identities and head generations, and issues fresh local
owner custody. `observation_port().readmitted_publication` takes a Foundational
`DurableCompositeCommitIdentity` as lookup data and returns an owning protected
view of that original occurrence and Native semantic result. It exposes no live
`CommitResult`, snapshot, timing or `Performed` delivery; Native semantics do not
reconstruct arbitrary application return values. Native source-backed recovery
also reconstructs the exact predecessor and unpublished effect;
`inspection_port().recovery_page` exposes a Recovered
row, `recovery_port().inspect_recovered_effects` claims its original identities,
and `release_recovered_effects` durably records release before dropping the real
Native capability. Pending or refused release returns the same custody.
Released artifacts require explicit no-effect evidence or the complete original
Native receipt, position and digest, checked against Native's actual inventory
on reopen. The real filesystem child-process court kills after Native durability
before the World occurrence, reopens and releases the original custody, then
reopens with the same unmoved product head. The ordinary occurrence filesystem
court additionally publishes twice, advances Native again, closes, reopens both
historical semantic results and the original product head, then reopens again.
Its bounded history-capacity control retains the actual partial catalog,
admitted Native capabilities and same Home lease until explicit cleanup. The
existing kill child also covers a later unpublished Native effect whose exact
World predecessor is generation 2; release and second reopen preserve that head.
Multiple-branch readmission, recovered continuation, general Signal/Bridge state, Host retry and World
checkpoint coverage remain required for phase completion.
See [Milestone 3](../plans/database-foundation/milestone-3.md)
and the owning [Home attachment](../workspaces/worth-store/crates/worth-store-application/src/world_section/readmission.rs).

### Read one protected physical checkpoint cut

Use `ServingPhysicalRuntime::try_records_for_checkpoint(original_request)` to
retain one actual reader without waiting on capture locks. Store owner adapters
then use `AnchoredRecords::open_payload_from` and
`OrderedJournal::{head_position_from,describe_entry_from,open_payload_from}`
with that same reader; `describe_entry_from` reads funded header metadata, not
verified payload bytes;
newer physical roots cannot expand its selected anchor or journal heads.
Payload streams retain the same request and stop after a refusal. Only actual
verified EOF validates payload bytes; pre-EOF sizes and coordinates are
metadata. Anchor EOF additionally validates the full anchor-record SHA. Consuming
`into_verified_membership(original_reader)` then moves both original resources
into opaque, cheaply Clone `AnchoredRecordReadMembership`. Controlled open
pre-reserves its fixed Arc metadata before I/O and table growth; conversion
performs ONE already-funded allocation, with no fresh request consultation.
Refusal returns both same resources. The table and metadata grants remain
inside its PRIVATE shared allocation; no public Arc/Weak/unwrap exists.
`validate_read_membership` checks its owner, original root and live protection.
`open_read_member` selects an actual table index in O(1), and journal
`head_position_from_read`, `describe_entry_from_read` and
`open_payload_from_read` use the same private reader. No latest-root recapture
or raw-reader getter participates. Generic disposal is unbudgeted.
For request-aware release, consume that exact membership with `into_retirement`,
then advance its `AnchoredRecordReadRetirement` against the ORIGINAL Serving
owner and original request. Foreign affinity and Busy/stop preserve the same
task. A nonlast local share releases only itself; a unique backing discharges
its actual EOF frame, operation reservation, C.10 references and lifecycle
counts in retained stages. `into_retired` yields inert original data and grants
for explicit unbudgeted `reclaim`; retain them through actual physical close.
This component does not retire another reader, Native, Query or a whole Home.
Historical journal linkage uses protected physical identities and frame
integrity, with no transitive journal-hash promise. Callers admit their own
payload storage and perform I/O outside logical reservations. This physical
producer grants no World reconciliation, Native complete-history image or Home
close authority. See [protected physical reads](../workspaces/worth-store/crates/worth-store/README.md#protected-physical-reads).

### Prepare an initial Signal stop under actual World close admission

Platform composition uses the original owning Signal runtime's
`try_prepare_initial_empty_stop` and the actual World owner's
`try_prepare_close_admission`. Signal preparation is reversible; World guard
Drop preserves Open. After Signal preparation succeeds, infallible World
`begin()` commits the existing close task before owner gates release. Signal
commit retains its original stopped allocation until explicit unbudgeted
reclaim. These owner primitives neither invoke general Signal cleanup nor
certify Bridge/Native/Store or full Host close. See the
[Signal owner workflow](../crates/worth-signal/README.md#retain-an-initial-empty-owner-during-composite-close)
and [World lifecycle](../crates/worth-runtime-world/README.md).

Before supplier preflight, World owners can use
`try_preflight_close_admission(original_request)` for a task-free observation of
the same actual admission gates. It releases every guard before returning;
final preparation reacquires the gates and validates the history frontier.
The actual close task retains its original-request metadata funding, and an
allocation refusal preserves `RuntimeWorldCloseDenial::Execution` rather than
being collapsed into cancellation. These are owner-component guarantees, not
qualification of consuming Query, Directory Native or full Host retirement.


### Read original Native bytes from a protected source cut

Native `RelationalDurableReadProtection` binds the original provider allocation
and frozen canonical/transition heads. Store recovery uses one physical reader
and its verified selected member table; owned reads admit the original request's
exact byte output before growth and require verified EOF. Unsupported providers
refuse without legacy `Vec` fallback. The ordinary protected source assembly
feeds existing Native frame and semantic readmission. It does not certify a
Complete checkpoint or fund recovery-plan/decoded-transition collections. See
[Native protected source reads](../crates/worth-relational/README.md#read-an-original-protected-source-cut)
and [Store provider ownership](../workspaces/worth-store/crates/worth-store-relational/README.md#protected-native-source-reads).


### Export the original Memory publication artifact

New Native Memory publications retain their original v4 artifact in the existing
residency cell, admitted under the actual publication request before head/position
cutover. The existing `canonical_commit_artifacts` iterator verifies and shares
those bytes rather than encoding another frame. Legacy authoritative tuple
length/digest observations are unchanged and no longer retain tuple bytes.
Memory publication pays the declared frame encoding/verification cost. CHK18
capture uses those original frames alongside protected Journal or selected-member
bytes, retaining complete indexed canonical coverage. See
[original Memory frames](../crates/worth-relational/README.md#retain-original-memory-publication-frames).


### Read covered canonical history from selected Native members

CHK18 serialized checkpoints require a complete ordered inventory of original
artifact v4 members. Covered head, historical, component and associated-effect
admission use the selected member directory and actual protected reader rather
than covered journal bytes. Uncovered tails keep their ordinary recovery path.
Original-request artifact export remains lazy and shares retained Memory frames.

The same prepared Home attempt retains interrupted placement transfer and old-read
retirement; cadence advances after required acknowledgement completes. Initial
selection guards and original source affinity remain in force. New scalable
owners retain their own request reservations; this does not qualify Complete,
service-free bare reconstruction or whole-provider close. See [selected canonical
history](../crates/worth-relational/README.md#canonical-checkpoint-members)
and [Store selected replacement](../workspaces/worth-store/crates/worth-store-relational/README.md#replace-selected-canonical-backing-in-the-same-home-attempt).

### Preserve original Native data when opening a new Home

Directory Host opening uses Store's actual physical new/existing disposition.
A genuinely new Home retains the original Native runtime and its installed
schema data through first admission; the existing-Home route reconstructs
Native from durable sources. An empty existing Home never becomes a fresh
initialization opportunity. Query index registration and fresh World attachment
consume the same original Native receipt. Source and preparation refusals keep
the actual Home custody so cleanup or retry acts on that owner.

Applications continue using the Host Home request. The restricted Native and
World admission ports are platform composition surfaces. Public Host opening
returns after genuine bootstrap publication, so its first manual checkpoint
is post-bootstrap. This fresh-data contract does not qualify selected Complete
Home reconstruction or physical Directory close. See [Native fresh admission](../crates/worth-relational/README.md#admit-the-original-fresh-home-source)
and [World fresh attachment](../crates/worth-runtime-world/README.md#attach-a-genuinely-fresh-home).

### Read the Bank current account roster through a managed Source

Bank's `queries::current_accounts()` is a current-only managed Parent over the
unchanged v1 account-discovery binding. It uses the actual principal mapping and
`DiscoverOwnAccounts` ability, with personal ownership, authorized-user and
business-owner paths deduplicated by account. The existing `queries::accounts()`
and its pinned reads retain their contract. All-account output visits every
matching Source row; this does not claim selective pruning or continuation.
The Bank contribution installs its finite provider through the ordinary
configuration owner. Actual grant/revoke and creation maintain stable occurrence
slots without rebuilding warm roots. See the [Bank declaration](../workspaces/worth-query-bank-world/crates/bank-domain/src/queries/current_accounts.rs),
[provider](../workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/current_accounts/provider.rs)
and [ordinary receiver courts](../workspaces/worth-query-bank-world/crates/worth-query-bank-court-external-rail-server/tests/worth_query_bank_court_external_rail_server/server/ordinary_reads/current_accounts/mod.rs).

Bank account activity accepts recorded original receipt data through
`BankAccountActivityRequestForPrincipal::historical_original`. It uses Query
`at_original_receipt` with fresh authorization and bounded current owner history;
receipt data cannot recreate a performed owner or revive unavailable history.

## Browser signal facade

The JavaScript `worth-signals-wasm` facade declares browser endpoint resources.
[Route authoring](../crates/worth-signal-wasm/docs/api-reference/route-authoring.md)
includes explicit `PATCH` transport for update semantics, with canonical request
identity and mutation-response reconciliation proofs. This is the browser signal
package surface, separate from the native query facades indexed above.
