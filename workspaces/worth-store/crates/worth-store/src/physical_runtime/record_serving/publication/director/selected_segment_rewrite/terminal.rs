use worth_proof::TransitionOutcome;

use super::super::durable_preparation::{
    canonical_request_failure, PhysicalMutationPreparationAdmission,
};
use crate::physical_runtime::{
    PhysicalMutationPreparationOutcome, PhysicalMutationPreparationSuccess,
};

pub(in crate::physical_runtime::record_serving::publication::director) fn admitted_terminal(
    admitted: PhysicalMutationPreparationAdmission,
) -> PhysicalMutationPreparationOutcome {
    match admitted {
        PhysicalMutationPreparationAdmission::Prepared(_) => canonical_request_failure(),
        PhysicalMutationPreparationAdmission::ProvenNoEffect(terminal) => {
            TransitionOutcome::success(PhysicalMutationPreparationSuccess::ProvenNoEffect(terminal))
                .into()
        }
        PhysicalMutationPreparationAdmission::Completed(terminal) => {
            TransitionOutcome::success(PhysicalMutationPreparationSuccess::Completed(terminal))
                .into()
        }
        PhysicalMutationPreparationAdmission::Indeterminate(terminal) => {
            TransitionOutcome::success(PhysicalMutationPreparationSuccess::Indeterminate(terminal))
                .into()
        }
    }
}
