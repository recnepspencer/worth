# worth-store-physical-backend

Owns the narrow S.1 physical backend facade: append, read, scan, locate,
publish root manifest, and reopen from persisted bytes.

The backend facade deals in framed bytes and physical references. It does not
own semantic commit legality, artifact meaning, or legacy compatibility policy.

For durability, the backend owns admitted filesystem capability and exact media
mechanics. `PhysicalDurabilityAdmissionBasis` can be issued only from one
qualified media generation with matching file-sync, directory-sync, and
durable-rename claims. Backend receipts describe completed mechanisms; they do
not know mutation grouping, pageLSN policy, checkpoint policy, current-root
authority, or final Store acknowledgment.

Ordinary execution remains behind the Store's C.4 media owner. There is no
public backend runtime that callers may compose into a second durability lane.

Windows path/listing application-heap envelopes qualify the exact stock compiler
builds `rustc 1.94.0 (4a4ef493e 2026-03-02)` and
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`. The 1.99 qualification retains the
existing bounds. Its source audit covers pinned winx 0.36.4's fixed handle-path
buffer; Rust's [Windows directory iterator](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/library/std/src/sys/fs/windows.rs),
[verbatim PathBuf reconstruction](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/library/std/src/path.rs),
WTF-8 conversion, Vec growth, and Arc layout; and cap-primitives 4.0.2's
single-normal-component open and device-name uppercase check. Target, dependency,
flag, and compiler-cache restrictions on path qualification remain in force.
These are requested application-heap bounds, excluding allocator bookkeeping and
OS storage, rather than a process memory bound.

Isolated allocator probes exercise long Unicode directory listings and retained
name/roster capacity, plus small, expanding-uppercase, supplementary-Unicode,
near-maximum and missing-file component opens. Their cumulative requested bytes
count positive realloc growth; they do not measure transient old/new allocation
overlap. That overlap remains covered by the source-derived envelopes.
