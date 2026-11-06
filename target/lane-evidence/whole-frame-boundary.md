# Whole-frame boundary before implementation

Source remains clean `dae78e6ff7`. This is a diagnostic boundary review, not an
approved implementation design. No product edits or new dev builds occurred.

The user requires an exhaustive owner-function census before the one designed
batch. `frame-census.md` now inventories 1658 named owner functions from all
unwind entries in seven existing dev binaries. All 572 entries initially
without exact PDB names were resolved by section-contribution/retained-COFF
matching. Source module, section length, checksum and all nonrelocation bytes
agree; every matching retained copy has the same normalized owner path.
No nearest-symbol attribution was used. Full exact match provenance and
PE/PDB GUID/age and SHA-256 identities are retained in the identified JSON.

The committed consumer dev profile disables debug information for Query
execution and topology. Test profiles use debug=0. Their PDBs retain public
names but do not provide all Rust function/type information. A Windows-only
type enumeration found `_CONTEXT`, not Rust UDT layouts. The one authorized
release build also uses the unmodified committed profile. It succeeds; 11 named
owner functions remain >=64 KiB and 72 are 32–64 KiB. Optimization does not
erase the problem. Release has 88 explicitly listed unnamed entries whose
temporary example objects were removed by Cargo; no second build is made.
That limits release owner totals to a lower bound, not the negative answer to
whether all release frames shrink below 32 KiB. Dev attribution is complete.

Authoritative inputs are compiler/linker unwind records, exact PDB coordinates
and current owner source. Frames from older existing artifacts are not claimed
to describe freshly compiled HEAD inputs. Source-level probable causes in the
inventory are explicitly hypotheses, not measured type sizes.

The first census review rejected the original inventory, correctly: it counted
lossy display aliases twice, misparsed arrows inside generic types, missed an
implementation for a foreign receiver, and had not examined retained COFF
objects. All four diagnostic defects are corrected together, and matching
three-example dev/release bands now recompute subset maxima. Do not use the
superseded 1831 count or the earlier lower-bound stop premise. The corrected
census requires fresh review before implementation design.

Potential representation boundaries verified in source (not yet a plan):

- Signal `conditional_execution/execution.rs` embeds a complete optional
  `SignalConditionalExecutionSlot` inside every evaluation admission. The slot
  owns a full evaluation partition, even when the option is None. Initial slot
  storage is admitted by `SignalEvaluationPartition::initial_heap_charge` before
  `from_admitted_retained` constructs it. Any boxing must preserve that exact
  charge, transfer, retry and unwind custody.
- Signal `ConditionalEvaluationDraft` retains previous evaluation storage by
  value; rejected evaluation also owns it by value. Its fork admission already
  reserves `capacity::<SignalEvaluationStorage>(1)` before effects. A later
  phase must consume its predecessor without losing the original reservation.
- Signal `AuthorityState` contains a whole SignalGraph by value. Branch cells,
  owner transfer packets and forks carry it. Any backing change must account
  for the new allocation in branch admission, snapshot and fork paths; an
  uncharged box is not a fix.
- Query history edit phases contain native operation, completed, terminal and
  cleanup owners inline. Owning refusals return Self. Construction is effect
  free and has no fresh execution argument; introducing a late allocation
  without original grant provenance would be dishonest. Existing Query
  bootstrap one-cell backing is an analogue, not permission to invent funding.
- Receipts carry the unique performed-change witness separately from their
  descriptive clones. A layout change cannot give cloned/recorded receipts a
  witness or live result. Reusing existing immutable allocation is preferable
  to adding uncharged post-effect boxing.

Protected files are listed by owner in the inventory. No P-A Native storage,
checkpoint or root file, or P-C World/Lifecycle file, has changed. A true
cross-owner contradiction must be reported instead of routed around.

Current verification count: 0 check/clippy, 1 release build, 0 test runs.
All seven dev scans and release scans are read-only diagnostic executions.
