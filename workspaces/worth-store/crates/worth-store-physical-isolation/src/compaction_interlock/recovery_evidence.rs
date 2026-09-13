use worth_store_recovery_physics::{PhysicalRecoveryResidueKind, SelectedCompactionProduct};

/// Recovery evidence consumed by the physical compaction interlock.
///
/// A selected product is already admitted by recovery source precedence. A
/// residue observation is descriptive evidence that must remain a denial and
/// cannot be promoted into a visible product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionRecoveryEvidence {
    SelectedProduct(SelectedCompactionProduct),
    Residue(PhysicalRecoveryResidueKind),
}

impl CompactionRecoveryEvidence {
    pub const fn selected_product(product: SelectedCompactionProduct) -> Self {
        Self::SelectedProduct(product)
    }

    pub const fn residue(kind: PhysicalRecoveryResidueKind) -> Self {
        Self::Residue(kind)
    }
}
