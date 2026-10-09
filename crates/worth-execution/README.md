# Execution fixed payload custody

The composition root constructs the process execution authority once. Its
`charged_memory_bytes: Option<u64>` selects an optional ceiling for execution
payload reservations. `Some` preserves the selected value, including zero;
`None` keeps checked ledger arithmetic while omitting only that process ceiling.
Every request and ancestor retains its explicit finite budget. This does not
make any algorithm budget unbounded or intercept process allocations.

`ExecutionResourceLease::controlled_child(cancellation, deadline)` retains the
parent's exact request policy and uses the earlier parent/caller deadline.
It adds the supplied cancellation token to the existing ancestor constraints
without mutating the parent. A payload reservation owns its ledger custody;
dropping this child does not release storage still owned by an array or buffer.

Byte and typed array builders require the same explicit
`ExecutionAllocationPolicy::SystemAllocation` or `Execution(&lease)`. There is
no default, pool construction or fallback. A supplied process lease admits
physical storage; it grants no graph/model permission. The private fixed owner
checks `Layout::array::<T>(element_count)`, checks cancellation/deadline, admits
the exact payload-byte reservation, then attempts a fallible fresh Global
allocation using pinned allocator-api2 0.2.21. For non-zero-sized elements that
implementation requests that layout and records exactly that capacity. This
guarantee is scoped to the pinned fresh allocation, not generic/std Vec behavior.
Zero-sized types have allocator capacity `usize::MAX` but zero payload bytes;
the independently retained logical count still bounds initialization and seal.
Over-aligned zero-sized consuming iteration keeps the original Vec and uses
safe pop in either direction; such values have no distinct stored
representation. Non-zero-sized iteration uses the pinned owning Vec iterator.

`ExecutionByteBuffer::allocate(total, policy)` retains byte authoring: append
within the declared length, overwrite only written regions, and poll live
status between 64 KiB copy chunks. Sealing requires exact fill, shares the same
backing without shrink or reallocation, and exposes immutable bytes. Clones of
`ExecutionImmutableBytes` share one allocation and its noncloneable reservation.

`ExecutionArrayBuilder::<T>::allocate(element_count, policy)` moves each value
through `push` and seals only after exact fill and a live-status check. Push
consumes the input, including on refusal; a stop detected after the move may
leave an initialized prefix which cannot seal. Neither the builder nor
`ExecutionArray<T>` implements Clone, exposes mutable storage, accepts an
arbitrary Vec, or extracts a reservation. Sharing is explicit through
`Arc<ExecutionArray<T>>`; it neither clones T nor charges another allocation.
The immutable slice preserves authored order and T's own interior mutability.
Consuming iteration moves T without allocation, preserving the original
backing and reservation even after exhaustion until the iterator drops.

Elements and their allocation drop before the reservation. Custody can outlive
the originating lease or World; it grants no further permission to execute.
Moved-out array values own their separate nested resources. The physical owner
charges only the contiguous element layout, not allocations nested inside T.

`ExecutionImmutableBytes::from_external_bytes(Arc<Box<[u8]>>)` retains already
allocated external bytes without copying or retrospectively admitting them.
`charged_payload_bytes()` returns `None` for external/system storage and the
actual retained ticket amount for execution storage, including `Some(0)` for
empty or zero-sized payloads. Equality compares values, never admission mode
or allocation identity. The Query checkpoint capture policy remains a semantic
re-export of this one physical policy, with no independent behavior.

Allocator, Arc and ledger metadata, nested element heaps, native checkpoint
image/codec allocations, imported buffers, decoding tables, upstream domain
maps/sets, temporary provider collections and compression/transport remain
separate owners. Only callers that explicitly admit this contiguous backing
retain its charge. This is neither full heap/RSS accounting nor removal of
domain count gates.
