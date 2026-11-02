# K1c slice 5 evidence and lane closeout

Current qualification is recorded in the [resumed repair ledger](milestone-9.19-k1c-certification-evidence.md).
This document preserves historical slice receipts. Its Store content courts and
Store qualification claims are withdrawn; K1c has no integrated Store content
court. Combined checkpoint/tail/replacement/retained-reader proof now belongs to
Native certification. Earlier filtered runs do not qualify the full binaries.

Status: the same independent reviewer issued S5 PASS under qa-loop, qa-tests and
code-quality-qa after reading final code, courts, plans, raw receipts and complete
scoped lint paths. All caused findings are closed. The reviewer ran no builds and
changed no files. Only completed runs are counted below.

## Shape and boundary

No AspectShape form was added. Native owns immutable canonical frames, 32-byte
content-derived ContentRefId, frozen ASCII format/version contracts, admissions,
retained directories, original-funded backing and reconstruction. Store is physical
backing through its existing seam. Decl/Host's field-bound content recipe and
controlled reader/writer use this Native owner and the existing pre-publication
`phase/prepared/domain_value_write.rs` seal. The obsolete AspectValue::Bytes arm
was retired; InlineBytes was not implemented or changed by this lane. The sole new
production Sha256 owner is `crates/worth-relational/src/content/frame/integrity.rs`,
explicitly added to the exact #36 permitted-owner set in S1.

S5 moves every Native frame equality decision under original control and a 65-unit
metadata checkpoint. Both different identities and shared Arc backing now poll/pay.
Nonshared equal identities still compare every paid byte and refuse an inconsistent
body. This changes no signature and adds no generic Execution engine body, observer,
cost API, content store or public permission constructor.

## Each court and its independent oracle

- Native `content`: actual schema/transaction/selected-root authority, format/version,
  missing bytes, canonical framing, exact numerical order, integrity, raw-write
  closure, savepoints, foreign owners, original controls, retention, preimages,
  journal/checkpoint and durable-publication fate. Literal LE bytes, an external
  Python hashlib/OpenSSL digest, and a position-based integer interpreter are
  independent of the writer. The inner-byte mutant repairs only the enclosing
  artifact digest independently so the content hash must detect the corruption.
- Native `content::recovered_reference`: a genuine two-format checkpoint roundtrip
  then swapped descriptive read selector refuses FormatMismatch, absent identity
  refuses MissingContent. A separate actual ContentRef-tagged value in a REG6 Head
  member is changed without repairing certificates; exact CorruptCheckpoint at
  physical integrity precedes selected-root change. These are separate claims.
- Native `cost::content`: actual 1/2/4 MiB frames, serial-accounted and leased
  original invoices, independently observed encode-window original reservations,
  full numerical scans and frozen external identities; corruption at offsets 4095,
  4096 and the last byte; constant shared comparison versus full nonshared equality.
  A genuinely funded-record paired invoice isolates actual hash Work: version 2
  refuses before hash/callback, version 1 hashes then a zero-Work validator refuses.
  Equal metadata paths and unchanged reservations certify the invoice difference;
  they do not serve as the integrity oracle.
- Native `cost::content::directory`: 100/1,000/10,000 genuinely retained unrelated
  record/content associations. Setup is outside ordinary invoices but funded.
  Existing exact borrowed-record projection plus its caller-paid bound reads the
  selected field; a real transaction admits and commits one local edit. Literal
  target integers and selected field identity check results. Finite differences
  across tenfold populations reject a linear scan despite fixed overhead. Every
  setup/transaction/root/reader owner drops before the zero-balance assertion.
- Native `cost::content::bounds`: exact 16 MiB complete frame plus one sample refuses;
  four distinct genuine 16 MiB staged frames fit, fifth refuses 80 MiB versus 64 MiB.
  The 256 MiB original budget reaches staging refusal before memory exhaustion.
  Selected root stays fixed; cancellation/drop releases all staged custody.
- Native private `content::frame::collision_tests`: injected unequal actual bodies
  with equal claimed identity reach the independent full-byte comparison refusal.
  The affected lib filter also runs existing partition-content independent
  reconstruction/incremental-commitment oracles; its count is not all K1c tests.
- Public Decl/Host `content_door`: one generic application fixture performs create,
  read, replace, clear, removal, wrong-format/version/missing references and
  corruption refusals, demand/created-output correction, undo/reapply, Source
  expectation and checkpoint/current-data journeys. A conservative key remains
  constant while changed content reaches exact candidate membership; a separate
  literal integer-list full scan is the membership oracle. The existing domain
  seal refuses before provider/index/selected-commit publication.
- Public `content_door::controls`: a small typed input generates a transient
  oversize numerical stimulus, proving Native SizeLimit owns refusal before effects.
  Cancelling the same original request inside an actual Source content callback
  produces the exact controlled stop, preserves selected commit, and a fresh
  request rebuilds successfully. No invented public memory observer is used.
- S1–S4 archive/installation, erased Source comparison, publication and facade
  compile courts retain their exact prior evidence in the linked ledgers. They
  prove portable recipe provenance, duplicate rejection/shared declaration reuse,
  sticky inconsistency propagation, legal facade access and callback confinement.
  S2's real Store court uses the actual durable adapter; no fake-medium result is
  promoted to a Store durability guarantee.

## Measured results

Native all2 passed 61 tests, 149 filtered, 17.02s. Native unit1 passed 11 tests,
2,261 filtered, 1.24s. Public all2 passed 21 tests, 327 filtered, 1.58s. These are
completed run counts, not a sum of unique tests; final controls-category rerun is
recorded separately in the ledger.

Both serial-accounted and leased original requests measured identical encode/read
Work at each scale. All leased/serial final retained balances return to zero.
The original-budget peak is independently sampled through completed encode;
it excludes caller-owned numerical inputs and does not claim total process peak.
Microseconds are a single-run telemetry observation under concurrent court load.

| Complete frame MiB | Encode Work, each posture | Cold read Work, each posture | Nonshared exact Work | Shared warm Work | Hash byte Work | Retained original bytes, two frames | Observed encode peak original bytes | Encode us | Cold read us |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 4194242 | 3145992 | 2097217 | 65 | 1048600 | 2097680 | 1050808 | 94898 | 880 |
| 2 | 8388546 | 6291720 | 4194369 | 65 | 2097176 | 4194832 | 2099384 | 119172 | 1521 |
| 4 | 16777154 | 12583176 | 8388673 | 65 | 4194328 | 8389136 | 4196536 | 211504 | 2519 |

Hash Work is complete frame bytes plus the literal 24-byte domain, isolated from
layout/name checks. Each scale checks successful payload bytes, validator-billed
payload bytes (not a claim of full validator byte visits), correspondence sample
bytes and a complete independent numerical scan. Frozen external SHA-256 vectors:

| MiB | Python hashlib/OpenSSL identity |
|---|---|
| 1 | b5f7b593224cd8bc3d834a3538a62d79aede53436db98e84a2031aae22764ffd |
| 2 | de88338e576680ec007496f7cc9bb885d5e31c97c4926107eeccf0cf82431d2f |
| 4 | 87d9abda52ccc291ce509c6b7e0e9c449944ebfa46819be10e1db277636e4db7 |

