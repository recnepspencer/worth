//! Actual cold World custody and original Query keys admit the service phase.
use super::*;
use crate::domain_computation::primary_graph::provider::aftermath_causality::readmission as corrections;
use worth_proof::CustodyReadmitted;

#[derive(Debug)]
pub enum QueryHomeCustodyReadmissionDenial {
    CorrectionMaterial(crate::facade::correction_material::QueryCorrectionMaterialDenial),
    ProgramSupport(crate::facade::program_support::QueryProgramSupportDenial),
    Activation(super::QueryHomeProgramActivationDenial),
    ProgramHistorySource(
        RuntimeWorldServiceDenial<worth_runtime_world::facade::WorldReadmissionDenial>,
    ),
    Correction(corrections::QueryHomeCorrectionLineageDenial),
    Execution(worth_execution::WorkCeilingDenial),
    World(RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>),
    Recovery(RuntimeWorldRecoveryDenial),
    UnclassifiedWorldCustody,
    OriginalBindingUnavailable,
    ForeignOriginalAffinity,
    OriginalMemoryAdmission,
    ReleasedSource(worth_runtime_world::facade::WorldDurableMediumDenial),
}
pub struct QueryHomeCustodyReadmissionRefused<Schema, Program> {
    query: ReadmittedHomeQueryService<Schema, Program>,
    cause: QueryHomeCustodyReadmissionDenial,
}
impl<Schema, Program> QueryHomeCustodyReadmissionRefused<Schema, Program> {
    pub fn into_parts(
        self,
    ) -> (
        ReadmittedHomeQueryService<Schema, Program>,
        QueryHomeCustodyReadmissionDenial,
    ) {
        (self.query, self.cause)
    }
}
impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub fn readmit_custody(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        obligations: std::sync::Arc<dyn crate::facade::program_support::QueryObligationsMedium>,
        material: std::sync::Arc<dyn crate::facade::correction_material::QueryCorrectionsMedium>,
    ) -> Result<CustodyReadmitted<Self>, QueryHomeCustodyReadmissionRefused<Schema, Program>> {
        let owner = self
            .runtime
            .runtime()
            .runtime
            .product_runtime
            .owner
            .owner_identity();
        if owner.durable().map(|owner| owner.home()) != Some(obligations.home()) {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::program_support::QueryProgramSupportDenial::ForeignHome,
                ),
            });
        }
        let Some(fence) = owner.home_incarnation_fence() else {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::program_support::QueryProgramSupportDenial::ForeignHome,
                ),
            });
        };
        if fence.admit(obligations.incarnation().fence()).is_err() {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::program_support::QueryProgramSupportDenial::StaleIncarnation,
                ),
            });
        }
        let support = self.runtime.runtime().installed_program_support();
        let Some(support) = support else {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::program_support::QueryProgramSupportDenial::SupportUnavailable,
                ),
            });
        };
        let originals = match self
            .runtime
            .runtime()
            .runtime
            .product_runtime
            .owner
            .inspection_port()
            .observe_program_activation_history(execution)
        {
            Ok(originals) => originals,
            Err(cause) => {
                return Err(QueryHomeCustodyReadmissionRefused {
                    query: self,
                    cause: QueryHomeCustodyReadmissionDenial::ProgramHistorySource(cause),
                })
            }
        };
        if material.home() != obligations.home()
            || material.incarnation() != obligations.incarnation()
        {
            let cause = if material.home() != obligations.home() {
                crate::facade::correction_material::QueryCorrectionMaterialDenial::ForeignHome
            } else {
                crate::facade::correction_material::QueryCorrectionMaterialDenial::StaleIncarnation
            };
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
            });
        }
        let material_admission = match self
            .runtime
            .runtime()
            .runtime
            .primary_provider
            .readmit_correction_material(material, &originals, execution)
        {
            Ok(admission) => admission,
            Err(cause) => {
                return Err(QueryHomeCustodyReadmissionRefused {
                    query: self,
                    cause: QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
                })
            }
        };
        self.correction_material_admission = std::sync::OnceLock::from(material_admission);
        let admission =
            match self.runtime.runtime().runtime.primary_provider
                .readmit_retained_program_support(support.lifecycle(), obligations, &originals, execution)
            {
                Ok(admission) => admission,
                Err(cause) => {
                    return Err(QueryHomeCustodyReadmissionRefused {
                        query: self,
                        cause: match cause {
                            crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial::Material(cause) => QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
                            crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial::ProgramSupport(cause) => QueryHomeCustodyReadmissionDenial::ProgramSupport(cause),
                            crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial::World(cause) => QueryHomeCustodyReadmissionDenial::ProgramHistorySource(cause),
                        },
                    })
                }
            };
        if let Err(cause) =
            support.bind_original_meaning_receiver(&self.runtime.runtime().runtime.primary_provider)
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(cause),
            });
        }
        if let Some(existing) = self.program_support_admission.get() {
            if admission.position() < existing.position()
                || (admission.position() == existing.position()
                    && (existing.digest() != admission.digest()
                        || existing.payload_digest() != admission.payload_digest()))
            {
                return Err(QueryHomeCustodyReadmissionRefused {
                    query: self,
                    cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                        crate::facade::program_support::QueryProgramSupportDenial::PrefixMismatch,
                    ),
                });
            }
        }
        // The lifecycle has folded this same issuer's prefix and acknowledged
        // every release before returning. A repeated opening may advance it
        // after a terminal original disposition, never roll it back.
        self.program_support_admission = std::sync::OnceLock::from(admission);
        if let Err(cause) =
            crate::domain_computation::primary_graph::bootstrap::branch_activation_history::verify(
                self.runtime.runtime(),
                execution,
            )
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::Activation(cause),
            });
        }
        if let Err(cause) = corrections::verify(self.runtime.runtime(), execution) {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::Correction(cause),
            });
        }
        if let Err(cause) = self.admit_original_custody(execution) {
            return Err(QueryHomeCustodyReadmissionRefused { query: self, cause });
        }
        Ok(CustodyReadmitted::from_owner(self))
    }
    fn admit_original_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryHomeCustodyReadmissionDenial> {
        use QueryHomeCustodyReadmissionDenial as Denial;
        let application = self.runtime.runtime();
        let owner = &application.runtime.product_runtime.owner;
        let snapshot = owner
            .inspection_port()
            .recovery_snapshot()
            .map_err(|cause| Denial::World(RuntimeWorldServiceDenial::OwnerUnavailable(cause)))?;
        if snapshot.reserved() != 0 || snapshot.abandoned() != 0 || snapshot.updating() != 0 {
            return Err(Denial::UnclassifiedWorldCustody);
        }
        let recovery = owner.recovery_port();
        let mut after = None;
        loop {
            execution.consult().map_err(Denial::Execution)?;
            let page = owner
                .inspection_port()
                .recovery_page(
                    after.as_ref(),
                    std::num::NonZeroUsize::new(32).expect("nonzero page"),
                )
                .map_err(Denial::World)?;
            for row in page.rows() {
                match row.state() {
                    RuntimeWorldRecoveryRecordState::Recovered => {}
                    RuntimeWorldRecoveryRecordState::Active
                    | RuntimeWorldRecoveryRecordState::Abandoned
                    | RuntimeWorldRecoveryRecordState::Retained
                    | RuntimeWorldRecoveryRecordState::Busy => {
                        return Err(Denial::UnclassifiedWorldCustody)
                    }
                }
                let effects = recovery
                    .inspect_recovered_effects(row.handle())
                    .map_err(Denial::Recovery)?;
                application
                    .runtime
                    .primary_provider
                    .admit_recovered_idempotency(execution, row.handle().clone(), &effects)
                    .map_err(original_binding_denial)?;
            }
            after = page.next_after().cloned();
            if after.is_none() {
                break;
            }
        }
        let mut after = None;
        loop {
            let page = recovery
                .released_original_page(
                    execution,
                    after.as_ref(),
                    std::num::NonZeroUsize::new(32).expect("nonzero page"),
                )
                .map_err(Denial::ReleasedSource)?;
            for original in page.rows() {
                application
                    .runtime
                    .primary_provider
                    .admit_released_idempotency(execution, original)
                    .map_err(original_binding_denial)?;
            }
            after = page.next_after().cloned();
            if after.is_none() {
                return Ok(());
            }
        }
    }
}

fn original_binding_denial(
    cause: crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial,
) -> QueryHomeCustodyReadmissionDenial {
    use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Source;
    use QueryHomeCustodyReadmissionDenial as Denial;
    match cause {
        Source::ForeignAffinity => Denial::ForeignOriginalAffinity,
        Source::RetentionCapacityExhausted | Source::RetentionIdentityExhausted => {
            Denial::OriginalMemoryAdmission
        }
        Source::PreparationBusy
        | Source::ActiveSnapshotCapacityExhausted { .. }
        | Source::SnapshotIdentityExhausted
        | Source::RecordedIntentUnverifiable
        | Source::ExecutionDenied(_)
        | Source::Unavailable => Denial::OriginalBindingUnavailable,
    }
}
