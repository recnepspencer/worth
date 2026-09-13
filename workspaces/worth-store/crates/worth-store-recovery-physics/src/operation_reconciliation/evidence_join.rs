use std::collections::BTreeSet;

use super::{
    classify_binding_freshness, OperationReconciliationDenial, ReconciledOperationFate,
    RecoveryOperationEvidenceInput,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciledOperationFates {
    operations: Box<[ReconciledOperationFate]>,
    counts: [u64; 4],
}

pub fn reconcile_operation_fates(
    selected_checkpoint_generation: u64,
    mut inputs: Vec<RecoveryOperationEvidenceInput>,
    maximum_bindings: u64,
) -> Result<ReconciledOperationFates, OperationReconciliationDenial> {
    if inputs.len() as u64 > maximum_bindings {
        return Err(OperationReconciliationDenial::BindingLimit);
    }
    inputs.sort_unstable_by_key(|input| input.identity);
    let mut seen = BTreeSet::new();
    let mut operations = Vec::with_capacity(inputs.len());
    let mut counts = [0_u64; 4];
    for input in inputs {
        if input.lease_issuance_generation >= input.lease_expiry_generation {
            return Err(OperationReconciliationDenial::InvalidLease);
        }
        if !seen.insert(input.identity) {
            return Err(OperationReconciliationDenial::DuplicateIdentity);
        }
        let freshness = classify_binding_freshness(
            selected_checkpoint_generation,
            input.lease_expiry_generation,
        );
        if freshness != input.freshness {
            return Err(OperationReconciliationDenial::FreshnessMismatch);
        }
        counts[input.fate as usize] += 1;
        operations.push(ReconciledOperationFate::new(input, freshness));
    }
    Ok(ReconciledOperationFates {
        operations: operations.into_boxed_slice(),
        counts,
    })
}

impl ReconciledOperationFates {
    pub(super) fn from_operations(operations: Vec<ReconciledOperationFate>) -> Self {
        let mut counts = [0_u64; 4];
        for operation in &operations {
            counts[operation.fate() as usize] += 1;
        }
        Self {
            operations: operations.into_boxed_slice(),
            counts,
        }
    }

    pub(super) fn into_operations(self) -> Box<[ReconciledOperationFate]> {
        self.operations
    }

    pub fn operations(&self) -> &[ReconciledOperationFate] {
        &self.operations
    }
    pub const fn acknowledged_durable(&self) -> u64 {
        self.counts[0]
    }
    pub const fn durable_unacknowledged(&self) -> u64 {
        self.counts[1]
    }
    pub const fn proven_no_effect(&self) -> u64 {
        self.counts[2]
    }
    pub const fn indeterminate(&self) -> u64 {
        self.counts[3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RecoveryBindingFreshness, RecoveryOperationEvidenceInput, RecoveryOperationFate,
        RecoveryOperationIdentity,
    };

    #[test]
    fn every_fate_and_owner_sampled_freshness_is_counted_exactly() {
        let fates = [
            RecoveryOperationFate::AcknowledgedDurable,
            RecoveryOperationFate::DurableUnacknowledged,
            RecoveryOperationFate::ProvenNoEffect,
            RecoveryOperationFate::Indeterminate,
        ];
        let inputs = fates
            .into_iter()
            .enumerate()
            .map(|(index, fate)| {
                let ordinal = index as u64 + 1;
                RecoveryOperationEvidenceInput::new(
                    RecoveryOperationIdentity::new([1; 16], 1, 1, ordinal, [ordinal as u8; 32])
                        .unwrap(),
                    [ordinal as u8; 32],
                    1,
                    4,
                    RecoveryBindingFreshness::Retained,
                    fate,
                )
            })
            .collect();
        let reconciled = reconcile_operation_fates(3, inputs, 4).unwrap();
        assert_eq!(reconciled.acknowledged_durable(), 1);
        assert_eq!(reconciled.durable_unacknowledged(), 1);
        assert_eq!(reconciled.proven_no_effect(), 1);
        assert_eq!(reconciled.indeterminate(), 1);
    }

    #[test]
    fn caller_substitution_of_the_sample_is_rejected() {
        let input = RecoveryOperationEvidenceInput::new(
            RecoveryOperationIdentity::new([1; 16], 1, 1, 1, [1; 32]).unwrap(),
            [2; 32],
            1,
            4,
            RecoveryBindingFreshness::Retained,
            RecoveryOperationFate::Indeterminate,
        );
        assert_eq!(
            reconcile_operation_fates(4, vec![input], 1),
            Err(OperationReconciliationDenial::FreshnessMismatch),
            "MUTANT_PREDICATE:c8-freshness-expiry-boundary-widened"
        );
    }
}
