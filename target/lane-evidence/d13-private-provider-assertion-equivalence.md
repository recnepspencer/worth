# D13 private-provider assertion equivalence

Status: corrected design, not yet implemented in full. Independent review rejected
opening-only reset proof and branch-merge attribution. Those designs are withdrawn.
No local provider, test adapter, fake custody, public root constructor or counter
observation is permitted. No numeric allowance changes.

## Existing-slot history preservation

Add private `arena/slot_lifecycle/current_slot_data.rs` (and narrowly split
metadata-retirement / generation-reuse leaves if needed). `CurrentSlotData<'a,K>`
borrows only existing current/suffix columns, pin counts and presence bits. It
contains neither `history_prefixes` nor `&mut RecordArena`. Arena entries resolve
the physical row and immediately delegate reset, extra update, retirement,
rematerialization and existing-slot reuse to that receiver. Preserve the existing
statements, error order, request callbacks and charged operations. Append and
suspension stay separate because they legitimately initialize/clear prefixes.
Thus the actual mutation algorithm cannot clear or replace an omitted prefix.

Extract prefix-opening and suffix projection steps from the actual owned-history
algorithm. Serving reconstruction continues to obtain the opening from its real
arena prefix; units supply plain immutable opening DATA, never an operational
prefix or source. The original forced-generation case uses the same mutation
receiver and projection steps with an independently authored complete metadata
oracle, comparing exact canonical history and both kind commitments before edits,
after endpoint edits, and after reset/reuse. Compiler-enforced prefix exclusion
is the preservation proof; an external opening alone is insufficient.

## Whole-slot adoption

Extract the actual sparse physical-slot selection into `arena/slot_adoption.rs`.
A shallow generic `copy_selected_value<T:StoredValue>` applies that plan to the
ENTIRE stored carrier. The production `history_prefixes` column must go through
it unchanged: its concrete T is `StorageAllocation<Option<RecordHistoryPrefix>>`.
`StorageAllocation::clone` shares its one immutable Arc backing; it cannot mix
selected member/opening fields with previous custody. Other truth columns use the
same plan. No new macro, blanket impl or generic provider interface is added.

Pure units use the actual plan and real SharedColumn/StorageAllocation carriers
with plain complete coordinate/source-identity DATA. Check sparse/existing and
appended destinations, untouched rows, whole-value equality, backing identity
preservation and the old original remaining unchanged. These plain values cannot
construct a serving prefix or issue custody. Prefix/suffix projection independently
compares exact history and both commitments against complete metadata.

Public qualification uses a cold `UpdateRelationEndpoints` transaction, which
reaches storage publication's slot adoption. Demand Full through a genuine main
observation and `read_truth().read_observation`, not through the substrate mirror
or capture of separately retained roots. Require zero reads of the selected prior History locator through cold Head admission
and endpoint edit/publication, then exactly one during the delimited observation-root
Full demand. Use a simple region without dependency histories. Require the exact
expected observation commit/component, the returned current entity/relation data,
and old/current neighbors to equal the authored oracle.
A warmed fixture with no deferred history read fails. Do not describe it as a
two-source runtime observation: complete carrier selection is proved by the
opaque type and independent value unit, while the public court qualifies real
admission/publication/deferred history.

## Cancellation

The actual demand function delegates loaded-result acquisition to private
`cell/demand/readmission_count.rs`. Typed Err returns before the cell's atomic
lease count changes. A pure unit checks Cancelled leaves counts 0, 3 and usize::MAX
unchanged, without Runtime, SerialRequest, ExecutionRequest or provider.

A public external SelectedSource court cancels after its actual selected Head
read, keeps exact Request(Cancelled), proves request-memory refund and a fresh
request's successful original-data retry. The <4 physical-read assertion counts
this region's actual Head and History locators, exactly the two members the
original private provider held. Symbol and unrelated partition reads are not
misrepresented as region reads. No public counter observer is added.

## Original named claims and replacements

- `head_prefixes_rebuild_owned_history_after_endpoint_edits_and_generation_reuse`:
  current/suffix receiver (compiler preservation), independent complete-history
  value oracle and genuine cold edit/Full companion. Retain the original until all
  three are implemented and green.
- `adopting_a_different_head_carries_its_owned_historical_projection_with_the_record`:
  actual opaque carrier/slot plan value proof, plus genuine cold edit/Full court.
  Exact private source Arc equality is not claimed as a public observation.
- `cancellation_during_readmission_preserves_the_request_and_does_not_damage_the_cell`:
  exact owner counter unit plus actual cancellation/refund/retry/region-read court.

User's codec-grant resume authorizes migration of these three claims, not their
retirement. Removing their local adapters is allowed only after equivalent proofs.
The newly attempted uncommitted dense-initialization src court was withdrawn for
Iteration Law2; no committed test was deleted. Existing public work/custody courts
remain, without claiming isolation of its first dense-level allocation.

## Review status

The revised adoption compiler proof and public cold-update companion were
independently APPROVED by d13_private_decomposition_review. Its reset/reuse P1 is
accepted; the narrow receiver is required. Neither branch merge nor a raw opening
is labeled an equivalent proof. Source implementation and runtime qualification
remain subject to fresh review before completion.

Review P2 corrections accepted: the public companion keeps exact 0/1 History I/O,
expected observation commit/component and exact current content assertions. The
borrowed receiver allocates nothing and preserves every existing charge/error order.
