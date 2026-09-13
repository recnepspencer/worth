use std::num::NonZeroU64;

use self::failure_outcome::{
    pressure_basis, project_candidate_admission_failure, project_residency_failure,
};
use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::{
        PhysicalDataDispatchFailureCause, PhysicalDataDispatchOutcome, PhysicalDataFrameKind,
    },
    record_serving::{
        residency::candidate_frame_residency::{
            CandidateFrameCoordinate, CandidateFrameDeclaration, CandidateFrameRole,
            CandidateFrameSet,
        },
        PhysicalRecordPressureBasis,
    },
    WalDurablePhysicalMutation,
};

mod candidate_cleanup;
mod effect_progression;
mod failure_outcome;

impl RecordPublicationDirector {
    pub(super) fn dispatch_wal_durable_data(
        &self,
        durable: WalDurablePhysicalMutation,
    ) -> PhysicalDataDispatchOutcome {
        let Some(runtime) = self.runtime.upgrade() else {
            return PhysicalDataDispatchOutcome::NotStarted {
                durable,
                cause: PhysicalDataDispatchFailureCause::PublicationAuthorityReleased,
            };
        };
        if let Some(cause) = self.dispatch_admission_failure(&durable) {
            return PhysicalDataDispatchOutcome::NotStarted { durable, cause };
        }
        let declaration = match candidate_declaration(&durable, self.current_root().generation()) {
            Some(declaration) => declaration,
            None => {
                return PhysicalDataDispatchOutcome::NotStarted {
                    durable,
                    cause: PhysicalDataDispatchFailureCause::CandidateFrameContract(
                        crate::physical_runtime::CandidateFrameContractViolation::UnexpectedFrame,
                    ),
                };
            }
        };
        let bytes = NonZeroU64::new(declaration.total_frame_bytes())
            .expect("a WAL-bound data plan has nonempty frames");
        let store_basis = PhysicalRecordPressureBasis::for_store(self.durability.store_identity());
        let allocation = match self.residency.begin_foreground_write_operation(bytes) {
            Ok(allocation) => allocation,
            Err(denial) => {
                return PhysicalDataDispatchOutcome::NotStarted {
                    durable,
                    cause: project_residency_failure(denial.into(), self.generation, store_basis),
                }
            }
        };
        let candidate_basis = declaration
            .declarations()
            .first()
            .copied()
            .and_then(|frame| {
                pressure_basis(
                    self.durability.store_identity(),
                    frame.coordinate(),
                    frame.length(),
                )
            })
            .unwrap_or(store_basis);
        let residency = match self
            .residency
            .begin_candidate_publication(&allocation, declaration)
        {
            Ok(residency) => residency,
            Err(denial) => {
                return PhysicalDataDispatchOutcome::NotStarted {
                    durable,
                    cause: project_candidate_admission_failure(
                        denial,
                        self.generation,
                        candidate_basis,
                    ),
                }
            }
        };
        effect_progression::DurableFrameDispatch::new(
            self,
            store_basis,
            runtime.executor.record_serving_media(),
        )
        .execute(durable, residency)
    }

    fn dispatch_admission_failure(
        &self,
        durable: &WalDurablePhysicalMutation,
    ) -> Option<PhysicalDataDispatchFailureCause> {
        let identity = durable.mutation_identity();
        if identity.store_identity() != self.durability.store_identity() {
            return Some(PhysicalDataDispatchFailureCause::ForeignStore);
        }
        if identity.runtime_identity() != self.durability.runtime_identity() {
            return Some(PhysicalDataDispatchFailureCause::StaleRuntime);
        }
        if durable.appended().reserved().signal_profile() != self.signal_profile {
            return Some(PhysicalDataDispatchFailureCause::SignalProfileMismatch);
        }
        None
    }
}

fn candidate_declaration(
    durable: &WalDurablePhysicalMutation,
    root_generation: u64,
) -> Option<CandidateFrameSet> {
    let frames = durable
        .data_frames()
        .iter()
        .map(|frame| {
            let target = frame.basis().target();
            let coordinate = target.coordinate();
            CandidateFrameDeclaration::new(
                candidate_role(target.kind()),
                CandidateFrameCoordinate::new(coordinate.artifact(), coordinate.offset()),
                coordinate.length(),
            )
        })
        .collect::<Option<Vec<_>>>()?;
    CandidateFrameSet::new(root_generation, frames)
}

pub(super) const fn candidate_role(kind: PhysicalDataFrameKind) -> CandidateFrameRole {
    match kind {
        PhysicalDataFrameKind::InlinePage => CandidateFrameRole::InlinePage,
        PhysicalDataFrameKind::ExtentChunk => CandidateFrameRole::ExtentChunk,
    }
}
