# Phase5 Bank current accounts: evidence

Status: all scoped runtime and repository verification passes:524 distinct
passing tests, with two established unrelated cleanup failures documented below.
Independent qa-loop, qa-tests and code-quality-qa all PASS. This Bank adoption
slice does not close Phase5 or Phase6.

## Authoritative receiving path

The current-only Bank Parent uses the unchanged v1 AccountDiscoveryQueryBinding,
actual principal binding/mapping and DiscoverOwnAccounts ability. Its three real
paths (personal owner, authorized user, business owner) preserve union/dedup and
AccountId ordering. Existing accounts() and pinned Source reads remain available.
The ordinary Bank receiver executes current_accounts() with the original result
limit, cancellation and allocation controls. Returning all accounts visits all
returned rows and pays ordering/meaning bookkeeping; selective pruning is not claimed.

The installed finite provider stores genuine stable SourceSlots in controlled
NodeWriter storage. Checked sorted construction appends once per Source; warm
changes remove actual before slots and insert actual proposed slots. Passive
entry-attempt/change counters use the existing fenced certification control.
Committed mutations plus an independent ordinary query establish success.

## Causal shared-owner changes

- Execution owns the promoted generic AVL mechanic formerly duplicated by the
  prefix example. A checked sorted builder writes each node once. The original
  prefix meaning stays in its existing value/consumer owners.
- Native prepared row reads retain the actual immutable loan and forward ticket.
  Construction validates full-generation membership and exact view/program/scope;
  fields retain original audience/kind/control checks. Warm retained coordinates
  acquire current inverse membership. Neither token nor coordinate grants READ.
- Evaluation memo storage retains its actual admitted root definition, lowers
  extent-local node/direction addressing into sparse pages, and compares complete
  anchors. Named definitions retain complete identity. Singleton storage avoids
  real tree allocations; growth, promotion, comparisons and final polls stay paid.
- Authored history and maintenance share one live original Source stage. Completed
  discovery/support/meter transfers only on exact actual record/adjacency equality
  and exact context/group/program custody. History precedes invocation union;
  authored extras retain full discovery. No packet, posting or interval is omitted.
- Cold Query qualification constructs region/parent columns directly from the
  complete captured ordinal extent, including holes. Full slot and parent checks,
  source bijection, connectivity and cycle qualification remain. Walk tags use
  funded indexed scratch: O(frontier) initialization and O(live nodes) parent reads.
  Warm predecessor editing remains unchanged.
- Cold postings use the existing controlled fallible heap and a checked native
  sorted-map constructor. It creates one actual Entry/Node pair per input, admits
  their original allocations, checks strict full keys/exact count and preserves
  original final-stop precedence. Payloads move with existing native custody.
- History birth seeding first checks each actual full-ID stream's strict order.
  Proven ordered streams use the same bulk constructor; other orders retain the
  original incremental collector. Actual before/after incidence and kind checks
  have one owner; later authored/physical/no-op/field/lifecycle handling is intact.
- The Source-meaning interner retains one paid registry Entry rather than repeating
  four descents. Equality, coordinate partitioning and conservative lookup price
  remain. Incoming/upgraded custody drops after the registry guard on typed error
  and unwinding; failed admission does not spend the next ordinal.

## Original-budget diagnosis and repair

All populations and the original Bank16M-work/128MiB policy remain unchanged.
The lawful160 case exposed discovery, cold qualification and later row interning
costs in sequence. Its exact159 limit cause remains ManagedIndex -> QueryRead ->
ResultLimitExceeded; production denial custody was not flattened.

The full Bank family then exposed four192/250/265 bootstrap regressions. Direct
cold columns restored both192 cases. Indexed cycle tags and bulk cold postings
removed actual persistent searches/path copies. The original SDK invoice, rather
than the narrower group counter, then isolated the remaining cost in authored
input collection. For the265 fixture:

| Actual original SDK stage | Before ordered birth seeding | After |
| --- | ---: | ---: |
| Authored input collection | 6,011,697 | 2,243,663 |
| Complete authored history | 7,713,471 | 3,945,437 |
| Complete Source preparation | 11,106,469 | 7,338,435 |

These are diagnostic stage invoices, not exclusive whole-request totals. The
later unchanged canonical authored-index preparation now fits the same request.
Both250/265 cases pass, and the final complete Bank family passes. Temporary
backtraces, prints, context callback parameters and installation diagnostics were
removed before final qualification. Earlier duplicate diagnostic labels at writer
allocation/final qualification establish only the broader cold-construction
attribution; they do not independently identify one of those two positions.

## Verification

Rust1.99.1, configured sccache, CARGO_TARGET_DIR=C:/worth-build-cache/target,
RUST_MIN_STACK=67108864. No new root/SDK family, feature/profile/flag change or
cache clean. Existing court families and scoped owner tests were reused.

