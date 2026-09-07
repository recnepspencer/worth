# WORTH Store offline integrity observer

This crate owns the implementation-independent C.9 offline observation lane.
It reads Store content from a closed or isolated Store root through ordinary
read-only OS file handles, applies finite caller-declared resource bounds, and
emits a descriptive version-1 report. It grants no runtime admission, recovery choice, repair,
quarantine mutation, reachability mutation, or semantic-service authority.

## Independent observation contract

`observe_store` follows one staged root-protocol chain:

1. independently read and SHA-256-check the fixed `namespace/identity` trust
   anchor under the same acquisition bounds;
2. enumerate `families/records/` under the entry/depth/time budget;
3. acquire the fixed `root-current.selector` and `root-previous.selector` slots
   and lawful-looking selector candidates under the byte/open-file/symlink
   budgets;
4. independently validate the durable-frame envelope, supported physical-format
   declaration, CRC32C relation, exact selector role, selector identity,
   Store binding, declared reciprocal linkage, and duplicated embedded format;
5. use only still-intact, exactly scoped canonical selector fields to address
   `families/records/roots/root-<generation>.manifest`;
6. independently validate every recognized manifest candidate (including
   unaddressed damaged or unsupported evidence), while only selectors address
   missing-root obligations, checking envelope, generation scope,
   capacity, shape, and root/segment/free-space pointer encodings; and
7. descend only from intact canonical roots into independently parsed routing,
   segment-membership, and free-space trees; bind pages to membership-issued
   segment/page/generation ranges and extents to routing-issued record bindings;
8. read bootstrap catalogs, WAL segments, pending physical work, and checkpoint
   streams independently, including record checksums and selective checkpoint
   footer aggregate digests; and
9. retain typed artifact outcomes, exact localization, completeness, and
   traversal/decoder counters in an `OfflineIntegrityReport`.

The current and previous selectors are separate reader entry points even though
they share one persisted grammar. Missing fixed slots, missing addressed roots,
unaddressed root candidates, hostile unknown entry kinds, hard-link aliases,
and duplicate selector/root identities remain explicit. Per-path validation
outcomes are non-lossy; physical-alias and semantic-identity duplication are
orthogonal evidence rather than replacement damage verdicts. A checksum
mismatch localizes the failed relation to the complete frame and checksum field;
the lone damaged artifact cannot honestly infer whether the covered bytes or the
stored checksum were edited.

The observer imports only `worth-store-physical-format::integrity_declarations`
and foundational descriptive protocol vocabulary. SHA-256, CRC32C, framing,
field decoding, traversal, physical-alias/duplicate detection, classification,
and report projection are implemented here. Runtime codecs, runtime validators, Store, recovery,
repair, maintenance, operations, and the legacy verifier are forbidden sources.

## Command

```text
physical_store_integrity_observer observe \
  --store-root <closed-or-isolated-store-root> \
  --report <path-outside-store-root|-> \
  --max-entries <n> \
  --max-bytes <n> \
  --max-open-files <n> \
  --max-depth <n> \
  --max-symlinks <n> \
  --max-elapsed-ms <n> \
  --max-report-bytes <n>
```

`-` emits the report to stdout. A file target uses create-new semantics and is
never overwritten. Its parent must already exist, and `--max-open-files` must
be at least `2` so the parent identity guard remains live through create-new.
The current root-protocol layout requires a traversal high water of `5`: the
canonical Store root, each lexical ancestor through `families/records/roots`,
and the observed root file remain open together. A lower declared limit yields
typed `OpenFileBoundExceeded` evidence without opening the next handle.

Before observation and again immediately before file creation, the implementation
canonicalizes the Store root and the report parent. Relative paths and dot
segments are normalized; resolvable symlinked parents are canonicalized; path
containment is component-wise, so a lexical lookalike such as `store-copy` is
not confused with `store`. A target equal to or beneath the canonical Store is
rejected. Store-root and destination-parent filesystem identities are compared
under the caller's shared elapsed budget. No output file or directory is
created before this proof succeeds. Emission is a separate post-observation
operation and does not retroactively alter traversal counters.

The caller owns the operational fact that the Store is closed or isolated.
Directory and file symlinks, including unrecognized entries, are classified and
never followed outside the canonical Store. Every entry admitted before a
directory bound is retained, sorted, and reported; exhaustion never discards an
already admitted candidate prefix.
Physical aliases are cached by filesystem identity before another semantic
parser runs, so one physical file is content-read once across roles and
families. On Windows, the observer opens non-reparse handles for the canonical
Store root, every lexical ancestor, and the final file. Those handles deny
delete sharing (and the final file also denies write sharing) while path-based
identity is queried, bytes are read, and the binding is rechecked. It then
obtains volume-qualified identity through bounded `fsutil file queryFileID`
and volume-serial adapters. Adapter output is capped by `max-bytes`, and the
child is killed at the remaining shared `max-elapsed-ms`; unavailable identity is a typed
`Indeterminate` outcome, never a canonical-path fallback. A detectable length,
timestamp, or identity change across bounded acquisition is `Indeterminate`;
the observer does not claim detection when a filesystem preserves all compared
snapshot metadata, and it does not retry until a convenient answer appears.

## Version-1 report wire

`encode_offline_integrity_report` emits deterministic UTF-8 JSON with:

- protocol `store.physical.integrity-observation`, version `1`, compatibility
  window `[1,1]`, and role `offline-root-observer`;
- executable, process, run, scenario, and independently anchored Store identity;
- every declared entry/byte/open-file/depth/symlink/elapsed/report bound;
- consumed entries, bytes, files, high water, depth, symlink refusals,
  duplicates, missing artifacts, checksum calculations, decoder entries, and exact
  report bytes;
- ordered artifact path, family, identity, generation, range, validator outcome,
  damage/unsupported/unknown/indeterminate detail, duplicate evidence, and
  blast radius; and
- `complete`, `bound_exhausted`, or `indeterminate` completeness.

Rendering counts every output byte while retaining at most the declared report
budget, so report-size exhaustion is enforced before an oversized allocation or
emission. The wire contains no admission proof, recovery option, repair token, owner
disposition, or reconciled verdict. Report-size exhaustion prevents emission and
does not alter the already observed Store bytes.

## Comparison

```text
physical_store_integrity_observer compare \
  --runtime-observation <runtime-v1.json> \
  --offline-observation <offline-v1.json> \
  --report <new-external-report.json|->
```

Comparison admits version-1 reports only, with distinct runtime/offline roles,
executables and processes, matching nonempty Store identity and scenario, and
independent run identities. Input size is capped at 16 MiB per report, artifact
count at 100,000 per report, and output at 64 MiB. Library callers can lower or
raise these explicit finite limits. Invalid schemas, compatibility windows,
duplicate scopes, or resource overflow fail closed.

The comparison wire is `store.physical.integrity-comparison` version `1`. It
retains both complete input observations and reports exact differences in
presence, identity, generation, byte range, outcome/localization, and duplicate
evidence. Same-posture differences remain visible. There is no winner,
consensus, repair policy, or admission result. Comparison never opens a Store;
the operator owns excluding the Store root from its output. The command rejects
an input as output and uses create-new semantics, including hard-link aliases.

Page and chunk observations describe embedded ranges, not invented standalone
files. Missing expected child containers remain damage at the expected child
scope. Unaddressed historical roots and unreachable residue remain Unknown:
their own bytes cannot establish the parent scope needed to call them Intact.
Unknown directories are reported without probing their content as canonical
artifacts. Observation completeness describes traversal, not universal integrity.
