# Semantic correction: Bank history transport

Status: Bank transport implemented; focused and real process evidence passes.
Governing contract: [9.18 Phase 4](../../plans/WORTH-query/milestone-9.18.md).
The [public cutover](semantic-correction-public-cutover.md)'s shared-codec/DTO
Bank checks were batched with this transport slice. Phase 4 is certified and
closed after the public family, locality, native, review and repository gates
passed. The adequate Phase 1–3 evidence is reused.

## Completed Bank evidence

The real authenticated Bank/user-node process journey
`recorded_correction::reapplication::historical_money_reapplication_preserves_selection_across_response_loss_and_later_lineage`
passes in 277.36 seconds. It proves response-loss recovery, no-active exact retry,
width-one history traversal and response replay, older U selection under active V,
stale and cross-session refusal, later-lineage retry and intent drift. Independent
balances and journal reads preserve the original movements. Actual completed
S/U/R/V/R2/V2 owners retain their own occurrences until historical use finishes;
all close normally afterward.

The focused actual-HTTP lifetime court
`advanced_history_page_requires_refresh_while_foreign_runtime_stays_denied`
passes in 1.98 seconds. Retained owners permit fresh initial and continued
width-one pages after an old chooser is refused. Releasing every relevant owner,
selection and observation before a later publication produces the exact native
`Source(Ancestry(Denied(History(SourceUnavailable))))` refusal and HTTP
Stale/Refresh, with no additional transfer. Foreign-runtime refusal retains its
separate permission-denied meaning.

The grouped Bank libraries and affected ordinary, approved-payment, protocol and
process targets compile. Fourteen adapter, DTO, shared-codec/source-only/freeze
and protocol controls pass. Existing registry evidence covers shared capacity,
owner/purpose/expiry, disappeared in-flight completion and failed-resume recovery.
The grouped Host contracts pass 40 positive and 68 compile-fail cases. Standard
boundary and generated-context checks pass on the checked candidate. Later native
changes require their affected evidence and final enforcement; they do not require
repeating these unchanged Bank builds or process portfolios.

## First working journey and completion work

An authenticated teller sends thirty, retains its completed ordinary owner,
reverses that exact occurrence, inspects its canonical historical alternative
through Bank and
user-node HTTP, explicitly chooses it, and reapplies the captured original input
under a new key. The ordinary SendMoney owner completes retained delivery.
Retain each completed S/U/R/V/R2/V2 owner while historical selection and retries
need that occurrence, then close every owner normally. Each completed owner pins
its own World observation; U does not retain S. Independent account activity and
balance reads prove a new journal and the original journals' preservation.
This real path is the first integration
checkpoint, using the process fixture and test owner named below.

Two prerequisites immediately precede that checkpoint: expose detached native
page custody and fresh native selection through publication; opt the lawful
SendMoney correction into its original source action and retain its ordinary
captured-input delivery owner. Complete the existing registry/HTTP/user-node
integration before claiming the journey, rather than proving disconnected DTOs.

Then extend the same journey to bounded multi-page history, older-alternative
selection, lost responses, later-lineage exact retry, fresh authorization and
stale/foreign/expired tokens. Focused registry controls establish transport
admission independently of the arithmetic. Existing adequate native and process
evidence is reused; a full unchanged process portfolio is not rerun.

## Inspected owners and authority boundary

The execution `correction_source/alternatives.rs` already owns lifetime-free
native pages, descriptive references, opaque continuation and complete active
resolution. Continuation and resolution bind the exact target read identity;
continuation also retains the native posting generation. Only complete traversal
permits selection. Selection rechecks canonical relationships, retained material,
ancestry and current source disclosure. Reapplication preparation then uses the
exact captured input and current original source action; completed command retry
restores its recorded selected/active pair before first-use eligibility checks.

Publication's alternatives wrapper exposes consuming `into_page()` and
`corrections().select_alternative_coordinates(&page, &source_coordinates, limits)`.
The latter's coordinates describe the selected reversal's **operated source**,
already exposed by each native reference. Execution resolves that source through
native history and fresh disclosure, reads its unique canonical reversal with
`reversal_of_source_at_basis`, checks the completed page's root, and delegates
existing native alternative selection. There is no public reference constructor.
This lets an earlier row be selected after the final page without retaining a
growing list of references or reconstructing a native identity from wire fields.

The transport contract is descriptive:

```text
history(source, bounds) -> rows[{source: BankHttpCorrectionSource}], token
continue_history(token, bounds) -> rows, same token, complete
create ReapplySendMoney { history_token: token, source: chosen_row.source }
  + ordinary custody_id, sequence, idempotency_key and request controls
  -> native source operation -> resume/deliver/read/close
```

`complete` describes traversal completion only; it is never an applicability
permit. No replacement SendMoney input is accepted. Bank scalar source DTOs and
their native conversion are owned by the preceding cutover. Current protocol
admission/version rules and the wire authority fence remain in force.

## Concrete module destinations

All paths below are relative to their named crate. New and touched Rust owners
stay at or below 400 lines; split by the listed responsibilities, not by size alone.