The independent construction is literal WRTHCONT +LEu16(1)+LEu16(16)+ASCII
exact-grid-costs+LEu32(1)+LEu64(8+8*n)+LEu32(1)+LEu32(n), followed by n signed
LEi64 words i-floor(n/2), where n=(MiB*1,048,576-48)/8. This does not invoke the
production encoder or its digest result. Mutations 4095/4096/last byte must refuse
IntegrityMismatch; zero-Work rejecting validators certify the separate hash invoice.

| Unrelated retained associations | Ordinary selected-field/content read Work | Admission plus complete local commit Work | Held original bytes during measured journey |
|---|---|---|---|
| 100 | 28,888 | 268,250 | 205,443 |
| 1,000 | 29,272 | 274,022 | 1,996,683 |
| 10,000 | 29,784 | 281,718 | 19,889,883 |

Read Work increments 384 then 512; edit increments 5,772 then 7,696. Both second
increments must stay within 4x the first, independently rejecting linear growth
under tenfold populations. Absolute ratios also remain bounded. Setup retains one
real record per distinct frame; all owners drop before the zero-balance assertion.
Persistent staging 256/512 frames measures 7,337,824/14,724,704 Work. Complete-frame
size refusal is 16,777,224 versus 16,777,216 bytes; staged refusal is 83,886,080 versus
67,108,864 bytes. Public generated-input refusal is 16,777,217 versus 16,777,216.

S5 adds no generic code to worth-query-execution. The actual rebuilt unoptimized
test-cache rlib is 319,059,810 bytes, SHA-256
`a25354b3da2980e653ee877d45ac882fac4e516ba2b20d806cc5343b75aac801`, at
`C:/worth-build-cache/target-cut-k1c-content/debug/deps/libworth_query_execution-b7016cf434752ce1.rlib`.
S4 measured 318,612,510 bytes; the 447,300-byte difference includes the sync master
and changed support/dependency graph, so it is integration telemetry and cannot
be attributed to S5 or used as a pub-mono performance claim.

## Readings I chose

1. Native authority had to be established, as explicitly authorized; an identifier
   or a physical Store blob is not an existing content admission authority.
2. Ordered identity-free numerical content uses ContentRef at every size. A spline
   of 1,048,576 controls with four exact rationals each can exceed 16 MiB: typed
   SizeLimit refuses at admit before effects. No inline fallback, segmentation,
   truncation or approximate numerical conversion is selected.
3. Corruption certificates compose: a stored field-byte mutation must stop at the
   earliest physical certificate; it need not bypass commitments to reach a
   semantic FormatMismatch. A genuine post-recovery swapped read selector and the
   actual public domain-field swap independently prove that semantic boundary.
   No forged Merkle/history/root commitments or random-digest search was substituted.
4. Checkpoints reconstruct current Native bytes. They do not mint live World Source
   images or old correction occurrences. Initial maintained open returns exact
   RootUnavailable; an opted-in genuine authored transition cold-stages current
   Source, including an unchanged recovered row. Live undo/reapply remains covered.
5. Exact borrowed projection is the ordinary field path. Eager read_snapshot is an
   intentional whole-snapshot export and cannot serve as a local-read cost fixture.
6. Memory peak means the maximum independently sampled original Native reservation
   during encode through held completion, not process/caller-input memory or
   readmission peak. Timing is telemetry; Work/growth/typed refusal are gates.
7. Untouched enforcement debt follows the explicit AGENTS scoped exception. Final
   diagnostic paths are compared against dirty, staged, untracked and the entire
   origin/master...HEAD branch scope; no scoped failure may be accepted or suppressed.

## Reviewer findings and closure

The same reviewer is retained for all five slices. S5 review independently found:

- A scheduler-dependent peak assertion: completion sampling now holds the genuine
  frame until observer join; an unwind guard prevents a panic from hanging the court.
- Fixed-cost ratios could conceal a linear directory scan: finite-difference
  assertions now gate both reads and edits. They caught the eager snapshot fixture,
  which was replaced by the existing exact borrowed owner and its paid bounds.
- The stronger checkpoint FormatMismatch claim exceeded its actual injection seam:
  report physical and semantic certificates separately, as described above.
- Public size/cancel cases are correctness, not a nested cost category: they live
  under content_door::controls, while all scale courts use top-level cost::content.
- An incidental Unicode fixture encoding change: the original single codepoint is
  restored as the ASCII source spelling `grid/\u{e9}`.

Compiler/clippy development catches and all failed attempts are listed below;
none is presented as a passing court. Final reviewer verdict is PASS, with every
caused finding closed. It independently confirmed the final raw test counts,
17-file cap and zero full-scope overlap of untouched clippy debt. No build or test
was delegated to the reviewer. The initial-open wording finding was withdrawn
after checking the actual current court at source/courts.rs289–298.

## Final enforcement posture and stops

Required clippy commands exit 101 for untouched debt: one Native diagnostic in
mvcc/retained_entity_selection/payload.rs; four Native-court diagnostics in existing
managed-source work-account/nested courts; 64 public-court diagnostics in existing
document-history/workflow courts. Final complete-scope overlap is zero. The first
Native-court clippy also caught two caused constant-chunk diagnostics; both were
fixed and the rerun contains only the four untouched findings. All failed output
is preserved below. No suppressions, boundary widening or unrelated repairs were used.

Boundary-check, agent-context, package formatting, 17-file dirty Rust line caps,
branch and dirty diff checks pass. The independent verdict is PASS. Staging and
final cached-diff receipts follow this report's reviewed body; the terminal report
also records the final post-staging check and commit SHA. The earlier authority-scope stop was resolved
by the user's explicit authorization to build Native's owner. There is no remaining
hard-rule stop; live-history reconstruction and the coordinated SDK repin are not
claimed by this lane.

## Commit lineage and earlier complete evidence

| Slice | Commit |
|---|---|
| Boundary review | 4087fc1cec74124d4b16797b2c1e190ead7a8081 |
| Boundary corrections after K1e | 0aa7a2612fb9dcbf7272a0029faf188db0b884dd |
| Native transactional identity/admission | c878b6928ea65b27dc62cc977e09441d0b109841 |
| Durable closure/retention | 52f8df312bc702bd4af909cb86a53177bb09e06c |
| Public field-bound door | e058cc58e9daa84510a44bcdb4f87cbb5120bf13 |
| Source/correction custody | 9f7f6193067acef44bdc8068a9777e0280b6f52a |

S5 fetched and merged origin/master 9d86651c21ad179d121c8b3360e95e7d51a9c4de
as a82ac6888583c480275186bcffcd0c4d5327a498. Provider lifecycle is
52e7dba3dcbe257a209c69878438e7627867f328 and already merged. All three commands
exited 0. The admitted-operation/identity engine bodies from master stayed concrete;
there were no conflicts. S5's commit SHA is reported after committing.

