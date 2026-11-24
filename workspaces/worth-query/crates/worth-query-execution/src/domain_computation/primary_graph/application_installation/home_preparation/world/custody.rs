//! Actual cold World custody and original Query keys admit the service phase.
use super::*;
use crate::domain_computation::primary_graph::provider::aftermath_causality::readmission as corrections;
use crate::domain_computation::primary_graph::query_obligations::history::journal_readmission::QueryObligationHistoryJournalReadmissionExt;
use worth_proof::CustodyReadmitted;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

#[derive(Debug)]
pub enum QueryHomeCustodyReadmissionDenial {
    RecoveryClaim(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial),
    CorrectionMaterial(crate::facade::correction_material::QueryCorrectionMaterialDenial),
    ProgramSupport(crate::facade::query_obligations::QueryObligationDenial),
    Activation(super::QueryHomeProgramActivationDenial),
    ProgramHistorySource(
        RuntimeWorldServiceDenial<worth_runtime_world::facade::WorldReadmissionDenial>,
    ),
    Correction(corrections::QueryHomeCorrectionLineageDenial),
    Dispatch(crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial),
    DispatchOutbox(crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial),
    DispatchCapability(crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial),
    Inbound(crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial),
    InboundCustody(crate::domain_computation::primary_graph::application_runtime::WorthQueryInboundAdmissionDenial),
    Execution(worth_execution::WorkCeilingDenial),
    World(RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>),
    Recovery(RuntimeWorldRecoveryDenial),
    UnclassifiedWorldCustody,
    /// A recovered original branch creation outlived attachment's decision.
    UndecidedBranchCreation,
    OriginalBindingUnavailable,
    ForeignOriginalAffinity,
    /// An indeterminate conditional row and World disagree on its original:
    /// another intent holds the key, or the row is joined to another original.
    ConditionalOriginalMismatch,
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
        obligations: std::sync::Arc<dyn crate::facade::query_obligations::QueryObligationsMedium>,
        material: std::sync::Arc<dyn crate::facade::correction_material::QueryCorrectionsMedium>,
        dispatch: std::sync::Arc<
            dyn crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchMedium,
        >,
    ) -> Result<CustodyReadmitted<Self>, QueryHomeCustodyReadmissionRefused<Schema, Program>> {
        let owner = self
            .runtime
            .runtime()
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .owner_identity();
        if owner.durable().map(|owner| owner.home()) != Some(obligations.home()) {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::query_obligations::QueryObligationDenial::ForeignHome,
                ),
            });
        }
        let Some(fence) = owner.home_incarnation_fence() else {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::query_obligations::QueryObligationDenial::ForeignHome,
                ),
            });
        };
        if fence.admit(obligations.incarnation().fence()).is_err() {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::query_obligations::QueryObligationDenial::StaleIncarnation,
                ),
            });
        }
        let support = self.runtime.runtime().installed_program_support();
        let Some(support) = support else {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::query_obligations::QueryObligationDenial::SupportUnavailable,
                ),
            });
        };
        let originals = match self
            .runtime
            .runtime()
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
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
            .inner
            .runtime
            .primary_provider_ref()
            .readmit_correction_material(std::sync::Arc::clone(&material), &originals, execution)
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
            match self.runtime.runtime().inner.runtime.primary_provider_ref()
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
        if let Err(cause) = crate::domain_computation::primary_graph::application_installation::checkpoint_lineage::readmit_journal(self.runtime.runtime(), &originals, execution) {
            return Err(QueryHomeCustodyReadmissionRefused { query: self, cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(cause) });
        }
        let serving = &self.runtime.runtime().inner.runtime;
        if let Err(cause) = support
            .obligation_history()
            .bind_live_receivers(serving.primary_provider_ref(), serving.execution())
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(cause),
            });
        }
        if let Some(existing) = self.program_support_admission.get()
            && (admission.position() < existing.position()
                || (admission.position() == existing.position()
                    && (existing.digest() != admission.digest()
                        || existing.payload_digest() != admission.payload_digest())))
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(
                    crate::facade::query_obligations::QueryObligationDenial::PrefixMismatch,
                ),
            });
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
        let unpublished = match self.unpublished_native_commits(execution) {
            Ok(commits) => commits,
            Err(cause) => return Err(QueryHomeCustodyReadmissionRefused { query: self, cause }),
        };
        if let Err(cause) = corrections::verify_with_unpublished(
            self.runtime.runtime().inner(),
            execution,
            &unpublished.continuable,
            &unpublished.all,
        ) {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::Correction(cause),
            });
        }
        if let Err(cause) =
            self.admit_dispatch_contracts(execution, &unpublished.continuable, &unpublished.all)
        {
            return Err(QueryHomeCustodyReadmissionRefused { query: self, cause });
        }
        if let Err(cause) = self.admit_dispatch_custody(execution, std::sync::Arc::clone(&dispatch))
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::Dispatch(cause),
            });
        }
        if let Err(cause) = self
            .runtime
            .runtime()
            .readmit_dispatch_completion_custody(execution)
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::InboundCustody(cause),
            });
        }
        if let Err(cause) = self.admit_original_custody(execution) {
            return Err(QueryHomeCustodyReadmissionRefused { query: self, cause });
        }
        if let Err(cause) = self.settle_indeterminate_decisions(execution) {
            return Err(QueryHomeCustodyReadmissionRefused { query: self, cause });
        }
        let originals = match self
            .runtime
            .runtime()
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
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
        if let Err(cause) = self
            .runtime
            .runtime()
            .admit_recovery_claim_sources(&originals, execution)
            .and_then(|()| self.runtime.runtime().settle_recovery_custody(execution))
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: QueryHomeCustodyReadmissionDenial::RecoveryClaim(cause),
            });
        }
        match self
            .runtime
            .runtime()
            .inner
            .runtime
            .primary_provider_ref()
            .readmit_correction_material(material, &originals, execution)
        {
            Ok(admission) => {
                self.correction_material_admission = std::sync::OnceLock::from(admission)
            }
            Err(cause) => {
                return Err(QueryHomeCustodyReadmissionRefused {
                    query: self,
                    cause: QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
                })
            }
        }
        // Cold terminal ACK settlement may advance the journal after its first
        // fold. Admit the final exact prefix before the service witness escapes.
        let application = self.runtime.runtime();
        match application.inner.runtime.dispatch_journal_ref().readmit(
            dispatch,
            execution,
            application
                .inner
                .runtime
                .primary_provider_ref()
                .dispatch_readmission_catalog(),
        ) {
            Ok(admission) => self.dispatch_custody_admission = std::sync::OnceLock::from(admission),
            Err(cause) => {
                return Err(QueryHomeCustodyReadmissionRefused {
                    query: self,
                    cause: QueryHomeCustodyReadmissionDenial::Dispatch(cause),
                })
            }
        }
        // Empty-original release and cold custody settlement may acknowledge
        // Query discharges after the first fold, just as dispatch does above.
        if let Err(cause) = application
            .inner
            .runtime
            .primary_provider_ref()
            .release_resolved_program_support_interpretations(
                application
                    .installed_program_support()
                    .expect("readmitted support")
                    .lifecycle(),
                &originals,
                execution,
            )
        {
            return Err(QueryHomeCustodyReadmissionRefused {
                query: self,
                cause: match cause {
                    crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial::Material(cause) => QueryHomeCustodyReadmissionDenial::CorrectionMaterial(cause),
                    crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial::ProgramSupport(cause) => QueryHomeCustodyReadmissionDenial::ProgramSupport(cause),
                    crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial::World(cause) => QueryHomeCustodyReadmissionDenial::ProgramHistorySource(cause),
                },
            });
        }
        let admission = match application
            .installed_program_support()
            .expect("readmitted support")
            .lifecycle()
            .admit_current_prefix(execution)
        {
            Ok(admission) => admission,
            Err(cause) => {
                return Err(QueryHomeCustodyReadmissionRefused {
                    query: self,
                    cause: QueryHomeCustodyReadmissionDenial::ProgramSupport(cause),
                })
            }
        };
        self.program_support_admission = std::sync::OnceLock::from(admission);
        Ok(CustodyReadmitted::from_owner(self))
    }
}

mod conditional_indeterminate;
mod originals;