| Crate / owner | Responsibility |
|---|---|
| Query execution `primary_graph/product_operation/correction_source/alternatives.rs` and child `alternatives/selection.rs` | Resolve selected source coordinates to an actual canonical reversal; preserve existing native selection and exact target checks. |
| Query publication `application_entry/correction/alternatives.rs` and exports | Detach the existing native page; expose fresh request selection; retain the current request-bound convenience API. |
| Bank domain `schema/correction.rs` | `ReverseSentMoney` source reapplication dispatch and validated source command key; original handler and current admission remain authoritative. |
| Bank server `correction.rs` and `lib.rs` | Narrow `prepare_send_money_reapplication` orchestration beside the accepted inverse entry; return the existing captured-input operation preparation. |
| Adapter `http/protocol/correction_history.rs`, protocol exports | Scalar history request/row/outcome DTOs and explicit selection; existing deadline, ancestry and disclosure dimensions plus a distinct nonzero page width. No native page/resolution type enters protocol. |
| Adapter `http/server/continuation_registry.rs` | Shared finite token inventory, random-token issue, total capacity, expiry and concrete record dispatch. |
| Adapter `continuation_registry/activity.rs` | Move account-activity record/replay progression under its concrete owner; completion checks retained in-flight identity after shared-registry expiry. |
| Adapter `continuation_registry/correction_history.rs` | Correction purpose/source/owner binding, latest native page custody and initial/last response replay. No canonical lineage decisions. |
| Adapter `continuation_executor.rs`; child `continuation_executor/correction_history.rs` | Existing bounded queue dispatch; freshly authenticated history page/continue/replay orchestration. Keep the current 360-line executor below its cap by extracting the new responsibility. |
| Adapter `correction_history_routes.rs`, `routes.rs`, installation wiring | Strict request admission and route binding; create one shared registry at installation using the existing capacity/lifetime configuration. |
| Adapter mutation command/application admission and `mutation_construction/correction/reapplication.rs` | Admit the explicit selector; call the original source action's ordinary preparation and translate typed denials. |
| Adapter `mutation_advancement.rs`, `mutation_executor.rs`, `mutation_worker.rs`, `mutation_custody.rs` | Pass shared registry custody; validate token before construction; retain captured SendMoney source custody using the existing macro and worker lifecycle. |
| User-node `protocol/correction_history.rs`, `session/correction_history.rs`, `server/routes/correction_history.rs` and owning declarations | Forward explicit chooser requests using the current session credential; retain no native page or correction state. |
| Adapter `continuation_registry/tests/{fixture,failure_recovery,stale_resolution}.rs` and owning registry tests | Real runtime disclosure/staleness and lifetime integration plus focused transport inventory tests. |
| Bank courtroom `transport_process_courtroom/recorded_correction/reapplication.rs`; owning module registration | The new real authenticated process journey. Split scenario helpers into a semantic child if necessary for the line cap. |

The existing account-activity registry is shape-specific; do not introduce a
generic registry framework. Its concrete activity and correction record variants
share the configured `opaque_handle_capacity`, `opaque_handle_lifetime` and
existing random-token recognition policy. Correction retains only the latest
native page/continuation, original traversal source, purpose and scalar response
descriptions needed for initial/last replay. It stores no active/consumed flags.

## Ordered implementation and refusal behavior

1. Add the native/public bridge above. Fresh selection must disclose the chosen
   operated source, not merely the traversal root. Prove lawful detached-page use
   under a new request and refusal after a target change, and retain the existing
   native ancestry/material/index stops. No whole-history lookup fallback.
2. Add lawful SendMoney reapplication and captured-input delivery custody. The
   current handler prepares a new journal from current account truth. It does
   not reuse a prior proposal or rewrite input. Completed external ApprovePayment
   remains unsupported; estate shared-codec proof belongs to the cutover.
3. Split the concrete registry responsibilities and share registry custody with
   the continuation executor and mutation workers through `Arc<Mutex<_>>`, as
   already done for recovery inventory. Authenticate before locking. Under the
   lock check expiry, owner, purpose and traversal source; clone the opaque page
   `Arc`; release the lock before disclosure, native history or mutation work.
   Both activity and correction-history completion must check the retained
   record's identity and in-flight request state before publishing progression.
   If concurrent expiry/purge removed it, return the existing typed unavailable
   or denied outcome, never Delivered with an unretained continuation token.
   A removed/expired in-flight entry is not resurrected after native work.
   Remove/replace pages under the lock, then release retired native page custody
   outside it; avoid making final native pin reclamation part of the lock hold.
4. Add HTTP/user-node page and continuation progression, then explicit mutation
   selection. Keep existing bounded queue admission. Mutation Create uses its
   existing custody permit; never wait for a permit while holding the registry
   lock. Native preparation/publication runs outside the registry lock. Store
   returned ordinary mutation custody before replying, including on response
   loss. `NativeJob` is birth-only and must not become a second page engine.
5. Complete the first process checkpoint, then extend its adversarial sequence.
   Finish consumer guidance in the existing correction/front-door owners and
   capability index, protocol source fence, generated contexts and enforcement.

