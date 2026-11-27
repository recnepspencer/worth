//! Compile-time `!Clone` fences on the large persistent-naming state owners
//! (RULES §4, layer 2).
//!
//! Each owner grows with names, history or group members, so an implicit
//! copy is O(state) on whatever path calls it. `assert_not_clone!(T)` is a
//! trait-ambiguity assertion: it compiles only while `T` does not implement
//! `Clone`, so adding `#[derive(Clone)]` (or a manual impl) to a fenced owner
//! fails the build. The copies the platform does make are named entry points
//! (`duplicate_for_branch_fork`, `duplicate_for_reconstruction`).

/// Resolvable for `Marker = ()` on every type, and also for `Marker =
/// IsClone` on `Clone` types, where `_` becomes ambiguous.
pub(crate) trait AmbiguousIfClone<Marker> {
    const PROBE: () = ();
}

impl<T> AmbiguousIfClone<()> for T {}

/// The marker only a `Clone` type resolves.
pub(crate) struct IsClone;

impl<T: Clone> AmbiguousIfClone<IsClone> for T {}

/// Fails to compile when `$owner` implements `Clone`.
macro_rules! assert_not_clone {
    ($owner:ty) => {
        const _: () = <$owner as $crate::lineage::clone_fences::AmbiguousIfClone<_>>::PROBE;
    };
}

pub(crate) use assert_not_clone;

#[cfg(test)]
mod tests {
    // The positive control: an unfenced `Clone` type resolves `IsClone`, so
    // the fence's ambiguity is what rejects a `Clone` owner.
    #[test]
    fn a_clone_type_resolves_the_clone_marker() {
        fn probe<T: super::AmbiguousIfClone<super::IsClone>>() {}
        probe::<Vec<u8>>();
    }
}
