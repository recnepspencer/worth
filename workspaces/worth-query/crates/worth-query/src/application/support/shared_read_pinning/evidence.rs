use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPinningInventoryEvidence {
    failure_count: usize,
    missing_operation_count: usize,
    inventory_digest: String,
}

impl WorthQuerySharedReadPinningInventoryEvidence {
    pub fn new(
        failure_count: usize,
        missing_operation_count: usize,
        inventory_digest: impl Into<String>,
    ) -> Self {
        Self {
            failure_count,
            missing_operation_count,
            inventory_digest: inventory_digest.into(),
        }
    }

    pub fn total_failure_count(&self) -> usize {
        self.failure_count + self.missing_operation_count
    }

    pub fn inventory_digest(&self) -> &str {
        &self.inventory_digest
    }

    #[cfg(test)]
    pub fn with_missing_operation_for_sabotage(&self) -> Self {
        Self {
            missing_operation_count: self.missing_operation_count + 1,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPinningHostileMatrixEvidence {
    certified: bool,
    matrix_digest: String,
}

impl WorthQuerySharedReadPinningHostileMatrixEvidence {
    pub fn new(certified: bool, matrix_digest: impl Into<String>) -> Self {
        Self {
            certified,
            matrix_digest: matrix_digest.into(),
        }
    }

    pub fn certified(&self) -> bool {
        self.certified && !self.matrix_digest.is_empty()
    }

    pub fn matrix_digest(&self) -> &str {
        &self.matrix_digest
    }

    #[cfg(test)]
    pub fn uncertified_for_sabotage(&self) -> Self {
        Self {
            certified: false,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPortabilityEvidence {
    scoped_thread_digest: String,
}

impl WorthQuerySharedReadPortabilityEvidence {
    pub fn proven(scoped_thread_digest: impl Into<String>) -> Self {
        Self {
            scoped_thread_digest: scoped_thread_digest.into(),
        }
    }

    pub fn proven_by_scoped_thread(&self) -> bool {
        !self.scoped_thread_digest.is_empty()
    }

    pub fn scoped_thread_digest(&self) -> &str {
        &self.scoped_thread_digest
    }

    #[cfg(test)]
    pub fn missing_for_sabotage(&self) -> Self {
        Self {
            scoped_thread_digest: String::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadStaleBasisDenialEvidence {
    typed_denial_digest: String,
}

impl WorthQuerySharedReadStaleBasisDenialEvidence {
    pub fn proven(typed_denial_digest: impl Into<String>) -> Self {
        Self {
            typed_denial_digest: typed_denial_digest.into(),
        }
    }

    pub fn proven_by_typed_denial(&self) -> bool {
        !self.typed_denial_digest.is_empty()
    }

    pub fn typed_denial_digest(&self) -> &str {
        &self.typed_denial_digest
    }

    #[cfg(test)]
    pub fn missing_for_sabotage(&self) -> Self {
        Self {
            typed_denial_digest: String::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPinningCounterEvidence {
    committed_read_hot_path_lock_count: usize,
    orphaned_generation_count: usize,
    unretired_pin_count: usize,
    shared_read_mint_row_clone_count: usize,
    reader_derived_evaluation_count: usize,
    counter_digest: String,
}

impl WorthQuerySharedReadPinningCounterEvidence {
    pub fn new(
        committed_read_hot_path_lock_count: usize,
        orphaned_generation_count: usize,
        unretired_pin_count: usize,
        shared_read_mint_row_clone_count: usize,
        reader_derived_evaluation_count: usize,
    ) -> Self {
        let counter_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_usize(
                    WorthQueryEvidenceTag::new("committed_read_hot_path_lock_count"),
                    committed_read_hot_path_lock_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("orphaned_generation_count"),
                    orphaned_generation_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("unretired_pin_count"),
                    unretired_pin_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("shared_read_mint_row_clone_count"),
                    shared_read_mint_row_clone_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("reader_derived_evaluation_count"),
                    reader_derived_evaluation_count,
                )
                .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
                .as_str()
                .to_string();
        Self {
            committed_read_hot_path_lock_count,
            orphaned_generation_count,
            unretired_pin_count,
            shared_read_mint_row_clone_count,
            reader_derived_evaluation_count,
            counter_digest,
        }
    }

    pub fn residue_count(&self) -> usize {
        self.committed_read_hot_path_lock_count
            + self.orphaned_generation_count
            + self.unretired_pin_count
            + self.shared_read_mint_row_clone_count
            + self.reader_derived_evaluation_count
    }

    pub fn counter_digest(&self) -> &str {
        &self.counter_digest
    }

    #[cfg(test)]
    pub fn with_unretired_pin_for_sabotage(&self) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            self.committed_read_hot_path_lock_count,
            self.orphaned_generation_count,
            self.unretired_pin_count + 1,
            self.shared_read_mint_row_clone_count,
            self.reader_derived_evaluation_count,
        ))
    }
}