[Complete S1–S3 ledger and all recorded failures](milestone-9.19-k1c-content-evidence.md)
and [complete S4 ledger, failures and scoped debt](milestone-9.19-k1c-content-slice4-evidence.md)
preserve every earlier check with exact counts. No workspace-wide command was run.
The coordinated SDK repin and consuming application cutover are outside this lane.

## Staging receipts

The same reviewer passed the final implementation before staging. Staged 22 reviewed
files on codex/cut-k1c-content-20261101. The first cached diff check caught only a
new blank line at the generated evidence file's EOF. It was trimmed without code
changes; the final cached recheck is reported in the terminal completion. Repeated
branch checking passes and staged dirty line caps still pass 17 Rust files. The
full lint-path union including cached changes still has zero final scoped overlap.

## S5 exact run ledger

Populated only from completed exit files. All Query courts set
`WORTH_IMPLEMENTATION_BUILD_ID=11111111-1111-1111-1111-111111111111`.
All long cargo commands ran hidden in the background to named logs, then were read.

<!-- GENERATED_RUN_LEDGER -->

| Run | Exact command | Exit | Counts/output |
|---|---|---|---|
| `slice5-boundary1` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice5-caps1` | `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | [workspace-rust-line-caps] PASS: checked 17 Rust files |
| `slice5-caps2` | `bash scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | [workspace-rust-line-caps] PASS: checked 17 Rust files |
| `slice5-context1` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed; no test count claimed. |
| `slice5-court-clippy1` | `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary `>= y + 1` or `x - 1 >=`; error: using `chunks_exact` with a constant chunk size; error: using `chunks_exact` with a constant chunk size; error: unnecessary closure used with `bool::then`; error: called `.err().expect()` on a `Result` value; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 6  Diagnostics=6; scoped=2. |
| `slice5-court-clippy2` | `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary closure used with `bool::then`; error: called `.err().expect()` on a `Result` value; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 4  Diagnostics=4; scoped=0. |
| `slice5-diff-branch1` | `git diff --check origin/master...HEAD` | 0 | Completed; no test count claimed. |
| `slice5-diff-branch2` | `git diff --check origin/master...HEAD` | 0 | Completed; no test count claimed. |
| `slice5-diff-cached1` | `git diff --cached --check` | 2 |  |
| `slice5-diff-dirty1` | `git diff --check` | 0 | Completed; no test count claimed. |
| `slice5-fetch` | `git fetch origin master codex/919-provider-lifecycle-20261027` | 0 | Completed; no test count claimed. |
| `slice5-merge-master` | `git merge --no-edit --file target/k1c-logs/slice5-merge-master-message.txt origin/master` | 0 | Completed; no test count claimed. |
| `slice5-merge-provider` | `git merge --no-edit --file target/k1c-logs/slice5-merge-provider-message.txt origin/codex/919-provider-lifecycle-20261027` | 0 | Completed; no test count claimed. |
| `slice5-native-all1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | test result: FAILED. 59 passed; 1 failed; 0 ignored; 0 measured; 149 filtered out; finished in 18.49s |
| `slice5-native-all2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 17.02s |
| `slice5-native-clippy1` | `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | 101 | error: returning the result of a `let` binding from a block; error: could not compile `worth-relational` (lib) due to 1 previous error; 8 warnings emitted Diagnostics=1; scoped=0. |
| `slice5-native-cost1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification cost::content:: -- --nocapture` | 101 | error[E0432]: unresolved import `worth_foundational::facade::SchemaVersionId`; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 1  |
| `slice5-native-cost2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification cost::content:: -- --nocapture` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 202 filtered out; finished in 19.71s |
| `slice5-native-directory2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification cost::content::directory:: -- --nocapture` | 101 | error[E0599]: no method named `checkpoint` found for enum `std::option::Option<T>` in the current scope; error[E0599]: no method named `checkpoint` found for enum `std::option::Option<T>` in the current scope; error[E0599]: no method named `payload` found for tuple `(worth_relational::facade::content::RelationalContentFrame, ; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 3  |
| `slice5-native-fmt1` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-fmt2` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-fmt3` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-fmt4` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-fmt5` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-fmtcheck1` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-fmtcheck2` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | 0 | Completed; no test count claimed. |
| `slice5-native-unit1` | `cargo test -p worth-relational --lib content:: -- --nocapture` | 0 | test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 2261 filtered out; finished in 1.24s |
| `slice5-public-clippy1` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --all-targets --no-deps` | 101 | error: unnecessary use of `clone` to create a slice from a reference; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: called `.err().expect()` on a `Result` value; error: using `clone` on type `CommitId` which implements the `Copy` trait; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: the loop variable `member_index` is only used to index `samples`; error: manual implementation of `.is_multiple_of()`; error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its ; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;  Diagnostics=64; scoped=0. |
| `slice5-public-clippy2` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --all-targets --no-deps` | 101 | error: unnecessary use of `clone` to create a slice from a reference; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: called `.err().expect()` on a `Result` value; error: using `clone` on type `CommitId` which implements the `Copy` trait; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: the loop variable `member_index` is only used to index `samples`; error: manual implementation of `.is_multiple_of()`; error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its ; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;  Diagnostics=64; scoped=0. |
| `slice5-public-controls3` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door::controls:: -- --nocapture` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 346 filtered out; finished in 0.14s |
| `slice5-public-fmt1` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host` | 0 | Completed; no test count claimed. |
| `slice5-public-fmt2` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host` | 0 | Completed; no test count claimed. |
| `slice5-public-fmt3` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host` | 0 | Completed; no test count claimed. |
| `slice5-public1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door:: -- --nocapture` | 101 | error[E0369]: binary operation `==` cannot be applied to type ; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 1 previous error; 2  |
| `slice5-public2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door:: -- --nocapture` | 0 | test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 327 filtered out; finished in 1.58s |
| `slice5-query-clippy1` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --all-targets` | 101 | error: unnecessary use of `clone` to create a slice from a reference; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: called `.err().expect()` on a `Result` value; error: using `clone` on type `CommitId` which implements the `Copy` trait; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: the loop variable `member_index` is only used to index `samples`; error: manual implementation of `.is_multiple_of()`; error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its ; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;  Diagnostics=64; scoped=0. |
| `slice5-query-fmtcheck1` | `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host` | 0 | Completed; no test count claimed. |
| `slice5-query-fmtcheck2` | `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host` | 0 | Completed; no test count claimed. |
| `slice5-stage` | `git add -- crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/mod.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/recovered_reference.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/content/transaction.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/bounds.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/controls.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/directory.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/frames.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/invoice.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/leased.rs crates/worth-court-relational-certification/tests/worth_court_relational_certification/cost/content/stimulus.rs crates/worth-relational/src/content/frame/mod.rs docs/capability-index.md plans/WORTH-query/milestone-9.19-k1c-content-evidence.md plans/WORTH-query/milestone-9.19-k1c-content-slice5-evidence.md plans/WORTH-query/milestone-9.19-k1c-content-slice5.md plans/WORTH-query/milestone-9.19.md workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/content_door/controls.rs workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/content_door/mod.rs workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/content_door/operation.rs workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/content_door/source/mod.rs workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/content_door/source/provider.rs` | 0 | Completed; no test count claimed. |

