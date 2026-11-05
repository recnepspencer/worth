//! Conservative node payload quote for the checkout's reviewed standard tree.
//!
//! rust-toolchain.toml pins 1.99.1, commit
//! 58cf8f9c35b355199727627d07ae34a4e2846d67. The reviewed alloc implementation:
//! https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/library/alloc/src/collections/btree/node.rs
//! has eleven keys/values and twelve child pointers. Map invariants permit at
//! most one empty leaf root; all other nodes contain an entry. At most n+1
//! nodes can therefore retain n entries. This quote also covers incremental
//! construction, not bulk FromIterator's separate staging/sorting allocation.
use std::alloc::Layout;

use crate::branch::RelationalBranchRootCaptureDenial as Denial;

pub(crate) fn retained_tree_bytes<K, V>(entries: usize) -> Result<u64, Denial> {
    retained_tree_layout_bytes(entries, Layout::new::<K>(), Layout::new::<V>())
}

fn retained_tree_layout_bytes(entries: usize, key: Layout, value: Layout) -> Result<u64, Denial> {
    let pointer = Layout::new::<Option<std::ptr::NonNull<()>>>();
    let short = Layout::new::<u16>();
    let alignment = key
        .align()
        .max(value.align())
        .max(pointer.align())
        .max(short.align());
    // Rounding EACH component to maximum alignment bounds every field order
    // of the reviewed Rust-layout leaf, plus the C-layout internal edges.
    let fields = [
        pointer.size(),
        short.size(),
        short.size(),
        key.size()
            .checked_mul(11)
            .ok_or(Denial::AllocationLayoutOverflow)?,
        value
            .size()
            .checked_mul(11)
            .ok_or(Denial::AllocationLayoutOverflow)?,
        pointer
            .size()
            .checked_mul(12)
            .ok_or(Denial::AllocationLayoutOverflow)?,
    ];
    let mut node = 0usize;
    for field in fields {
        node = node
            .checked_add(padded(field, alignment)?)
            .ok_or(Denial::AllocationLayoutOverflow)?;
    }
    let bytes = padded(node, alignment)?
        .checked_mul(
            entries
                .checked_add(1)
                .ok_or(Denial::AllocationLayoutOverflow)?,
        )
        .ok_or(Denial::AllocationLayoutOverflow)?;
    u64::try_from(bytes).map_err(|_| Denial::AllocationLayoutOverflow)
}

fn padded(bytes: usize, alignment: usize) -> Result<usize, Denial> {
    bytes
        .checked_add(alignment - 1)
        .map(|bytes| bytes & !(alignment - 1))
        .ok_or(Denial::AllocationLayoutOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_tree_quote_refuses_capacity_overflow() {
        assert!(matches!(
            retained_tree_bytes::<u64, u64>(usize::MAX),
            Err(Denial::AllocationLayoutOverflow)
        ));
    }

    #[test]
    fn standard_tree_quote_includes_all_nodes_and_alignment_padding() {
        #[repr(align(64))]
        struct Aligned {
            _bytes: [u8; 64],
        }
        let one = retained_tree_bytes::<Aligned, u8>(0).unwrap();
        let many = retained_tree_bytes::<Aligned, u8>(100).unwrap();
        assert!(one >= 11 * size_of::<Aligned>() as u64);
        assert_eq!(one % align_of::<Aligned>() as u64, 0);
        assert_eq!(many, one * 101);
    }

    #[test]
    fn standard_tree_quote_requires_the_reviewed_compiler_implementation() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-toolchain.toml");
        let toolchain = std::fs::read_to_string(path).unwrap();
        assert_eq!(
            toolchain.lines().find(|line| line.starts_with("channel")),
            Some("channel = \"1.99.1\"")
        );
    }
}
