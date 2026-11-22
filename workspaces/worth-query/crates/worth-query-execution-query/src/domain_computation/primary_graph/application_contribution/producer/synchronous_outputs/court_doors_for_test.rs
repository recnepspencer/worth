//! Sealed test doors over the wave's private orphan backstop.

/// The first derived task unreached from `roots` through `prerequisites`
/// (the orphan backstop's walk, uncharged).
#[doc(hidden)]
pub fn first_orphan_for_test(
    roots: &[usize],
    derived: &[bool],
    prerequisites: &[Vec<usize>],
) -> Option<usize> {
    super::wave::first_orphan(roots, derived, prerequisites, &mut |_| Ok::<_, std::convert::Infallible>(()))
        .unwrap_or_else(|never| match never {})
}
