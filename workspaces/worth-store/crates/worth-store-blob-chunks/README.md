# worth-store-blob-chunks

Provides the downward mechanism contracts for native chunk trees: chunking,
digest comparison, sequence/frontier laws, and pure placement and reachability
planning. These contracts are not evidence that a Store operation executed.

This crate does not depend on `worth-store`, issue Store allocations, own a
physical runtime, or persist blob bytes. The production owner is
`worth-store::physical_runtime`, accessed through
`ServingPhysicalRuntime::blobs()`. It joins these mechanisms to C.5 records,
WAL/root publication, protected reads, and the Store scheduler. Blob bytes
live in the Store's extent arenas, not beside it in an external file server.

Mechanism surfaces for resume, dedupe, movement, and reachability do not imply
those operations are available from the serving Store. See the
[C.11 specification](../../../../plans/worth-store/physical-reconstruction-c11-layout-index-and-native-blob-adoption.md)
for their integration phases and the
[caller contract](../../../../plans/worth-store/physical-blobs-and-chunk-trees.md)
for currently implemented operations.
