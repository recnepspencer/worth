# IN PROGRESS: main-stack round2

Reviewed source HEAD: `b2ba8dc3b102d71b899052d7ccc95a0718b76bbf`. No push. All source is committed; the following
evidence-only commit records this handoff without changing compiled inputs.

Round2 retains admitted data and managed cleanup behind original-funded owned
storage. Core: 8,760 -> 104 bytes; typed admission: 8,912 -> 256; managed cleanup: 7,672 -> 16;
publication cleanup: 8,880 -> 1,224. Data (8,712 bytes) and cleanup state (4,680 bytes) remain real owned
backing. Funding, cancellation, drop order and retry custody remain intact.

## Commits this resume

- 052437570f: admitted-core backing, prepaid cleanup and all callers/currentness.
- 4d063ecf8d: missing private-data constructor import.
- 29b4ecb5ce: cleanup fixture retains its actual original-admission binding.
- 3f9ed1751f: scalar observations at defining owners; funding proofs in Bridge court.
- d6c3a928ab: lifetime court uses public clones; private fresh-instance proof stays.
- 3c710d6f6a: optional Store field observations mirror owner gates; import cleanup.
- b2ba8dc3b1: correct direct-snapshot fixture; restore original import reachability.

Previously accepted750f746567(spec tracks),805533bb7f(type census) and
d2994fc364(round1) are unchanged. No new release build this resume.

## Evidence and outcomes

Complete command/HEAD/pass/fail/ignored/log table: layout-resume-lanes.md and JSON.
Full logs remain at those exact worktree paths. No ignored test was selected or
executed; ignored inventory metadata is explicitly separated in the table.

Final focused cases: Root 4/0/0, Query guards 3/0/0, Bridge funding 3/0/0,
Store guard 1/0/0. Corrected cleanup fixture 1/0/0. Reused original Store guards 2/0/0,
Bank guard 1/0/0, consumer guard 1/0/0 and other Querycleanup controls are recorded
in the table. The two Bank parents are 2/0/0 with RUST_MIN_STACK absent.
Consumer-root main exits0. All three genuine dev example mains still fail with 0xc00000fd.
This is H10, integration-owned, and is not waived by C2. No stack/thread/profile
workaround, test weakening or caller-permission/baseline addition.

Final frame census:1,658 -> 1,563 functions>=32KiB;375 -> 323>=128KiB,
485 -> 470 at64--128KiB,798 -> 770 at32--64KiB;860 -> 793>=64KiB.
Six unchanged executable/PDB pairs are hash-verified, changed Rootcourt refreshed;
zero unresolved/demangle failures. All4 ordinary PE reserves stay1048576.
Earlier release comparison is reused with its recorded attribution limitation.

Formatting, dirty/slice caps(823Rustfiles) and agent-context pass. Boundary exits1:
17 owned findings removed;15 held findings remain(8P-A Native,7P-C World/Store).
Bare Store check exits0 with 10 unresolved warnings(3 unused-import plus 7 dead-code).
layout-round2-open-items.md records every exact warning and held boundary path.
Strict lint, broad normal lanes, Home/Store direct controls, six Native courts,
scale, exhaustive oracle and scenario suite were not run this resume.

## Failures and corrections

All complete diagnostic sets were read before their correction. Product-side
constructor migration missed one import(E0422); fixed. The cleanup fixture used
bare intent instead of its admission-enriched original receipt binding; corrected
the fixture, retaining all Native/drift/send/no-reexecution assertions. Scalar
collectors and foreign request construction were in the wrong test topology;
relocated without changing ceilings or claims. The lifetime court called a
private method(E0624); use public clones and keep private canonical proof.
Three Store observations incorrectly assumed optional fields always existed;
mirror actual cfg gates. Registered-source fixture setup lacked its required
adapter; remove that irrelevant lane, retaining the real direct snapshot read.
Import-only narrowing exposed cascading warnings; reverse it. All historical
failed logs are retained. During the initial failed compile the owned runner
also refused ambiguous lib/lib-test diagnostic attribution; the real compiler
output remains intact. No runner fix or successful failed-unit attribution is
claimed. A byte-level restore guard stopped on CRLF/Git-LF;
completed all five files after normalized comparison before compiling.

## Independent review and remaining work

Fresh round2_final_source_and_evidence_review: bounded source APPROVE; overall
completion REJECT. Finding1(P1):H10 open, exact three main failures above.
No further material source defect found. No reviewer build/edit. Detailed verdict
in layout-round2-independent-review.md. Earlier Store-cfg/topology findings are
resolved; warnings and held paths remain explicit, not exceptions.

This resume:13 compiling Cargo checks(10pass,3historical failures),0 Clippy;12 focused builds;
12 test invocations(11 nextest + 1 direct Bank),4 ordinary executions and3 diagnostic
reexecutions. Four owned constitution-tool invocations across the two phase
checks are separate from these counts. All builds serial; disk stayed above
25GiB; no cleanup or extra target directory. Next work is the remaining main-path
phase-carrier layout, listed in round2-remaining-main-frames.md, plus the explicit
warning/held-owner obligations. This report does not certify task completion.
