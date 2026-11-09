# D13: six incoming callers outside the frozen reach

Read-only independent boundary design for the six paths in
`master2-r13-interim-caller-membership.json`. No product edits, Cargo, builds,
tests, or final qualification approval. The frozen 54-row list is unchanged.

## Governing boundary and capability search

D13, milestone-3.md lines 526-531, permits one named interim door and says
callers are only removed. `seal_rules/factory_ownership.rs:87` makes
QueryExecution its sole owner. `seal_rules/interim_tests.rs` demonstrates that
another construction caller inventory cannot grant reach. A new caller cannot
be laundered through a frozen fixture, source factory, wrapper, or renamed door.

I first read the capability-index procedure and its entries for source-owned
publications, coupled outputs, Home restart, and owned-history navigation, then
the linked installation/transition guidance and the real Host interfaces.
Relevant index entries are at lines 569-574, 668, 681, and 684. The index's
Home composition section at 1540 and pending checkpoint/transition notices at
1564-1568 distinguish the supported durable lane from ordinary memory behavior.
Documentation is narrower than some current transition implementation; the
following conclusions use the actual implementation as well.

The public nearest route is real:

- `application_home/request.rs:70`: `WorthQueryApplicationHomeRequest::new`,
  taking location, actual program/roster/declaration/configuration, build,
  limits, and one initializer.
- `application_home/opening.rs:23`: `open_in_advancement`. Directory enters
  `opening/preparation.rs:34`, which passes the one Host support callback to
  `prepare_home_program`. It does not take an application checkpoint.
- `application_home/opening/memory.rs:1`: MemoryOnly is fresh composition and
  has no physical reopening frame or checkpoint import parameter.
- `application_home/transition/request.rs:40`: `with_program_transition`
  retains the true predecessor request and complete-plan author. Its opening
  requires Directory, and `transition/opening.rs:85-107` submits both successor
  and predecessor to the same support callback. It cannot evade the family gate.
- `application_home/checkpoint.rs:17,26,52`: checkpoint/recover/inspect act on
  the actual Directory owner; there is no Home API importing arbitrary bytes.

The support callback is the immediate concrete dependency, rather than an
undiscovered naming alternative:

1. `application_installation/prepared_meaning.rs:49` invokes
   `contracts.admit_home_seed(declaration, support)` before configuring a runtime.
2. `application_contribution/contracts/home_seed.rs:31` maps **any installed
   producer** to `SignalNodeDefinitions`. Host `application_home/support.rs:37`
   assigns that family phase 5.
3. `support.rs:47-54` assigns output-demand/source-custody, program-support,
   adoption and output-read custody to phase 6. At `support.rs:130`, only phases
   through 4 pass. This is not a per-fixture policy: the callback applies to
   Directory fresh open, existing open, and both sides of program transition.
4. Operation admission is separately enforced by
   `execution_runtime/product_world/durable_admission.rs:32-75` and
   `application_runtime/durable_admission.rs:9-70`. Required-output capabilities
   require phase 6; a correction-bound operation additionally requires retained
   correction material (phase 9). Removing only the seed refusal would therefore
   neither implement nor admit the full journeys.

All six fixture families install producers. They fail the phase-5 seed
requirement before their proposed Directory journeys can start. Their required
output behavior subsequently depends on phase 6. Co-anchor/coupled correction
sequences additionally execute correction-bound operations. No support grant
flip is justified by this review: authored definitions, reconstruction,
currentness, custody and close must actually be implemented and qualified by
their owning phases.

## Per-caller disposition

The paths below are relative to `workspaces/worth-query/crates/`.