| Receiving or owner lane | Result |
| --- | --- |
| Complete Bank external-rail/server court | 187 PASS,1 existing ignored speed probe;44.39s |
| Current rebuilt rail product/process receiver | 55 PASS;18.79s (overlap with Bank family, not added to distinct count) |
| Complete Native external court | 98 PASS,1 existing ignore;51.17s |
| Execution generic ordered-tree owner | 4 PASS;.05s |
| Existing Execution court, new ordered_map leaf | 4 PASS;.01s; public full-key/rank/edit/share oracles, N-entry linear retained footprint, original stop/release/retry |
| Native sparse memo owner | Final7 PASS;.01s; full-key model, sparse page bounds, promotion/growth stops |
| Query Source-meaning interner owner | Final15 PASS;.66s combined with cold5; full coordinate/equality/release/error/ordinal custody |
| Final Native birth-order owner | 3 PASS;.01s combined with memo7;52.40s owner build |
| Final Query cold-cycle owner | 5 PASS;.66s combined with interner15;2m51s owner build |
| Final Query consumer family and large geometry case | Broad200 PASS;103.07s,2m17s build; large1 PASS;1127.62s |
| Actual Cargo boundary/context gates | PASS, both actual Cargo commands |
| Dirty formatting/line caps/function scan | Formatted;81 Rust files pass caps;47 function advisories,0 scan errors |
| UTF-8/C0/new-mojibake/diff checks | PASS;92 dirty files/81 formatted Rust files; zero text errors; configured Git diff check clean |
| Independent qa-loop/qa-tests/code-quality-qa | Final PASS for all three skills after runtime, scope, documentation and text review |

The200-case consumer run excludes the separately executed large geometry case
and these two established unrelated cleanup failures:
`partial_attempt_disposal_retries_actual_retired_cleanup_until_its_reader_releases`
and `previous_head_cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again`.
An explicit current-binary audit with the required64MiB stack reproduces their
established NestedStopped/cleanup-settlement failures in1.52s. An initial audit
without that stack setting overflowed and is excluded from evidence. No cleanup
code or oracle was changed. The independent reviewer verified the identical prior
baseline assertions, branch/attempt retirement ownership and retained-reader scope;
no causal blocker for this Bank/index slice was found. The family is not reported as203 passes.

The five actual Bank current-account cases are included in187: real three-path
parity/dedup/scope/unopened-empty roots; warm grant/revoke/remaining path; personal
and business creation; original pinned Source preservation/equal-head Parent
refusal; unchanged160 setup,159 limit/lawful160/repeated160 reads and cancellation.
The Native family includes actual foreign/current-view/removed/cancelled row-ticket
cases plus no-op/field omissions, original invoice, installed allowance and support.

The actual boundary gate first rejected the new live-request unit-test file.
Those four tests moved into a new leaf of the existing Execution court target.
No runtime hook, new court root or SDK family was introduced. Public rank/edit/
retention oracles replace private Node-height introspection; balanced midpoint
construction is a static argument. The topology gate and four relocated tests
then pass. The final receiver receipts precede only this test-only relocation,
which changes no production behavior.

The separate large geometry case preserves every original threshold and reports
these complete operation invoices (setup/scope work stays included in its actual
request; separately authored remote setup batches remain visible):

| Real Source population | Refused edit | Committed edit |
| --- | ---: | ---: |
| 1,000 | 2,694,909 | 2,246,447 |
| 2,000 | 2,901,423 | 2,598,671 |
| 2,000 plus50,000 unrelated | 3,475,565 | 3,163,546 |

Growth stays below25% on doubling and50% beside unrelated geometry, including
the existing difference-of-invoices guard. The actual provider counters and
independent coordinate/contact oracle remain part of the court. This is paired
maintenance/verification evidence, not actual topology face-discovery proof.

The distinct passing count is187 Bank +98 Native +201 consumer +4 Execution
ordered-tree +4 Execution external sorted-map +10 Native owned mechanics
+20 Query owned mechanics =524. Process55 overlaps Bank and is not added.
The two ignored tests are the existing Bank speed probe and Native ignore.

## Remaining milestone work

Public topology's actual discovery1000/2000/+50000 qualification, integrated public
closure/docs and Phase6's single final private pin/adoption remain unfinished.
The paired large geometry maintenance court is not substituted for actual
public topology discovery. All scoped runtime, documentation, text and independent final gates are closed.
This slice is cleared for commit.


## Independent final gate

The maintained independent reviewer clears qa-loop, qa-tests and code-quality-qa
for this complete Bank slice after524 distinct passes, final public funding/order
receivers, actual repository gates and the92-file text audit. The corrected test
placement uses the existing Execution court target; no runtime hook or new root
is present. The two established cleanup failures remain explicit, with unchanged
retirement ownership and no causal blocker. No required scoped check remains.
The next implementation target is actual public topology discovery.
