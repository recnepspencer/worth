# Residency phase 2 matrix ledger

This is the working coverage inventory, not a completion declaration. Each
covered refusal family tests missing member, source unavailable, damaged,
integrity mismatch, limit exceeded and memory exhausted. Proven damage is
sticky; the other classes retry. Every refusal checks the journal frontier and
publication offers. Positive cases check real producer contents rather than
accepting `None` or an empty answer. Growth assertions use headroom.

| Guarded operation family | Region Head | Region history | Envelope | ID symbols | Intern bucket | Lineage node | Lineage events | Derived index |
|---|---|---|---|---|---|---|---|---|
| Transaction entity/relation reads and current writes | current matrix | must stay untouched | metadata only | current matrix | write bucket matrix | no lookup | no lookup | no lookup |
| Remote invariant preparation | preparation matrix | must stay untouched | nonhead merge matrix | admitted current values | bucket matrix | audit pending | audit pending | index matrix |
| Full scans, historical visibility, statistics, inspection, retention | Full read matrix | Full read matrix | operation-specific below | Full read matrix | no interning | node doors below | audit pending | index doors below |
| Entity/relation/lineage aspect history, with and without trace | aspect matrix | aspect matrix | aspect matrix | aspect matrix | no interning | node matrix | event matrix (real Replace chain) | no lookup |
| Envelope surfaces, bounded patches and patch stream | no region lookup | no region lookup | envelope matrix | envelope-owned text | no interning | envelope-owned facts | envelope-owned facts | optional artifact surface covered |
| Retained artifact export | no region lookup | no region lookup | envelope matrix | envelope-owned text | no interning | envelope-owned facts | envelope-owned facts | omitted rebuildable cache |
| Embedded native export | Full read matrix | Full read matrix | envelope matrix | Full read matrix | no interning | node matrix | event matrix | index matrix |
| Portable memory checkpoint | Full read matrix | Full read matrix | envelope matrix | Full read matrix | no interning | node matrix | event matrix | index matrix |
| Native external capture | must not load | must not load | complete index, no body load | carry directory custody | carry directory custody | carry page directory | event pages (CHK11) | carry image directory |
| Independent merge planning/preparation | damage court; class expansion pending | audit pending | nonhead merge matrix | audit pending | shared bucket matrix | audit pending | audit pending | audit pending |
| Transaction merge-parent validation | current preparation matrix | must stay untouched | nonhead merge matrix | current preparation matrix | shared bucket matrix | audit pending | audit pending | shared index matrix |
| Node point/history/aspect/graph/divergence doors | operation Full matrix | operation Full matrix | selected history matrix | operation Full matrix | no interning | lineage matrix | event matrix | no lookup |
| Cached derived-index use, pin and candidate preparation | operation Full matrix | operation Full matrix | audit pending | operation Full matrix | no interning | no lookup | no lookup | index matrix |
| Derived-index build/reconstruction | Full read matrix | Full read matrix | metadata receipt only; historical court pending | Full read matrix | no interning | no lookup | no lookup | index matrix |
| Replay envelope access, execution and ancestor receipts | historical snapshot Full guard | historical snapshot Full guard | envelope matrix in checkpoint and tail shapes | guarded snapshot admission | shared bucket admission | identity frontier courts | event pages (CHK11) | replay cache reconstruction |
| Historical lineage, selected observations, aspect wrappers, graphs, divergence and exports | operation region guard | operation Full guard | envelope courts | operation symbol guard | node matrix | event matrix, 11 doors | cache subject expansion pending |
| Tail replay | existing replay courts; full class expansion pending | existing replay courts; full class expansion pending | actual journal readmission | shared symbol admission | shared bucket admission | identity frontier courts | event pages (CHK11) | replay cache reconstruction |

Current refusal family sizes: Full reads 792, envelope/export/checkpoint/replay/receipt 156,
lineage node 54, lineage events 66, index cache/build/export 60, current Head/ID symbol 264, intern bucket 30,
remote invariant Head 18, aspect history 144, nonhead merge 18: 1,602 cells. Positive current and
remote-invariant unavailable-history families add 150 cells. These totals are an
inventory of the tables; final logs must prove the final tree's execution.

Additional acceptance courts already present include probe miss/identity
reservation tests, the Head type-level historical-access fence, selective
journal envelope readmission, dictionary directory placement custody, paired
memory/durable envelope lifetime, complete live index capture/reopen, and the
two priority regression tests. Recovered tail lifetime is covered in both real journal shapes. Still required:
index-build/checkpoint cells and the final complete home-oracle lane. The
new real Store-home agreement court passes cold/warm/fully-warmed reads and
observations retained across commits and checkpoints. The
final lane inventory and logs will be recorded after these gaps close.


Lineage event slice adds a real two-replacement fixture with unrelated ranges,
shared-cell/per-runtime admission, concurrent demand, range-boundary locality,
actual byte corruption in both body-first and export-first orders, older-page
placement custody, and native capture from a fork whose shared pages are
resident but query index is empty. The latter first emitted an empty event
member directory; capture now reads counted resident page witnesses directly.
The 66-cell event table passed in the complete 1,990-test native run. The outstanding
reserved-ID checkpoint court exposed lost low IDs and now passes with CHK12
exact page membership. Separate proofs refuse forged interior membership,
duplicate publication IDs and overflowing high bits without another load.


Public projection scan rows now include both exact and historical bases for
bounded entity/relation kind scans and record projection callbacks. Their first
cold Head returned WorkerFailed after a resident-only panic. All eight doors
now pass the same three-subject, six-class table (144 added refusal cells).
The complete native run passes 1,994 tests with 31 ignored. Pure metadata point
and adjacency projection surfaces, historical index selection and the tail
recovery class matrix remain under audit; final lanes remain unrun.


Current projection points add six doors (borrowed entity projection, admitted
entity projection, ordinary/admitted entity revision, relation revision and
current adjacency IDs): 72 refusal cells and 36 positive history-untouched
cells. Actual callbacks remain at zero on a failed demand. The original
transaction/preparation doors and their assertions are retained.


Accepted-pin damage isolation now crosses actual embedded transport for all six
failure classes. A raw view rejects proven damage; an accepted pin's metadata,
point projection and Full scan keep their immutable meaning with zero further
loads. Unproven transport failures preserve the healthy resident raw view.
Resident basis construction also has a type-level fence against Default and
infallible promotion from a selected metadata-only basis. The native full run
passes 1,996 tests with 31 ignored; the subsequent added type fence passes its
own focused lane. All final clean-tree evidence remains outstanding.


Issued-snapshot kind/status/aspect reads and retirement add four current doors
(48 refusal cells and 24 unavailable-history positives). The same table now
covers 22 current doors, including genuine absence only after successful guard
admission. Separate actual deletion/reopen evidence proves original creation
and deletion versions are in Head. Head-only metadata pin courts prove cold
construction refuses without I/O, shared authenticated Head bytes do not admit
a fork's dictionary, and accepted meaning survives a later commit or proven
transport damage. Public pin types are aggregated through the native facade;
post-publication optional witness sealing consumes this no-load capability.
