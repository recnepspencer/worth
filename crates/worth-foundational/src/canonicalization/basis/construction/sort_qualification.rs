//! The admitted stable-sort bound belongs to the configured compiler and target.
mod refusal_control;
#[cfg(test)]
mod tests;
pub use refusal_control::with_refused_canonical_sort_qualification_for_test;

/// Actual compiler/target context when admitted stable-sort qualification refuses.
/// This is diagnostic data, not a compiler qualification or a canonical artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalSortImplementationDenial {
    compiler_release: &'static str,
    compiler_commit: &'static str,
    target_pointer_width: u32,
}

impl CanonicalSortImplementationDenial {
    pub const fn compiler_release(&self) -> &'static str {
        self.compiler_release
    }
    pub const fn compiler_commit(&self) -> &'static str {
        self.compiler_commit
    }
    pub const fn target_pointer_width(&self) -> u32 {
        self.target_pointer_width
    }
}

pub(super) fn require_qualified_sort() -> Result<(), CanonicalSortImplementationDenial> {
    if cfg!(canonical_verified_std_sort)
        && matches!(usize::BITS, 32 | 64)
        && !refusal_control::qualification_refused()
    {
        return Ok(());
    }
    Err(CanonicalSortImplementationDenial {
        compiler_release: env!("WORTH_CANONICAL_SORT_COMPILER_RELEASE"),
        compiler_commit: env!("WORTH_CANONICAL_SORT_COMPILER_COMMIT"),
        target_pointer_width: usize::BITS,
    })
}
