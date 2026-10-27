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
| Embedded native export | existing region courts; class expansion pending | existing region courts; class expansion pending | envelope matrix | pending subject expansion | no interning | pending subject expansion | event pages (CHK11) | pending subject expansion |
| Portable memory checkpoint | class expansion pending | class expansion pending | envelope matrix | pending subject expansion | no interning | pending subject expansion | event pages (CHK11) | pending subject expansion |
| Native external capture | must not load | must not load | complete index, no body load | carry directory custody | carry directory custody | carry page directory | event pages (CHK11) | carry image directory |
| Independent merge planning/preparation | damage court; class expansion pending | audit pending | nonhead merge matrix | audit pending | shared bucket matrix | audit pending | audit pending | audit pending |
| Transaction merge-parent validation | current preparation matrix | must stay untouched | nonhead merge matrix | current preparation matrix | shared bucket matrix | audit pending | audit pending | shared index matrix |
| Node point/history/aspect/graph/divergence doors | operation Full matrix | operation Full matrix | selected history matrix | operation Full matrix | no interning | lineage matrix | event matrix | no lookup |
| Cached derived-index use, pin and candidate preparation | operation Full matrix | operation Full matrix | audit pending | operation Full matrix | no interning | no lookup | no lookup | index matrix |
| Derived-index build/reconstruction | expansion pending | expansion pending | expansion pending | expansion pending | no interning | audit pending | audit pending | expansion pending |
| Replay envelope access, execution and ancestor receipts | historical snapshot Full guard | historical snapshot Full guard | envelope matrix in checkpoint and tail shapes | guarded snapshot admission | shared bucket admission | identity frontier courts | event pages (CHK11) | replay cache reconstruction |
| Historical lineage, selected observations, aspect wrappers, graphs, divergence and exports | operation region guard | operation Full guard | envelope courts | operation symbol guard | node matrix | event matrix, 11 doors | cache subject expansion pending |
| Tail replay | existing replay courts; full class expansion pending | existing replay courts; full class expansion pending | actual journal readmission | shared symbol admission | shared bucket admission | identity frontier courts | event pages (CHK11) | replay cache reconstruction |

Current refusal family sizes: Full reads 576, envelope/export/checkpoint/replay/receipt 156,
lineage node 42, lineage events 66, cached index 36, current Head/ID symbol 72, intern bucket 30,
remote invariant Head 18, aspect history 144, nonhead merge 18: 1,158 cells. Positive current and
remote-invariant unavailable-history families add 54 cells. These totals are an
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
