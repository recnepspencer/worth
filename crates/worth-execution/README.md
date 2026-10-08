# Execution payload-byte custody

The composition root constructs the process execution authority once. Its
`charged_memory_bytes: Option<u64>` selects an optional ceiling for execution
payload reservations. `Some` preserves the selected value, including zero;
`None` keeps checked ledger arithmetic while omitting only that process ceiling.
Every request and ancestor retains its explicit finite budget. This does not
make any algorithm budget unbounded or intercept process allocations.

`ExecutionByteBuffer::allocate(total, policy)` receives an explicit
`ExecutionByteAllocationPolicy::SystemAllocation` or `Execution(&lease)`.
There is no default, pool construction or fallback. The execution path checks
the exact payload layout, checks cancellation/deadline, admits its reservation,
then attempts a fallible fresh Global allocation using pinned allocator-api2
0.2.21. That implementation requests exactly `Layout::array::<u8>(total)` and
records that capacity. This guarantee is scoped to that implementation and
fresh allocation, rather than inferred from generic/std Vec behavior.

The opaque builder appends only within reserved capacity and overwrites only
already-written regions. Both operations poll live status between bounded
copy chunks. Sealing requires exact consumption, shares the existing backing
without shrink or reallocation, and exposes immutable bytes. Clones share one
allocation and its noncloneable reservation. The backing deallocates before
its reservation drops. It can outlive the originating lease or World; that
custody grants no further permission to execute.

`ExecutionImmutableBytes::from_external_bytes(Arc<Box<[u8]>>)` retains already
allocated external bytes without copying or retrospectively admitting them.
`charged_payload_bytes()` returns `None` for external/system storage and the
actual retained ticket amount for execution storage, including `Some(0)`.
Equality compares byte values, never admission mode or allocation identity.

Only requested payload backing is charged. Allocator, Arc and ledger metadata,
native checkpoint image/codec allocations, imported buffers, decoding tables,
domain readsets/candidates and compression/transport remain separate owners.
This is neither full heap/RSS accounting nor removal of domain count gates.