Completed receipts: 39; nonzero exits: 11. No running or missing exit file was counted. Repeated filtered runs overlap.

A manual `git diff --check` after the Unicode fix also exited0 with no diagnostics. Two ignored editing-script attempts failed before changing code: an expected-text AssertionError, then a BOM-decoding SyntaxError (invalid U+00BB). The script was corrected to UTF-8, then ran successfully; the fixture and docs were independently reviewed.

## Failed-run output

Compiler/test output and all clippy diagnostics below are from the named failed runs. Only trailing whitespace and preceding unrelated warnings are omitted.

### slice5-court-clippy1

```text
error: unnecessary `>= y + 1` or `x - 1 >=`
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational\
managed_source_work_account\public_only.rs:38:17
   |
38 |         assert!(stopped.work <= EXTRA - 1);
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ help: change it to: `stopped.work < EXTRA`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#int_plus_one
   = note: `-D clippy::int-plus-one` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::int_plus_one)]`

error: unnecessary `>= y + 1` or `x - 1 >=`
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational
\managed_source_work_account.rs:292:21
    |
292 |             assert!(stopped.work <= EXTRA - 1);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^ help: change it to: `stopped.work < EXTRA`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#int_plus_one

warning: unused import: `aspect_field_locator`
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\relational_cert
ification\aspect_storage\mod.rs:13:5
   |
13 |     aspect_field_locator, aspect_key, entity_field_aspect, entity_summary_struct_aspect, field_key,
   |     ^^^^^^^^^^^^^^^^^^^^

error: using `chunks_exact` with a constant chunk size
  -->
crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\frames.rs:78:54
   |
78 |         for (word, expected) in frame.payload()[8..].chunks_exact(8).zip(&input.samples) {
   |                                                      ^^^^^^^^^^^^^^^ help: consider using `as_chunks` instead:
`as_chunks::<8>().0.iter()`
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#chunks_exact_to_as_chunks
   = note: `-D clippy::chunks-exact-to-as-chunks` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::chunks_exact_to_as_chunks)]`

error: using `chunks_exact` with a constant chunk size
  -->
crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\leased.rs:25:53
   |
25 |         for (word, expected) in read.payload()[8..].chunks_exact(8).zip(&input.samples) {
   |                                                     ^^^^^^^^^^^^^^^ help: consider using `as_chunks` instead:
`as_chunks::<8>().0.iter()`
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#chunks_exact_to_as_chunks

error: unnecessary closure used with `bool::then`
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational
\managed_source_work_account.rs:104:27
    |
104 |             maximum_work: scenario.rule_limit.then(|| std::num::NonZeroU64::MIN),
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#unnecessary_lazy_evaluations
    = note: `-D clippy::unnecessary-lazy-evaluations` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::unnecessary_lazy_evaluations)]`
help: use `then_some` instead
    |
104 -             maximum_work: scenario.rule_limit.then(|| std::num::NonZeroU64::MIN),
104 +             maximum_work: scenario.rule_limit.then_some(std::num::NonZeroU64::MIN),
    |

error: called `.err().expect()` on a `Result` value
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\managed_source
\nested.rs:110:40
    |
110 |                     return Err(stopped.err().expect("original nested request is cancelled"));
    |                                        ^^^^^^^^^^^^ help: try: `expect_err`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
    = note: `-D clippy::err-expect` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::err_expect)]`

warning: `worth-court-relational-certification` (test "worth_court_relational_certification") generated 2 warnings
error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 6
previous errors; 2 warnings emitted
```

### slice5-court-clippy2

```text
error: unnecessary `>= y + 1` or `x - 1 >=`
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational\
managed_source_work_account\public_only.rs:38:17
   |
38 |         assert!(stopped.work <= EXTRA - 1);
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ help: change it to: `stopped.work < EXTRA`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#int_plus_one
   = note: `-D clippy::int-plus-one` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::int_plus_one)]`

error: unnecessary `>= y + 1` or `x - 1 >=`
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational
\managed_source_work_account.rs:292:21
    |
292 |             assert!(stopped.work <= EXTRA - 1);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^ help: change it to: `stopped.work < EXTRA`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#int_plus_one

warning: unused import: `aspect_field_locator`
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\relational_cert
ification\aspect_storage\mod.rs:13:5
   |
13 |     aspect_field_locator, aspect_key, entity_field_aspect, entity_summary_struct_aspect, field_key,
   |     ^^^^^^^^^^^^^^^^^^^^

error: unnecessary closure used with `bool::then`
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational
\managed_source_work_account.rs:104:27
    |
104 |             maximum_work: scenario.rule_limit.then(|| std::num::NonZeroU64::MIN),
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#unnecessary_lazy_evaluations
    = note: `-D clippy::unnecessary-lazy-evaluations` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::unnecessary_lazy_evaluations)]`
help: use `then_some` instead
    |
104 -             maximum_work: scenario.rule_limit.then(|| std::num::NonZeroU64::MIN),
104 +             maximum_work: scenario.rule_limit.then_some(std::num::NonZeroU64::MIN),
    |

error: called `.err().expect()` on a `Result` value
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\managed_source
\nested.rs:110:40
    |
110 |                     return Err(stopped.err().expect("original nested request is cancelled"));
    |                                        ^^^^^^^^^^^^ help: try: `expect_err`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
    = note: `-D clippy::err-expect` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::err_expect)]`

warning: `worth-court-relational-certification` (test "worth_court_relational_certification") generated 2 warnings
error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 4
previous errors; 2 warnings emitted
```

### slice5-diff-cached1

```text
plans/WORTH-query/milestone-9.19-k1c-content-slice5-evidence.md:2838: new blank line at EOF.
```

### slice5-native-all1

```text
thread 'cost::content::directory::content_local_lookup_and_edit_are_bounded_against_unrelated_directories' (46844)
panicked at
crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\directory.rs:163:9:
unrelated-directory increments must stay bounded: [28911, 31095, 49607]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test cost::content::directory::content_local_lookup_and_edit_are_bounded_against_unrelated_directories ... FAILED

failures:

failures:
    cost::content::directory::content_local_lookup_and_edit_are_bounded_against_unrelated_directories

test result: FAILED. 59 passed; 1 failed; 0 ignored; 0 measured; 149 filtered out; finished in 18.49s

error: test failed, to rerun pass `-p worth-court-relational-certification --test worth_court_relational_certification`
```

### slice5-native-clippy1

```text
error: returning the result of a `let` binding from a block
  --> crates\worth-relational\src\mvcc\retained_entity_selection\payload.rs:34:5
   |
