# Relational Directory format retirement fixtures

The Phase 0A writer at `873c04ff88` captured original Relational records from
these Store Directory histories. The schema has one entity kind,
`legacy.directory.record`, and one schema version, `legacy.directory@1`.

| Fixture | CHK24 covered position | Uncompacted V10 journal positions |
| --- | ---: | --- |
| `ordinary.bin` | 1, ordinary create | 2, ordinary create |
| `merge-checkpoint.bin` | 4, ordinary commit above merge at 3 | 5, ordinary create |
| `merge-tail.bin` | 1, ordinary create | 2, branch create-row. 3, merge to main. |

Each file holds the original checkpoint blob, followed by three counted lists
of blobs: the checkpoint's original selected members, the ordered canonical
journal, and the ordered branch transition journal. Each count or blob byte
length is a little-endian `u32`. The frames and member bytes are unchanged.

The court publishes these opaque records through Store into a temporary
Directory, closes it, then reopens from the selected checkpoint and the
uncompacted journals. Store owns its physical layout and WAL for each test run.