| Caller and introducing commit | Actual retained claim | Nearest lawful route; what can move now | Concrete remaining dependency |
|---|---|---|---|
| `worth-query-court-decl-host-authored-ownership/tests/authored_ownership/co_history.rs:13,120`; added by `cb6e44d70cb9094a99f9a4e9b204be2ce4723f4e` | Create real source/co-anchor and separate producer realizations; retire and reverse through Query; cold corruption of a recorded co-anchor must reach the actual authored-closure owner, return `CorruptPublication` naming the source, preserve unrelated records, and have a clean twin. | Native owns recorded birth/provenance and authored cascade admission. A focused Native recovery/closure court can use a genuine Native publication, capture, untrusted corrupted historical input and `prepare_authored_source_retirement`; assert exact source subject, clean twin and unchanged basis. Existing Native `authored_retirement/restoration.rs` and `corruption_subjects.rs` establish the real issuance/readmission pattern. Keep ordinary Query correction/closure integration coverage separately. | The **whole cold Query journey** needs Directory producer admission (H5), required-output persistence/readmission (H6), and correction material (H9). Native-only proof preserves the Native diagnostic invariant but is not the same Query cold integration claim. |
| `worth-query-court-decl-host-authored-ownership/tests/authored_ownership/cold_owned_revision.rs:10,115`; added by `b3830a0e600341a3bc4c8c51e9414233629fd01d` | A real ordinary performed source revision settles its declared required body before capture; after host loss/reopen, the next revision settles, removes old output 103 and exposes 104. The fixture deliberately uses worker request semantics. | Public Directory close/open with the same actual RequiredProgram, schema/configuration/build and a once-only initializer is the correct destination. Existing Home request supplies all ordinary semantic inputs. Native can protect its own output-retirement/currentness pieces, but it cannot execute Query's performed source and required output continuation. | Genuine H5/H6 Home producer/required-output readmission is absent at the gates above. A fresh MemoryOnly install or reconstructed literal outputs would erase the cold-reopen claim. No current equivalent public capability was found. |
| `worth-query-court-decl-host-authored-ownership/tests/authored_ownership/coupled/history.rs:8,106`; added by `cb6e44d70cb9094a99f9a4e9b204be2ce4723f4e` (later modified by `2a52026a4a383c1d5e807ff627678aae5a319e8f`) | Two actual writers share one coupled publication. A historical twin makes both claim the same recorded edge. After reopen, retirement must refuse at the closure owner as `CorruptPublication`, name the actual source, and leave a sentinel usable; clean history admits the same candidate. | Duplicate-edge custody is a Native canonical group/closure invariant. It can move to Native's genuine installed coupled publication/recovery boundary, with only an untrusted input twin changed and no manufactured authority. `history/data/codec_retirement_closure/source_scope_closure/coupled.rs` owns the historical group check; the real Native publication pipeline already owns coupled publication. Preserve Query's current coupled Preserve integration elsewhere. | Directory producer/required-output and correction gates prevent the full cold Query route. A Native diagnostic court is a narrower honest proof, not automatic permission to delete the incoming Query integration claim. |
| `worth-query-court-decl-host-authored-ownership/tests/authored_ownership/current_roster.rs:19,127,153`; added by `cb6e44d70cb9094a99f9a4e9b204be2ce4723f4e` (framing changed by `893f1e22d4dd115b50b9ddbd9a7a43fbd33d4ea2`) | Changes 0/1/2 must reject dropped/added/repointed current routes during real reopen with the exact typed owned-recovery subject. Change 9 must survive frame admission and fail the first actual ownership access/ordinary mutation as `CorruptCurrentOwnership`, with no record loss. A pre-roster prefix must never mean empty ownership. | Native owns current catalog/independent presence pairing. The owner already has `durability/data/native_checkpoint/roster_corruption.rs` and `publication/ownership/cold_checkpoint_source.rs`; `cold_pairing_court.rs` uses a genuine archive/canonical source and verifies a genuinely cold first access. Port the particular cases to Native full restore/access with real registered storage and installation authority. The pre-roster audience test can use public Query `inspect_application_checkpoint` and match `InspectionDenial::Native`/`UnsupportedLegacySemantics` without an interim constructor. | Current-roster lazy ownership validation is not merely codec inspection. Public inspection cannot substitute for the first mutation/access or claim issuance. A full Query cold variant still needs Home H5/H6 support. Native data-only corruption never supplies Query Original/Origin. |
| `worth-query-court-decl-host-owned-codec-transition/tests/owned_codec_transition.rs:7,46`; added by `cb6e44d70cb9094a99f9a4e9b204be2ce4723f4e` (later source-route fixes in `0c3213b3827dfeb52c75728f54dcb7ee6fdc000e`) | Produce an actually source-owned V1 output; authenticate a same-route codec transition; preserve authored payload/source identity and original bytes; reopen codec-retired route; ordinary source deletion must remove both publication/presence rows and survive reopen; independent branch resumes and produces fresh V2 output, then preserves it on edit. | The correct public route is successor Directory request `.with_program_transition(real_predecessor, resources, author)` plus ordinary close/open. Native's existing `durability/authority/recovery/codec_transition/tests` has genuine installed publication and authenticated transition proof, cold retired-route observations and graph retirement. A source-owned-current-storage variant can protect retirement/presence/delete semantics there using legitimate installation grants. | The current Home transition explicitly applies the same producer gate to both schemas. Its public DATA plan grants no private Query materialization authority. Full Query transition/reopen/regeneration remains an H5/H6+D16 dependency; a Native port alone cannot claim V2 Query producer settlement or Preserve behavior. |
| `worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/owned_outputs/world.rs:194-219`; added by `7df5047373838628e09b8a5c64397b590e23ba67` | Shared cold world for two-family owned history: same-build selection reuses both outputs with zero producer/source projection contacts; changed build reruns only old family; role corruption refuses with original publication/source/family and no effect; refusal data outlives teardown without retaining Native custody. Other callers cover cold parent/discovery and predecessor correspondence. | The eventual lawful route is Directory Home with real producer support, actual durable branches/history, same/new build and current principals, retaining Home through teardown. `cold_selection.rs`, `build_change.rs`, `hierarchy/cold_parent/*` and `predecessor/*` consume this restore fixture. A current MemoryOnly host can keep the live navigation/forest semantics tests; Native owns selected-publication correspondence and can protect its separate invariant. | Full cold/build/history semantics require Query-owned accepted outputs, implementation identities, program/output custody and branch/currentness admission. Native cannot reproduce Query producer fallback counts, discovery, fresh-principal decisions, or absence of Query refusal custody. No existing public non-interim import/open route supporting this population was found. This is a genuine H5/H6 Home capability blocker, with branch/history integration that must be qualified, not a fixture naming issue. |