20 | /     let source = crate::storage::residency::probe_head(
21 | |         execution,
22 | |         &historical.with_symbol_admission(&runtime.services.symbols),
23 | |         |source| {
...  |
33 | |     .map_err(|failure| storage(&input.historical, failure))?;
   | |_____________________________________________________________- unnecessary `let` binding
34 |       source
   |       ^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#let_and_return
   = note: `-D clippy::let-and-return` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::let_and_return)]`
help: return the expression directly
   |
20 ~
21 ~     crate::storage::residency::probe_head(
22 +         execution,
23 +         &historical.with_symbol_admission(&runtime.services.symbols),
24 +         |source| {
25 +             crate::storage::residency::probe_head(
26 +                 execution,
27 +                 &current.with_symbol_admission(&runtime.services.symbols),
28 +                 |target| read(input, source, target),
29 +             )
30 +             .map_err(|failure| storage(&input.transaction.basis, failure))?
31 +             .map_err(Error::Admission)
32 +         },
33 +     )
34 +     .map_err(|failure| storage(&input.historical, failure))?
   |

warning: `worth-relational` (lib) generated 8 warnings
error: could not compile `worth-relational` (lib) due to 1 previous error; 8 warnings emitted
```

### slice5-native-cost1

```text
error[E0432]: unresolved import `worth_foundational::facade::SchemaVersionId`
 --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\bounds.rs:6:49
  |
6 | use worth_foundational::facade::{ContentFormat, SchemaVersionId};
  |                                                 ^^^^^^^^^^^^^^^ no `SchemaVersionId` in `facade`

warning: unused import: `identity::*`
 --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\support\index_retirement\mo
d.rs:3:24
  |
3 |     history::BranchId, identity::*, indexes::*, runtime::RelationalRuntime,
  |                        ^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `aspect_field_locator`
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\relational_cert
ification\aspect_storage\mod.rs:13:5
   |
13 |     aspect_field_locator, aspect_key, entity_field_aspect, entity_summary_struct_aspect, field_key,
   |     ^^^^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0432`.
warning: `worth-court-relational-certification` (test "worth_court_relational_certification") generated 2 warnings
error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 1
previous error; 2 warnings emitted
```

### slice5-native-directory2

```text
error[E0599]: no method named `checkpoint` found for enum `std::option::Option<T>` in the current scope
  -->
crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\directory.rs:67:29
   |
67 |                     context.checkpoint(projection.exact_entity_state_read_work_bound().unwrap()).unwrap();
   |                             ^^^^^^^^^^ method not found in `std::option::Option<&ExecutionResourceLease<'_>>`

error[E0599]: no method named `checkpoint` found for enum `std::option::Option<T>` in the current scope
  -->
crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\directory.rs:71:33
   |
71 |                         context.checkpoint(128).unwrap();
   |                                 ^^^^^^^^^^ method not found in `std::option::Option<&ExecutionResourceLease<'_>>`

error[E0599]: no method named `payload` found for tuple `(worth_relational::facade::content::RelationalContentFrame,
ExecutionReport)` in the current scope
  -->
crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\content\directory.rs:81:41
   |
81 |                 i64::from_le_bytes(read.payload()[8..16].try_into().unwrap()),
   |                                         ^^^^^^^ method not found in
`(worth_relational::facade::content::RelationalContentFrame, ExecutionReport)`
   |
help: one of the expressions' fields has a method of the same name
   |
81 |                 i64::from_le_bytes(read.0.payload()[8..16].try_into().unwrap()),
   |                                         ++

For more information about this error, try `rustc --explain E0599`.
warning: `worth-court-relational-certification` (test "worth_court_relational_certification") generated 2 warnings
error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 3
previous errors; 2 warnings emitted
```

### slice5-public-clippy1

```text
error: unnecessary use of `clone` to create a slice from a reference
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\cell_history
\checkpoint_cases.rs:52:5
   |
52 |     assert_eq!(live_at::current(&restored, 1), [survivor.clone()]);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#cloned_ref_to_slice_refs
   = note: `-D clippy::cloned-ref-to-slice-refs` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::cloned_ref_to_slice_refs)]`

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
capability\grant_commands\mod.rs:40:9
   |
40 |         std::thread::scope(|threads| {
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods
   = note: `-D clippy::disallowed-methods` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::disallowed_methods)]`

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
capability\grant_commands\mod.rs:41:38
   |
