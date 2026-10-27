# Residency phase 2 matrix ledger

This is the current coverage inventory. Final clean-tree lane execution is
still outstanding. Slice logs are intermediate evidence, not final acceptance.
The chronological boundary findings, corrections and slice results are in
`residency-phase-2.md`.

Each refusal family tests six independently stated source outcomes: missing
named member, source unavailable, damaged, integrity mismatch, limit exceeded
and memory exhausted. Only missing named content and proven damage are sticky;
transport, allocation and limit failures retry. Refusals preserve the exact
subject and cause and check journal frontier and offered artifacts. Preparation
courts additionally check position reservations and provisional effects.

| Operation family | Head | History | Envelope | ID symbols | Intern bucket | Lineage node/events | Derived image |
|---|---|---|---|---|---|---|---|
| Current transaction and projection points, metadata and retirement | current table | fault-armed untouched | metadata only | current table | no interning | no old-page lookup | no lookup |
| Exact projection kind/record scans | scan table | fault-armed untouched | metadata only | current closure; historical-only page untouched | no interning | no lookup | no lookup |
| Versioned scans, historical visibility, statistics, inspection and retention | Full table | Full table | selected metadata or history door below | Full table | no interning | explicit lineage doors below; record inspection uses no lineage port | no cache lookup except indexed door below |
| Projection adjacency/frontiers | anchor and remote table | historical only; exact untouched | metadata only | anchor and remote table | no interning | no lookup | no lookup |
| Transaction validation/candidate/commit and remote uniqueness | preparation tables | fault-armed untouched | merge-parent table | current value admission | bucket table | new nodes/events derive from staged Head; no old-page lookup | index table |
| Historical merge planning | merge member table | merge member table | nonhead merge table | merge member table | shared preparation bucket table | fault-armed untouched | fault-armed untouched |
| Independent merge execution | accepted Head draft probes; planning supplies read inventory | historical planning above | admitted planning inventory | accepted dictionary custody | bucket table | fault-armed untouched | cache-unused table; explicit rebuild checked afterward |
| Entity/relation/lineage aspect history, including trace | aspect table | aspect table | aspect table | aspect table | no interning | node/event tables | no lookup |
| Bounded patches/streams and envelope surfaces | no lookup | no lookup | envelope table | envelope-owned meanings | no interning | envelope-owned facts | optional artifact surface covered |
| Retained export | no lookup | no lookup | envelope table | envelope-owned meanings | no interning | envelope-owned facts | rebuildable cache omitted |
| Embedded export and portable memory checkpoint | Full table | Full table | envelope table | Full table | no interning | node/event tables | index table |
| External native checkpoint capture | no load | no load | index only; no body load | placement custody | placement custody | page-directory custody | image placement custody |
| Node point/history/aspect/graph/divergence/observation doors | operation region guard | historical guard when needed | selected history table | operation symbol guard | no interning | node table; event table for resolution | no generation-cache port |
| Cached generation reads, pin and field-read preparation | operation guard | Full pin guard | metadata only; no body port | operation guard | no interning | no lookup | index table |
| Historical index reconstruction | historical-source table | historical-source table | fault-armed unused body | historical-only symbol table | no interning | no lookup | historical-source table |
| Replay execution and ancestor receipts | historical snapshot guard | historical snapshot guard | checkpoint/tail envelope table | snapshot admission | shared issuance | identity-frontier courts and event table | replay cache reconstruction |
| Tail recovery | tail table refusal | tail table untouched | checkpoint-head refusal; supplied-tail body untouched | tail table refusal | tail table untouched | tail node/event pages untouched | tail table refusal |

The table names both consumed and unconsumed subjects. An absent call to a
subject's port is not modeled as a storage failure. In particular, uniqueness
scans authoritative selected-state fields; merge's target identity index is
built from selected records; neither consumes a derived-generation payload.
Derived generations are explicitly refreshed. Historical merge planning uses
versioned source, target and ancestor reads and therefore requires Full; its
applied native commit draft uses the shared Head probe mechanism.