## Concrete Native placement and authority limits

For movable diagnostics, a Native court must create its own genuine Native
schema/publications through existing installation receipts. The existing
`crates/worth-court-relational-certification/.../authored_retirement/registration.rs:5`
uses the real public `register_current_publication_storage` registrar;
`restoration.rs:17-50` captures, restores, readmits the true installation receipt,
and issues the installed publication owner. That is an existing lawful owner
route, not an application Original factory.

The exact negative cases must retain their distinguishing assertions and
meaningful clean twin. A direct classifier unit or a codec-only inspection
would not replace real historical closure, current roster restore, or lazy
current-ownership admission. Any mutated bytes remain adversarial **input**;
an owner may not label them an issued original image or transplant a genuine
Origin from different bytes. Place native-only tests at Native's existing
runtime/recovery owners or appropriate existing court crate. Do not mount a
Query application runtime in a Native unit test to disguise the crossing.

Existing cold source/recorder support provides a concrete narrow Native seam
for catalog/presence cases. Historical co-anchor and coupled-edge cases may
need a current-grammar recorder at their owning Native boundary; that is test
input construction, not permission for a new authority constructor, source
factory caller row, or application recovery route. It must retain the real
canonical membership/index integrity relevant to reaching the disputed check.
The stale CHK20.1 scanner itself is independently being ported; that does not
solve the D13 reach or authority problem.

## Disposition for the integration owner

There is no lawful six-caller mechanical rename. The meaningful Native
diagnostic subclaims in rows 1/3/4 and Native retirement subclaims in row 5
can be developed through existing Native ownership, preserving actual issuance
and malformed-input distinctions. That work can proceed without adding a
single D13 caller. It does **not** by itself close their full Query cold
integration claims, nor rows 2/6's required Query continuations/build/history.

The complete happy journeys need the actual Home capabilities named above.
Milestone 3's H5/H6/H9 and D16 own those grants. The public route already exists;
its supported state families do not. Implementing/qualifying those owner
capabilities is the honest completion path. Merely changing the support callback
would make an unsupported claim and is not proposed. Replaying seed state,
installing a fresh host with remembered outputs, swapping old data for a new
capture, routing through one of the frozen 54 callers, re-freezing the list,
or making byte-only data into Origin are not alternatives.

If those prerequisite owner slices are outside this integration assignment,
the remaining decision is a real scope/claim disposition for the coordinator:
assign the missing Home implementation, or explicitly decide the incoming
unsupported cold-journey tests' retained/held status with their exact claims
and restore references. This review does not authorize removal, ignored tests,
weakened assertions, a frozen-list expansion, or calling a Native-only proof a
passing Query journey. Compilation and unrelated allowed Tier-2 work can
continue while that concrete dependency is resolved. No final approval is given.
