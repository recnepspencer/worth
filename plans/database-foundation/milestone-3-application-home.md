# Application home ownership batch

Authority: Milestone 3 D1–D3, D6–D8, D11–D14 and D17, with the shared roadmap
requirements. This follows the independently approved publication prerequisite
at `00174a3445`. Phase 3 and Phase 4 land together, after coordinator
certification. No API below is claimed implemented yet.

The batch transfers actual physical admission, binding, incarnation,
checkpoint selection and close to `worth-store-application`. The first proof
is a real filesystem open, commit, checkpoint, close, reopen and continued
commit journey. Bulk Query constructor migration follows that owner journey.

## Boundary brief

The adversarial case is a crash after a recovery basis is durable but before
every owner has selected it. A second selector, dropped lease, fabricated
admission proof or forgotten accepted work would make reopen dishonest.

Verified current seams:

- StoreRelational `home.rs` opens media, builds the native runtime, then derives
  the descriptor binding from that runtime. Binding currently follows build.
- `physical_store.rs` and `physical_store/media_admission.rs` preserve held
  recovery media through initialization/recovery and serving handoff. Move
  that custody path, without a parallel location check or lease reacquisition.
- `home_descriptor.rs` owns `home`, `WSRHOME1` version 2 and its reserved tagged
  bindings. It records namespace identity, incarnation and runtime/profile,
  but no application program binding.
- `checkpoint_record.rs` selects `relational.checkpoint`, prepares members,
  holds their fence, publishes and selects. `cold_source.rs` also names that
  anchor. Both selectors must change. The `WSRCKPT1` version-3 native section
  remains unchanged.
- `home/close.rs` drops runtime/cold source before checking Store uniqueness
  and can return bare `StillShared`. Renaming that result cannot restore a
  usable refused-close posture.
- Query Execution's `application_installation/construction.rs` admits package,
  installed schema and program roster before calling `builder.build()`.
  Composition must supply the one readmitted runtime there, rather than keep
  a home beside an independently built memory runtime.
- Store physical-format already depends on Foundational and Proof. Query
  Installation already owns the closed support roster; program revision
  minting remains private to validated declaration meaning.

## Destination and ports

Store Application owns physical opening, descriptor, versioned journal
registry, recovery progression, `home.checkpoint`, triggers, resolve routing
and close. Split those responsibilities into named modules at or below 400
lines. Delete the old home type/request/lifecycle exports; retain no wrapper.

### Destination topology

The primary axis is authority owner (Application versus Relational versus
Query), then lifecycle operation, then the owner-specific step. Physical
opening, application binding and native section meaning never share a file.
The destination is:

```text
workspaces/worth-store/crates/worth-store-application/
  src/lib.rs                       # module wiring; facade re-export only
  src/facade.rs                    # stable composition audience
  src/home_context.rs              # owned lease and live owner custody
  src/home_request.rs              # admitted opening inputs
  src/registry.rs                  # exact journal/anchor declaration
  src/resolve.rs                   # owner resolution routing
  src/open/{mod,physical_admission,descriptor_admission}.rs
  src/open/readmission/{mod,relational,signal_bridge,world,query,custody,transition}.rs
  src/descriptor/{mod,binding,incarnation,codec}.rs
  src/checkpoint/{mod,manifest,publication,triggers}.rs
  src/close/{mod,admission,posture}.rs
workspaces/worth-store/crates/worth-store-relational/
  src/lib.rs                       # stable native-section/medium facade
  src/section/{mod,attachment,readmission,checkpoint,codec,selection}.rs
  src/{journal_medium,recovered_history,recovered_transitions}.rs
  src/cold_source.rs               # existing placement/demand owner
  src/cold_source/                 # existing member custody and its courts
workspaces/worth-store/crates/worth-store-application-oracle/
  tests/worth_store_application_oracle.rs
  tests/worth_store_application_oracle/
    {home_lifecycle,checkpoint,relational_section,transition,retry,process}/
```

Braced filenames declare separate responsibility files, not bucket modules.
Only implemented owners/operations get bodies; later phases extend the
declared slots rather than adding dead placeholder variants. Parent `mod.rs`
files wire or compose their one operation and stay within the line cap.
Oracle cases split by the behavior they dispute; reference model, schema,
batch authoring and process dispatch have their own named owners.

Shared value definitions live in
`crates/worth-foundational/src/durability/` by meaning: home identity,
incarnation, attempt association, World identity family, owner family and
retention class. Existing publication-fate vocabulary stays canonical.
`crates/worth-proof/src/home_recovery/` has one file per named D8 proof.
Both use their existing crate facades; no runtime counter or live table moves
into either substrate.

Query Installation's admitted binding belongs in
`src/application_home/{mod,binding}.rs`, exported through its facade. Query
Execution's preparation/completion stay inside the existing
`src/domain_computation/primary_graph/application_installation/` owner, split
into `preparation.rs`, `completion.rs` and the existing construction core as
needed. Its audience facade remains `src/facade/application_installation.rs`.
Host gains `src/application_home/{mod,request,open,memory_only,directory,
interim_checkpoint,discard}.rs`; only that production composition subtree
names Store Application. Host's existing `src/facade.rs` exports Host-owned
request/results, not Store internals. These later Query/Host edits follow the
owner journey and may not become a competing constructor implementation.

Forbidden placements: a flat `home.rs`/`application.rs` containing all owners
and phases; catch-all `helpers`, `util`, `common` or test `support` bags; owner
codecs in the home manifest module; cold-source internals in public facades;
Store composition in other Query crates; or old-home/checkpoint adapters kept
beside the new owner. A new readmission owner extends its named step and the
typed progression constructor, not a boolean-driven generic recovery bucket.