## Table inventory

These count loop cases, not test functions or every subsidiary assertion.
Additional setup/no-load/genuine-absence checks within a case are not counted
again. Growth tests use headroom; zero I/O and identity uniqueness assertions
are semantic requirements, not growth estimates.

| Table | Typed refusal cases | Fault-armed untouched cases |
|---|---:|---:|
| Full/versioned reads and exact projection scans | 744 | 48 |
| Envelope/export/checkpoint/replay/receipt | 156 | 0 |
| Lineage node, nine doors | 54 | 0 |
| Lineage event, eleven doors | 66 | 0 |
| Generation cache/build/export/pin/inspection, fourteen doors | 84 | 0 |
| Current Head/ID-symbol reads, twenty-two doors | 264 | 132 |
| Intern buckets, five preparation doors | 30 | 0 |
| Remote invariant preparation, three doors | 18 | 18 |
| Aspect history | 144 | 0 |
| Nonhead merge-parent history | 18 | 0 |
| Projection adjacency, both bases and local/remote subjects | 390 | 258 |
| Historical index source | 24 | 6 |
| Historical merge planning members | 18 | 12 |
| Independent merge unused generation cache | 0 | 6 |
| Tail recovery, nine inputs | 24 | 30 |
| **Total** | **2,034** | **510** |

The tail table uses actual producer Head/history, ID pages, bucket-page lists,
node/event pages, a built generation, the covered checkpoint-head position and
supplied canonical tail positions. A refused staging runtime leaves the
receiving runtime untouched. A new recovery plan owns new cells; repairing
transport and retrying a fresh plan does not reuse or clear a damaged live cell.

## Additional acceptance evidence

- Probe/identity: `cold_candidates`, `cold_symbols` and preparation-round courts
  prove misses cannot escape, abandoned identities are spent, diagnostics and
  counters settle only for accepted rounds, and warm/cold meanings agree.
- Head fence: residency type-level/compile-fail courts reject historical access
  through Head, while Full demand folds history and returns the retained value.
- Selective envelopes: `cold_envelopes` and `cold_envelope_lifetime` prove bounded
  query reads, sticky journal damage, live/checkpoint/tail release and paired
  memory-only retention. Historical index source selection leaves its unused
  envelope cold with all six failure classes armed.
- Paging: symbol view-level and ring dictionaries, node range locality, event
  range/replacement courts, concurrent page admission and per-runtime fork
  admission prove reopen loads only touched meanings. `cold_unique_symbols`
  re-admits a real paged field in cold/Head-warm/Full-warm runtimes, gets the
  original ID, and rejects its duplicate with a typed invariant conflict.
- Custody: dictionary, index, node/event placement courts retain older carriers
  across replacement; the outstanding-reservation checkpoint court preserves
  late-published low identities. Full folding uses the authenticated dependency
  directory rather than eagerly restoring old rows.
- Accepted reads: pin/projection and deterministic Head callback courts retain
  their immutable values after later transport damage; fresh reads refuse.
- Lineage success adapters assert actual point nodes, resolved chains, aspect
  entries, graph membership, divergence membership and visible entity identity.
- Priority regressions remain unchanged:
  `complexity_budget_unobserved_write_copies_no_partition` and
  `publication_port_performs_one_exact_candidate_and_reports_the_loser_stale`.
- Real home agreement compares cold, warm and fully warmed reads to a memory
  reference, including observations retained across later commits/checkpoints.
  Its complete home-oracle lane remains required, along with every other final
  lane in the brief.

Final evidence belongs in `target/lane-evidence/<lane>.log`, with exact command,
HEAD, clean/dirty status, UTC start/end and exit code. The reviewer report must
list final counts and log paths and identify every unrun lane as unrun.