Initial replay identity includes authenticated owner, correction purpose,
traversal source and request ID. A repeated initial request returns its recorded
first response; the last continuation request returns its recorded last response
without advancing again. Both protected responses require fresh native disclosure
of the original traversal source. Denied access scrubs protected replay. Continuing
uses the retained native continuation and fresh disclosure; changed target/index
generation refuses rather than restarting silently. Incomplete pages cannot select.

Selection additionally discloses the chosen source and uses current native
resolution. A completed page is not refreshed implicitly after a branch advance.
Foreign owner/purpose/source, expired tokens and stale resolution deny fresh use.
An opaque token is transport custody, not durable command meaning: after refresh,
the same selected source and same key can recover the recorded command despite a
later active reversal or no active reversal. A changed selected source under the
same key remains intent drift. Lost mutation responses use normal retained custody
resume; chooser expiry never discards or settles that mutation owner.

Dropping/expiring a chooser releases its native continuation pins. A page `Arc`
shares any existing native boundary reservation; it does not clone a charge or
detach the backend. Token inventory and caller page width bound counts, but do
not themselves prove retained payload-byte accounting. Preserve existing native
execution policy and typed stops; resolve the accounting question below through
the owning allocation path rather than add a guessed transport byte/page ceiling.
The page's Relational continuation custody does not retain World source ancestry.
Keep the existing completed operation custody for every occurrence needed by the
historical journey. Releasing those owners permits later publications to retire
optional World history; scalar coordinates and a live chooser token cannot prevent it.

## Evidence design and resolved ownership questions

The ignored real process selector is
`recorded_correction::reapplication::historical_money_reapplication_preserves_selection_across_response_loss_and_later_lineage`.
Use the existing real identity provider, cold Bank/user-node processes and TCP
fixture. Perform S30, unrelated7, U, R, V, unrelated3; traverse width-one pages and
choose older U under V. After reapplying thirty, independently expect source -40
and recipient +40 from their baselines, with distinct journals and preserved
original/reversal rows. Lose a mutation response and recover normal custody.
Reverse the new occurrence again, refresh history, retry its old key and prove
the same performed receipt with no additional journal or balance change. Also
retry once with no active reversal. Cross-session and stale-token new commands
must leave independently read account activity unchanged. Retain the actual completed
S/U/R/V/R2/V2 custody through selection and retry, then close all owners normally.
The local control releases every relevant completed owner, selection and observation,
publishes again, and requires fresh source selection to report unavailable ancestry
and HTTP Stale/Refresh without executing another transfer.

Focused registry tests cover shared activity/history capacity, foreign owner and
purpose, expiry, initial and last exact response replay, and expiry during native
progression without resurrection. Add a small registry-boundary regression for
each concrete purpose: remove an in-flight record before completion, complete
with its new native custody, and assert the existing denied/unavailable outcome,
no replacement record and no exposed continuation token. Return discarded native
custody for release outside the lock. Use actual native pages for integration tests
covering incomplete selection, completed-page reuse after head change, current
chosen-source disclosure and protected replay revocation. Do not fabricate native
page/resolution constructors or duplicate Phase3 tree/required-output portfolios.

Source review and grouped compilation resolved the following ownership requirements:

- `Arc<NativePage>` satisfies the real Send/Sync requirements through its existing
  owners. Its boundary
  retains an immutable native basis and generation through existing owners.
- Bank installs a Serial policy with 128 MiB charged memory and 16 million work
  units. Query serial placement still issues a bounded inline execution lease,
  so this history path uses `ExecutionAllocationPolicy::Execution`. The native
  boundary reserves lookup-key/branch-name bytes and transient IDs are funded.
  The execution allocation owner reserves newly retained page backing and owned
  payload copies through its existing reservation, shared by `Arc`. Native page
  funding controls pass. Shared basis and
  generation backing are not charged twice. Transport retains only the latest
  native page plus bounded ordinary initial/last scalar replies per configured
  record. Those adapter replies make no execution-funded allocation claim. No
  new quota/framework or unrelated substrate expansion belongs to this ruling.
- Final boundary/payload ownership releases outside the lock under expiry/replacement.
  Both concrete purposes check retained in-flight identity/state,
  return their existing typed unavailable/denied outcome if absent, and release
  new native custody outside the lock. No broader activity policy changes.
- Shared lookup releases the registry lock before native execution or
  an awaited permit. Root disclosure alone is insufficient for selection; the
  coordinate resolver's chosen-source disclosure and native root check are
  mandatory review points.

One coordinator compiles affected libraries, grouped Host contracts, adapter tests
and the process target, then runs the named focused/process selectors. Reuse
accepted Phase2 accounting/estate/external journeys and Phase3 history/output
evidence. Run scoped formatting/line caps/whitespace, boundary and generated-context
checks and obtain independent source/QA clearance. Private repositories, later
milestones, store-backed custody reconstruction, locality/family regeneration and
full unchanged process portfolios are outside this slice; their exclusion does
not certify the rest of Phase4 or authorize the final merge.