StoreRelational retains journal placement and cold demand reads. Its narrow
section carrier hides the cold-source implementation and custody registry. It
attaches one medium to the native builder, readmits its section and two tails,
and prepares native bytes/coverage/members while holding the actual capture
and member-selection fence. The application owner publishes the union once,
then installs selection and acknowledges capture. Copied bytes plus an
unbound later acknowledgement are insufficient.

Query Installation supplies a private-construction admitted binding derived
from the installed schema, selected validated program and closed support
roster. Store records descriptive addresses; their digests never mint program
or roster authority. Query Execution gains preparation/completion ports around
its existing core, allowing Host to supply the readmitted runtime. The
original bootstrap phase/request pays for setup and retains cancellation and
memory controls; installed runtime policy does not change.

Applications still import only Decl and Host. Only Host's production
composition module names Store Application. Apply D1's dependency/source
rules in `road1.toml`; Store Application depends on owner ports, never Host.
Keep physical-only Store packages off Query dependencies. D14 forwards only
Store's retained certification fault authority; no new feature vocabulary or
feature outside Store is introduced.

## Ordered implementation

1. Implement the shared contracts in
   [Durable owner contracts](milestone-3-durable-owner-contracts.md), with
   owner-private issuers and distinguishing compiler fences. Settle the actual
   interfaces and disjoint write owners before assigning any World worker.
2. Move physical opening and the registry. Preserve media qualification,
   root-schema refusal, continuous lease and typed contention. Only Store's
   lease holder determines newness; a partly initialized Store is not empty.
3. Admit declared binding before native construction; durably advance
   incarnation before service. Extend the descriptor's reserved tag list,
   preserving its envelope. A legacy Relational-only descriptor without
   application binding refuses explicitly: do not adopt it as the caller's
   program, invent its predecessor, or initialize over its history.
4. Extract section ports and remove both old checkpoint selectors. Read the
   application manifest once, let Relational readmit its section/tails, and
   retain the actual runtime. Move native home courts and the relational_home
   oracle into the application oracle's Relational section. Component courts
   use genuine native artifacts; they do not fabricate Query program binding
   or claim application service from a partial owner state.
5. Publish one home checkpoint with prepared sections and all their held
   fences. Select members and acknowledge native captures only after success.
   No-effect/indeterminate outcomes retain the prior recovery posture. Count,
   byte, explicit and close triggers use the same operation.
6. Close stops admission and consults actual owner settlement/custody within
   the supplied deadline. A pre-teardown refusal returns the same serving
   home. A checkpoint failure is reported in `Closed`, not a close refusal.
   If teardown reaches a recovery posture, return owned lease/recovery custody
   and concrete next actions; never drop the runtime and claim it still
   serves. Arc counts are not evidence that accepted work settled.
7. Prove the complete owner filesystem journey and distinguishing refusal
   cases. An incomplete D8 progression remains a typed opening state. It must
   not mint World/Query proofs or expose a serving application facade.
8. Complete Host service composition with real Phase-4 readmission. Manually
   migrate one ordinary, rostered, trusted-time, checkpoint, transition and
   advancement caller. Only then use existing codemod infrastructure for
   recognized lossless argument mappings. Preserve authority, roster, time,
   build identity, initialization and transition inputs; unfamiliar forms
   remain manual. The interim checkpoint caller subset is frozen explicitly.

## Ownership and handoff

The Phase 3 implementation agent owns the new Store Application/Application
Oracle crates, Store manifest
and lock membership, narrow boundary entries, StoreRelational lifecycle and
section extraction, required Query binding/preparation ports, and later Host
composition/docs/representative caller migrations. Shared Foundation/Proof
contracts have one writer. The main chat's Root coordinates worker boundaries,
measurement windows, integration and final landing certification; it is not
the implementation owner named above. B4 Python issuance/selection/compiler-reader work
and unrelated physical testkit changes remain excluded.

The candidate future World slice is described in the shared-contract document.
It is not assigned here. No bulk migration or World implementation begins on
an unsettled interface. The lost-response/kill/reopen/original-result retry
journey remains the joint Phase 3/4 gate.

## Acceptance and verification

| Requirement | Distinguishing evidence |
|---|---|
| D1/D8 one runtime | Composition consumes the readmitted runtime; no second memory runtime is constructed. |
| D7 newness/lease | Interrupted initialization is completed/refused honestly; competing writable opens have one winner and typed contention. |
| D7 binding/fencing | Foreign/missing binding refuses before native build; incarnation advances before service; old authority refuses while old durable data resolves. |
| D12 checkpoint | Only `home.checkpoint` selects recovery; failed publication retains prior basis; fences protect cold members. |
| D11 close | In-flight refusal returns a usable serving home; checkpoint failure is reported by successful close; recovery posture retains owned custody. |
| Shared semantics | Real open/commit/checkpoint/close/reopen preserves artifacts and graph state; recovery invokes no handler. |
| D17 progression | Missing, foreign and reused stage proofs fail compilation; partial opening cannot construct service. |

Use pinned checkout compiler/configuration and existing runners. Capture exact
command/head/state/times/exit/counts. Run affected checks and named cases once
per coherent batch, then only invalidated cases after correction. Run explicit
boundary/context/formatting/dirty-and-range caps at batch completion, with no
automatic per-write checks. Record actual affected build costs and distinguish
warm/incremental measurements from controlled clean comparisons.

Ordinary focused process correctness runs may overlap under Root coordination.
Only controlled timing comparisons need an exclusive agreed window. Do not
run ignored tests, Query-scale, worth-ui or unrelated exhaustive campaigns.
The full media/process oracle runs once at joint final certification after
its inputs are final. Each implementation batch gets fresh independent review.
