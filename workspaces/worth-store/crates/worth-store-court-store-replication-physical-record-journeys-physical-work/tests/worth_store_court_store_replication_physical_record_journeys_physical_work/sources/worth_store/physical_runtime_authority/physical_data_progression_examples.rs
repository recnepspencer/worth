use worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission;
use worth_store::physical_runtime::{
    PhysicalDataDispatchOutcome, PhysicalDataSettlementOutcome, WalDurablePhysicalMutation,
};

fn dispatch_and_settle_exact_data(
    submission: &CertificationPhysicalRecordSubmission,
    durable: WalDurablePhysicalMutation,
) {
    match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => {
            match dispatched.settle_exact_effects() {
                PhysicalDataSettlementOutcome::Settled(settled) => {
                    let _completed_identity = settled.mutation_identity();
                }
                PhysicalDataSettlementOutcome::InspectionRequired { dispatched, cause } => {
                    let _inspection_basis = (dispatched.mutation_identity(), cause);
                }
            }
        }
        PhysicalDataDispatchOutcome::NotStarted { durable, cause } => {
            let _preserved_authority = (durable.mutation_identity(), cause);
        }
        PhysicalDataDispatchOutcome::Suspended(retry) => {
            let _retained_progress = (
                retry.durable().mutation_identity(),
                retry.completed_effects().len(),
                retry.cause(),
            );
            let _preserved_authority = retry.into_durable();
        }
        PhysicalDataDispatchOutcome::Indeterminate(indeterminate) => {
            let _inspection_basis = (
                indeterminate.mutation_identity(),
                indeterminate.completed_frames(),
                indeterminate.cause(),
            );
        }
    }
}

fn main() {
    let _ = dispatch_and_settle_exact_data;
}