41 |             let competitor = threads.spawn(|| {
   |                                      ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\mod.rs:63:20
   |
63 |     let terminal = std::thread::scope(|threads| {
   |                    ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\mod.rs:64:34
   |
64 |         let competitor = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only
\managed_cleanup\mod.rs:120:29
    |
120 |     let (terminal, cause) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only
\managed_cleanup\mod.rs:122:14
    |
122 |             .spawn(|| {
    |              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\retained_capacity\mod.rs:24:24
   |
24 |         let terminal = std::thread::scope(|threads| {
   |                        ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\retained_capacity\mod.rs:27:36
   |
27 |             let pressure = threads.spawn(move || {
   |                                    ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
native_postures\mod.rs:59:20
   |
59 |     let terminal = std::thread::scope(|threads| {
   |                    ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
native_postures\mod.rs:60:33
   |
60 |         let advancing = threads.spawn(|| {
   |                                 ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
retained\native_postures\mod.rs:20:24
   |
20 |         let terminal = std::thread::scope(|threads| {
   |                        ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
retained\native_postures\mod.rs:21:37
   |
21 |             let advancing = threads.spawn(|| {
   |                                     ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\native_workflow_operations\cap
ability\grant_commands\mod.rs:40:9
   |
40 |         std::thread::scope(|threads| {
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\native_workflow_operations\cap
ability\grant_commands\mod.rs:41:38
   |
41 |             let competitor = threads.spawn(|| {
   |                                      ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:19:37
   |
19 |     let (reached, update, result) = std::thread::scope(|threads| {
   |                                     ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:20:30
   |
20 |         let worker = threads.spawn(|| document.refresh_context(&request, limits(1)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:57:29
   |
57 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:58:30
   |
58 |         let worker = threads.spawn(|| document.refresh_context(&stopped, limits(1)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\cancel
lation.rs:158:29
    |
158 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\cancel
lation.rs:160:30
    |
160 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handoff
.rs:58:33
   |
58 |         let (reached, result) = std::thread::scope(|threads| {
   |                                 ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handoff
.rs:60:34
   |
60 |             let worker = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handof
f.rs:118:29
    |
118 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handof
f.rs:120:30
    |
120 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recover
y.rs:87:19
   |
87 |     let reached = std::thread::scope(|threads| {
   |                   ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recover
y.rs:90:14
   |
90 |             .spawn(|| execution.advance(document, &host.request(&interrupted_principal, &stopped)));
   |              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: called `.err().expect()` on a `Result` value
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\resourc
es.rs:48:10
   |
48 |           .err()
   |  __________^
49 | |         .expect("member bound4 admits2, result bound1 must refuse");
   | |_______________^ help: try: `expect_err`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
   = note: `-D clippy::err-expect` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::err_expect)]`

error: using `clone` on type `CommitId` which implements the `Copy` trait
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\mod.rs:154:10
    |
154 |         [first.commit_reference().commit_id.clone()]
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try removing the `clone` call:
`first.commit_reference().commit_id`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#clone_on_copy
    = note: `-D clippy::clone-on-copy` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::clone_on_copy)]`

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ca
ncellation.rs:98:30
   |
98 |     let (reached, outcome) = std::thread::scope(|threads| {
   |                              ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ca
ncellation.rs:99:30
   |
99 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:45:32
   |
45 |     let (reached, won, lost) = std::thread::scope(|threads| {
   |                                ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:46:30
   |
46 |         let worker = threads.spawn(|| loser.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:98:38
   |
98 |     let (reached, updates, result) = std::thread::scope(|threads| {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:99:30
   |
99 |         let worker = threads.spawn(|| owner.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
ace.rs:165:29
    |
165 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
ace.rs:166:30
    |
166 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\re
covery.rs:82:29
   |
82 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\re
covery.rs:83:30
   |
83 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:332:25
    |
332 |     let (left, right) = std::thread::scope(|threads| {
    |                         ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:333:28
    |
333 |         let left = threads.spawn(|| {
    |                            ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:337:29
    |
337 |         let right = threads.spawn(|| {
    |                             ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:351:25
    |
351 |     let (left, right) = std::thread::scope(|threads| {
    |                         ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:352:28
    |
352 |         let left = threads.spawn(|| {
    |                            ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:356:29
    |
356 |         let right = threads.spawn(|| {
    |                             ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:26:29
   |
26 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:27:30
   |
27 |         let worker = threads.spawn(|| document.history_page(&stopped, limits(1), Some(&cursor)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:60:37
   |
60 |     let (reached, update, result) = std::thread::scope(|threads| {
   |                                     ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:61:30
   |
61 |         let worker = threads.spawn(|| document.history_page(&request, limits(1), None));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\cap
ture_race.rs:40:38
   |
40 |     let (reached, updates, result) = std::thread::scope(|threads| {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\cap
ture_race.rs:41:30
   |
41 |         let worker = threads.spawn(|| edit.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\han
doff.rs:29:29
   |
29 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\han
doff.rs:31:21
   |
31 |             threads.spawn(|| edit.advance(&host.runtime().request(&principal, &stopped_scope)));
   |                     ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\rec
overy.rs:35:30
   |
35 |     let (reached, outcome) = std::thread::scope(|threads| {
   |                              ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\rec
overy.rs:36:30
   |
36 |         let worker = threads.spawn(|| {
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\redo\handoff.
rs:35:41
   |
35 |         let (reached, action, result) = std::thread::scope(|threads| {
   |                                         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\redo\handoff.
rs:36:34
   |
36 |             let worker = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\can
cellation.rs:92:29
   |
92 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\can
cellation.rs:93:30
   |
93 |         let worker = threads.spawn(|| {
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\ca
ncellation.rs:167:29
    |
167 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\ca
ncellation.rs:168:30
    |
168 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: the loop variable `member_index` is only used to index `samples`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\group.rs:180:25
    |
180 |     for member_index in 0..2 {
    |                         ^^^^
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#needless_range_loop
    = note: `-D clippy::needless-range-loop` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::needless_range_loop)]`
help: consider using an iterator
    |
180 -     for member_index in 0..2 {
180 +     for <item> in samples.iter().take(2) {
    |

error: manual implementation of `.is_multiple_of()`
  -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:72:23
   |
72 |         let days = if depth % 2 == 0 && offset == 0 {
   |                       ^^^^^^^^^^^^^^ help: replace with: `depth.is_multiple_of(2)`
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#manual_is_multiple_of
   = note: `-D clippy::manual-is-multiple-of` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::manual_is_multiple_of)]`

error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its
contained lifetimes
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:217:5
    |
217 |     drop(request);
    |     ^^^^^^^^^^^^^
    |
note: argument has type `worth_query_host::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_,
worth_query_testkit_certification_canonical::facade::certification::canonical_identity::DocumentRetentionSchema>`
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:217:10
    |
217 |     drop(request);
    |          ^^^^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#drop_non_drop
    = note: `-D clippy::drop-non-drop` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::drop_non_drop)]`

warning: `worth-query-court-decl-host` (test "worth_query_court_decl_host") generated 1 warning
error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;
1 warning emitted
```

### slice5-public-clippy2

```text
error: unnecessary use of `clone` to create a slice from a reference
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\cell_history
\checkpoint_cases.rs:52:5
   |
52 |     assert_eq!(live_at::current(&restored, 1), [survivor.clone()]);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#cloned_ref_to_slice_refs
   = note: `-D clippy::cloned-ref-to-slice-refs` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::cloned_ref_to_slice_refs)]`

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
capability\grant_commands\mod.rs:40:9
   |
40 |         std::thread::scope(|threads| {
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods
   = note: `-D clippy::disallowed-methods` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::disallowed_methods)]`

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
capability\grant_commands\mod.rs:41:38
   |
41 |             let competitor = threads.spawn(|| {
   |                                      ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\mod.rs:63:20
   |
63 |     let terminal = std::thread::scope(|threads| {
   |                    ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\mod.rs:64:34
   |
64 |         let competitor = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only
\managed_cleanup\mod.rs:120:29
    |
120 |     let (terminal, cause) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only
\managed_cleanup\mod.rs:122:14
    |
122 |             .spawn(|| {
    |              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\retained_capacity\mod.rs:24:24
   |
24 |         let terminal = std::thread::scope(|threads| {
   |                        ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\retained_capacity\mod.rs:27:36
   |
27 |             let pressure = threads.spawn(move || {
   |                                    ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
native_postures\mod.rs:59:20
   |
59 |     let terminal = std::thread::scope(|threads| {
   |                    ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
native_postures\mod.rs:60:33
   |
60 |         let advancing = threads.spawn(|| {
   |                                 ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
retained\native_postures\mod.rs:20:24
   |
20 |         let terminal = std::thread::scope(|threads| {
   |                        ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
retained\native_postures\mod.rs:21:37
   |
21 |             let advancing = threads.spawn(|| {
   |                                     ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\native_workflow_operations\cap
ability\grant_commands\mod.rs:40:9
   |
40 |         std::thread::scope(|threads| {
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\native_workflow_operations\cap
ability\grant_commands\mod.rs:41:38
   |
41 |             let competitor = threads.spawn(|| {
   |                                      ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:19:37
   |
19 |     let (reached, update, result) = std::thread::scope(|threads| {
   |                                     ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:20:30
   |
20 |         let worker = threads.spawn(|| document.refresh_context(&request, limits(1)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:57:29
   |
57 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:58:30
   |
58 |         let worker = threads.spawn(|| document.refresh_context(&stopped, limits(1)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\cancel
lation.rs:158:29
    |
158 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\cancel
lation.rs:160:30
    |
160 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handoff
.rs:58:33
   |
58 |         let (reached, result) = std::thread::scope(|threads| {
   |                                 ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handoff
.rs:60:34
   |
60 |             let worker = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handof
f.rs:118:29
    |
118 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handof
f.rs:120:30
    |
120 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recover
y.rs:87:19
   |
87 |     let reached = std::thread::scope(|threads| {
   |                   ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recover
y.rs:90:14
   |
90 |             .spawn(|| execution.advance(document, &host.request(&interrupted_principal, &stopped)));
   |              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: called `.err().expect()` on a `Result` value
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\resourc
es.rs:48:10
   |
48 |           .err()
   |  __________^
49 | |         .expect("member bound4 admits2, result bound1 must refuse");
   | |_______________^ help: try: `expect_err`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
   = note: `-D clippy::err-expect` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::err_expect)]`

error: using `clone` on type `CommitId` which implements the `Copy` trait
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\mod.rs:154:10
    |
154 |         [first.commit_reference().commit_id.clone()]
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try removing the `clone` call:
`first.commit_reference().commit_id`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#clone_on_copy
    = note: `-D clippy::clone-on-copy` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::clone_on_copy)]`

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ca
ncellation.rs:98:30
   |
98 |     let (reached, outcome) = std::thread::scope(|threads| {
   |                              ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ca
ncellation.rs:99:30
   |
99 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:45:32
   |
45 |     let (reached, won, lost) = std::thread::scope(|threads| {
   |                                ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:46:30
   |
46 |         let worker = threads.spawn(|| loser.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:98:38
   |
98 |     let (reached, updates, result) = std::thread::scope(|threads| {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:99:30
   |
99 |         let worker = threads.spawn(|| owner.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
ace.rs:165:29
    |
165 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
ace.rs:166:30
    |
166 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\re
covery.rs:82:29
   |
82 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\re
covery.rs:83:30
   |
83 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:332:25
    |
332 |     let (left, right) = std::thread::scope(|threads| {
    |                         ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:333:28
    |
333 |         let left = threads.spawn(|| {
    |                            ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:337:29
    |
337 |         let right = threads.spawn(|| {
    |                             ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:351:25
    |
351 |     let (left, right) = std::thread::scope(|threads| {
    |                         ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:352:28
    |
352 |         let left = threads.spawn(|| {
    |                            ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:356:29
    |
356 |         let right = threads.spawn(|| {
    |                             ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:26:29
   |
26 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:27:30
   |
27 |         let worker = threads.spawn(|| document.history_page(&stopped, limits(1), Some(&cursor)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:60:37
   |
60 |     let (reached, update, result) = std::thread::scope(|threads| {
   |                                     ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:61:30
   |
61 |         let worker = threads.spawn(|| document.history_page(&request, limits(1), None));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\cap
ture_race.rs:40:38
   |
40 |     let (reached, updates, result) = std::thread::scope(|threads| {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\cap
ture_race.rs:41:30
   |
41 |         let worker = threads.spawn(|| edit.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\han
doff.rs:29:29
   |
29 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\han
doff.rs:31:21
   |
31 |             threads.spawn(|| edit.advance(&host.runtime().request(&principal, &stopped_scope)));
   |                     ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\rec
overy.rs:35:30
   |
35 |     let (reached, outcome) = std::thread::scope(|threads| {
   |                              ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\rec
overy.rs:36:30
   |
36 |         let worker = threads.spawn(|| {
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\redo\handoff.
rs:35:41
   |
35 |         let (reached, action, result) = std::thread::scope(|threads| {
   |                                         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\redo\handoff.
rs:36:34
   |
36 |             let worker = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\can
cellation.rs:92:29
   |
92 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\can
cellation.rs:93:30
   |
93 |         let worker = threads.spawn(|| {
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\ca
ncellation.rs:167:29
    |
167 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\ca
ncellation.rs:168:30
    |
168 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: the loop variable `member_index` is only used to index `samples`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\group.rs:180:25
    |
180 |     for member_index in 0..2 {
    |                         ^^^^
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#needless_range_loop
    = note: `-D clippy::needless-range-loop` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::needless_range_loop)]`
help: consider using an iterator
    |
180 -     for member_index in 0..2 {
180 +     for <item> in samples.iter().take(2) {
    |

error: manual implementation of `.is_multiple_of()`
  -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:72:23
   |
72 |         let days = if depth % 2 == 0 && offset == 0 {
   |                       ^^^^^^^^^^^^^^ help: replace with: `depth.is_multiple_of(2)`
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#manual_is_multiple_of
   = note: `-D clippy::manual-is-multiple-of` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::manual_is_multiple_of)]`

error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its
contained lifetimes
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:217:5
    |
217 |     drop(request);
    |     ^^^^^^^^^^^^^
    |
note: argument has type `worth_query_host::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_,
worth_query_testkit_certification_canonical::facade::certification::canonical_identity::DocumentRetentionSchema>`
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:217:10
    |
217 |     drop(request);
    |          ^^^^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#drop_non_drop
    = note: `-D clippy::drop-non-drop` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::drop_non_drop)]`

warning: `worth-query-court-decl-host` (test "worth_query_court_decl_host") generated 1 warning
error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;
1 warning emitted
```

### slice5-public1

```text
error[E0369]: binary operation `==` cannot be applied to type
`worth_query_host::facade::primary_graph::ContentWriteDenial`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\cost.rs:26:5
   |
26 | /     assert_eq!(
27 | |         cause.downcast::<ContentWriteDenial>().unwrap(),
28 | |         ContentWriteDenial::Content(RelationalContentDenial::SizeLimit {
29 | |             observed: 49 + count * 8,
30 | |             maximum: 16 * 1024 * 1024
31 | |         })
32 | |     );
   | |     ^
   | |     |
   | |_____worth_query_host::facade::primary_graph::ContentWriteDenial
   |       worth_query_host::facade::primary_graph::ContentWriteDenial
   |
note: `worth_query_host::facade::primary_graph::ContentWriteDenial` does not implement `PartialEq`
  --> crates\worth-query-execution\src\domain_computation\primary_graph\content\denial.rs:27:1
   |
27 | pub enum ContentWriteDenial {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ `worth_query_host::facade::primary_graph::ContentWriteDenial` is defined in another
crate

For more information about this error, try `rustc --explain E0369`.
warning: `worth-query-court-decl-host` (test "worth_query_court_decl_host") generated 2 warnings
error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 1 previous error; 2
warnings emitted
```

### slice5-query-clippy1

```text
error: unnecessary use of `clone` to create a slice from a reference
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\cell_history
\checkpoint_cases.rs:52:5
   |
52 |     assert_eq!(live_at::current(&restored, 1), [survivor.clone()]);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#cloned_ref_to_slice_refs
   = note: `-D clippy::cloned-ref-to-slice-refs` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::cloned_ref_to_slice_refs)]`

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
capability\grant_commands\mod.rs:40:9
   |
40 |         std::thread::scope(|threads| {
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods
   = note: `-D clippy::disallowed-methods` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::disallowed_methods)]`

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
capability\grant_commands\mod.rs:41:38
   |
41 |             let competitor = threads.spawn(|| {
   |                                      ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\mod.rs:63:20
   |
63 |     let terminal = std::thread::scope(|threads| {
   |                    ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\mod.rs:64:34
   |
64 |         let competitor = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only
\managed_cleanup\mod.rs:120:29
    |
120 |     let (terminal, cause) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only
\managed_cleanup\mod.rs:122:14
    |
122 |             .spawn(|| {
    |              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\retained_capacity\mod.rs:24:24
   |
24 |         let terminal = std::thread::scope(|threads| {
   |                        ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
managed_cleanup\retained_capacity\mod.rs:27:36
   |
27 |             let pressure = threads.spawn(move || {
   |                                    ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
native_postures\mod.rs:59:20
   |
59 |     let terminal = std::thread::scope(|threads| {
   |                    ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
native_postures\mod.rs:60:33
   |
60 |         let advancing = threads.spawn(|| {
   |                                 ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
retained\native_postures\mod.rs:20:24
   |
20 |         let terminal = std::thread::scope(|threads| {
   |                        ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\application_graph\source_only\
retained\native_postures\mod.rs:21:37
   |
21 |             let advancing = threads.spawn(|| {
   |                                     ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\native_workflow_operations\cap
ability\grant_commands\mod.rs:40:9
   |
40 |         std::thread::scope(|threads| {
   |         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\native_workflow_operations\cap
ability\grant_commands\mod.rs:41:38
   |
41 |             let competitor = threads.spawn(|| {
   |                                      ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:19:37
   |
19 |     let (reached, update, result) = std::thread::scope(|threads| {
   |                                     ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:20:30
   |
20 |         let worker = threads.spawn(|| document.refresh_context(&request, limits(1)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:57:29
   |
57 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\context\hando
ff.rs:58:30
   |
58 |         let worker = threads.spawn(|| document.refresh_context(&stopped, limits(1)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\cancel
lation.rs:158:29
    |
158 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\cancel
lation.rs:160:30
    |
160 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handoff
.rs:58:33
   |
58 |         let (reached, result) = std::thread::scope(|threads| {
   |                                 ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handoff
.rs:60:34
   |
60 |             let worker = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handof
f.rs:118:29
    |
118 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\handof
f.rs:120:30
    |
120 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recover
y.rs:87:19
   |
87 |     let reached = std::thread::scope(|threads| {
   |                   ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recover
y.rs:90:14
   |
90 |             .spawn(|| execution.advance(document, &host.request(&interrupted_principal, &stopped)));
   |              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: called `.err().expect()` on a `Result` value
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\resourc
es.rs:48:10
   |
48 |           .err()
   |  __________^
49 | |         .expect("member bound4 admits2, result bound1 must refuse");
   | |_______________^ help: try: `expect_err`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
   = note: `-D clippy::err-expect` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::err_expect)]`

error: using `clone` on type `CommitId` which implements the `Copy` trait
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\mod.rs:154:10
    |
154 |         [first.commit_reference().commit_id.clone()]
    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try removing the `clone` call:
`first.commit_reference().commit_id`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#clone_on_copy
    = note: `-D clippy::clone-on-copy` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::clone_on_copy)]`

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ca
ncellation.rs:98:30
   |
98 |     let (reached, outcome) = std::thread::scope(|threads| {
   |                              ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ca
ncellation.rs:99:30
   |
99 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:45:32
   |
45 |     let (reached, won, lost) = std::thread::scope(|threads| {
   |                                ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:46:30
   |
46 |         let worker = threads.spawn(|| loser.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:98:38
   |
98 |     let (reached, updates, result) = std::thread::scope(|threads| {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\ra
ce.rs:99:30
   |
99 |         let worker = threads.spawn(|| owner.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
ace.rs:165:29
    |
165 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
ace.rs:166:30
    |
166 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\re
covery.rs:82:29
   |
82 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\re
covery.rs:83:30
   |
83 |         let worker = threads.spawn(|| owner.advance(&host.request(&principal, &stopped_scope)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:332:25
    |
332 |     let (left, right) = std::thread::scope(|threads| {
    |                         ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:333:28
    |
333 |         let left = threads.spawn(|| {
    |                            ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:337:29
    |
337 |         let right = threads.spawn(|| {
    |                             ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:351:25
    |
351 |     let (left, right) = std::thread::scope(|threads| {
    |                         ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:352:28
    |
352 |         let left = threads.spawn(|| {
    |                            ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\navigation\r
esources.rs:356:29
    |
356 |         let right = threads.spawn(|| {
    |                             ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:26:29
   |
26 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:27:30
   |
27 |         let worker = threads.spawn(|| document.history_page(&stopped, limits(1), Some(&cursor)));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:60:37
   |
60 |     let (reached, update, result) = std::thread::scope(|threads| {
   |                                     ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\paging\handof
f.rs:61:30
   |
61 |         let worker = threads.spawn(|| document.history_page(&request, limits(1), None));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\cap
ture_race.rs:40:38
   |
40 |     let (reached, updates, result) = std::thread::scope(|threads| {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\cap
ture_race.rs:41:30
   |
41 |         let worker = threads.spawn(|| edit.advance(&request));
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\han
doff.rs:29:29
   |
29 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\han
doff.rs:31:21
   |
31 |             threads.spawn(|| edit.advance(&host.runtime().request(&principal, &stopped_scope)));
   |                     ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\rec
overy.rs:35:30
   |
35 |     let (reached, outcome) = std::thread::scope(|threads| {
   |                              ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\rec
overy.rs:36:30
   |
36 |         let worker = threads.spawn(|| {
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\redo\handoff.
rs:35:41
   |
35 |         let (reached, action, result) = std::thread::scope(|threads| {
   |                                         ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\redo\handoff.
rs:36:34
   |
36 |             let worker = threads.spawn(|| {
   |                                  ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\can
cellation.rs:92:29
   |
92 |     let (reached, result) = std::thread::scope(|threads| {
   |                             ^^^^^^^^^^^^^^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\can
cellation.rs:93:30
   |
93 |         let worker = threads.spawn(|| {
   |                              ^^^^^
   |
   = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to worth-execution
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::scope`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\ca
ncellation.rs:167:29
    |
167 |     let (reached, result) = std::thread::scope(|threads| {
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: use of a disallowed method `std::thread::Scope::spawn`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\retention\ca
ncellation.rs:168:30
    |
168 |         let worker = threads.spawn(|| {
    |                              ^^^^^
    |
    = note: Threads and async executors belong only to the reviewed owning sites; computation belongs to
worth-execution
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: the loop variable `member_index` is only used to index `samples`
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\group.rs:180:25
    |
180 |     for member_index in 0..2 {
    |                         ^^^^
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#needless_range_loop
    = note: `-D clippy::needless-range-loop` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::needless_range_loop)]`
help: consider using an iterator
    |
180 -     for member_index in 0..2 {
180 +     for <item> in samples.iter().take(2) {
    |

error: manual implementation of `.is_multiple_of()`
  -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:72:23
   |
72 |         let days = if depth % 2 == 0 && offset == 0 {
   |                       ^^^^^^^^^^^^^^ help: replace with: `depth.is_multiple_of(2)`
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#manual_is_multiple_of
   = note: `-D clippy::manual-is-multiple-of` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::manual_is_multiple_of)]`

error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its
contained lifetimes
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:217:5
    |
217 |     drop(request);
    |     ^^^^^^^^^^^^^
    |
note: argument has type `worth_query_host::facade::application_entry::WorthQueryApplicationRequest<'_, '_, '_,
worth_query_testkit_certification_canonical::facade::certification::canonical_identity::DocumentRetentionSchema>`
   -->
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\cost\document_history\growth\scenario.rs:217:10
    |
217 |     drop(request);
    |          ^^^^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#drop_non_drop
    = note: `-D clippy::drop-non-drop` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::drop_non_drop)]`

warning: `worth-query-court-decl-host` (test "worth_query_court_decl_host") generated 1 warning
error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;
1 warning emitted
```
