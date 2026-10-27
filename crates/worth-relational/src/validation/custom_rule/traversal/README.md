# Structural traversal ownership

The private breadth-first driver serves two policies. Bounded scope widening
retains its existing touched-derived quotas and maximum depth. Complete reads
use one relation kind and exhaust the frontier or return a typed refusal. They
consume the request sealed on their native view and retain the original meter
with owned output. The typed invariant adapter consumes those native facts;
it does not run another traversal. Snapshot/operation traversal belongs to the
native pinned-query executor and its separate visibility and packet-memory
lifecycle, rather than importing custom-rule internals.

`complete/retention_extent.rs` admits collection backing before growth. Its
implementation bounds assume the supported Rust 1.94 standard collections:
the [BTree node layout](https://raw.githubusercontent.com/rust-lang/rust/1.94.0/library/alloc/src/collections/btree/node.rs)
and [RawVec capacity growth](https://raw.githubusercontent.com/rust-lang/rust/1.94.0/library/alloc/src/raw_vec/mod.rs).
These bounds cover this owner's storage, including initial capacities and
growth; they are not application work units or a new product allowance. A
toolchain collection-layout change must revisit them.

Read observations count complete-walk boundary acquisitions. Internal native
visibility and filtering work remains on the original native work meter.
An acquired entity rejected by declared access still counts; a refusal before
acquisition does not. Refusals retain observations and never a partial answer.
