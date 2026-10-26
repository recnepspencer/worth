use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::branch::{
    WorthQueryProductBranchComponentPosture, WorthQueryProductBranchComponents,
    WorthQueryProductBranchForkIntent,
};
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource, SignalBranchCreationPlan,
};

use super::{WorthQueryProductBranch, WorthQueryProductBranches};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCreationDenial, WorthQueryProductRuntime,
};

/// In-progress creation from one explicit product occurrence.
#[must_use = "a product branch fork is configured and created or dropped"]
pub struct WorthQueryProductBranchFork<'runtime> {
    runtime: &'runtime WorthQueryProductRuntime,
    source: WorthQueryProductBranch,
    intent: Option<WorthQueryProductBranchForkIntent>,
    output_lineage: Option<std::sync::Arc<std::sync::Mutex<
        crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage,
    >>>,
    application_provider: Option<&'runtime crate::domain_computation::primary_graph::WorthQueryPrimaryGraphProvider>,
    source_program_resolver: Option<&'runtime dyn WorthQuerySourceProgramResolver>,
}

pub(crate) trait WorthQuerySourceProgramResolver {
    fn source_program(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Option<ApplicationProgramRevision>, super::WorthQueryProductBranchAdmissionDenial>;
}

#[derive(Debug)]
pub enum WorthQueryProductBranchCreateError {
    ComponentsIncomplete,
    IdentityExhausted,
    SourceAdmission(super::WorthQueryProductBranchAdmissionDenial),
    Creation(WorthQueryProductBranchCreationDenial),
    ProductUnpublished(super::WorthQueryProductBranchCreationRecovery),
}

impl<'runtime> WorthQueryProductBranches<'runtime> {
    pub fn fork(self, source: WorthQueryProductBranch) -> WorthQueryProductBranchFork<'runtime> {
        WorthQueryProductBranchFork {
            runtime: self.runtime,
            source,
            intent: None,
            output_lineage: None,
            application_provider: None,
            source_program_resolver: None,
        }
    }
}

impl<'runtime> WorthQueryProductBranchFork<'runtime> {
    pub(crate) fn with_application_lifecycle(
        mut self,
        output_lineage: std::sync::Arc<std::sync::Mutex<
            crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage,
        >>,
        application_provider: &'runtime crate::domain_computation::primary_graph::WorthQueryPrimaryGraphProvider,
        source_program_resolver: &'runtime dyn WorthQuerySourceProgramResolver,
    ) -> Self {
        self.output_lineage = Some(output_lineage);
        self.application_provider = Some(application_provider);
        self.source_program_resolver = Some(source_program_resolver);
        self
    }

    pub fn components(
        mut self,
        choose: impl FnOnce(WorthQueryProductBranchComponents) -> WorthQueryProductBranchComponents,
    ) -> Self {
        self.intent = Some(WorthQueryProductBranchForkIntent::new(choose(
            WorthQueryProductBranchComponents::new(),
        )));
        self
    }

    /// Source and activation refusals precede destination-name issuance. Once
    /// submitted to World, the intent consumes its public ordinal even when
    /// World denies it or retains unpublished effects; ordinals are never reused.
    pub fn create(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryProductBranch, WorthQueryProductBranchCreateError> {
        let phase_request = phase.execution_request_for(self.runtime).map_err(|_| {
            WorthQueryProductBranchCreateError::SourceAdmission(
                super::WorthQueryProductBranchAdmissionDenial::Refused(
                    AdmissionRefusal::ForeignPhase,
                ),
            )
        })?;
        let execution = &phase_request;
        let Self {
            runtime,
            source,
            intent,
            output_lineage,
            application_provider,
            source_program_resolver,
        } = self;
        let application_commit_lane = application_provider.map(|provider| {
            provider.application_branch_commit_lane_for_occurrence(source.occurrence())
        });
        let _coordination = application_commit_lane.as_ref().map(|lane| lane.enter());
        let components = intent
            .ok_or(WorthQueryProductBranchCreateError::ComponentsIncomplete)?
            .components();
        let relational = component_plan(components.relational())?;
        let signal = component_plan(components.signal())?;
        let source = runtime
            .admit_product_occurrence(source.occurrence())
            .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?;
        let source_program = source_program_resolver
             .map(|resolver| {
                let _read = runtime.enter_named_read(crate::domain_computation::execution_runtime::product_world::read_consult::NamedRead::SourceProgramResolution);
                resolver.source_program(execution, &source)
            })
            .transpose()
            .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?
            .flatten();
        let reservation = runtime
            .reserve_product_branch_creation(&source, source_program.as_ref())
            .map_err(WorthQueryProductBranchCreateError::Creation)?;
        let ordinal = runtime
            .reserve_public_branch_ordinal()
            .ok_or(WorthQueryProductBranchCreateError::IdentityExhausted)?;
        let product_name = format!("query-product-{ordinal}");
        let relational = match relational {
            WorthQueryProductBranchComponentPosture::ReuseExact => {
                RelationalBranchCreationPlan::ReuseExact
            }
            WorthQueryProductBranchComponentPosture::Fork => {
                RelationalBranchCreationPlan::ForkExact {
                    target: worth_relational::facade::history::BranchId(format!(
                        "{product_name}-relational"
                    )),
                }
            }
        };
        let signal = match signal {
            WorthQueryProductBranchComponentPosture::ReuseExact => {
                SignalBranchCreationPlan::ReuseExact
            }
            WorthQueryProductBranchComponentPosture::Fork => SignalBranchCreationPlan::ForkExact {
                target: worth_signal::facade::branch::validate_signal_branch_name(format!(
                    "{product_name}-signal"
                ))
                .expect("generated Query product branch names are valid Signal names"),
            },
        };
        let intent = ProductBranchCreationIntent::from_source(
            product_name,
            ProductBranchCreationPlans::new(relational, signal),
        )
        .expect("generated Query product branch names satisfy World limits");
        let cancellation = RuntimeWorldCancellationSource::new();
        match reservation
            .create(intent, &cancellation.token())
            .map_err(WorthQueryProductBranchCreateError::Creation)?
        {
            RuntimeWorldBranchCreationOutcome::Performed(observation) => {
                if let Some(output_lineage) = output_lineage {
                    output_lineage
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .register_fork(source.observation(), &observation);
                }
                Ok(WorthQueryProductBranch::from_occurrence(
                    observation.lifecycle_incarnation(),
                ))
            }
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                Err(WorthQueryProductBranchCreateError::ProductUnpublished(
                    super::WorthQueryProductBranchCreationRecovery::new(
                        effects,
                        runtime.owner.recovery_port(),
                        runtime.clone(),
                    ),
                ))
            }
        }
    }
}

fn component_plan(
    posture: Option<WorthQueryProductBranchComponentPosture>,
) -> Result<WorthQueryProductBranchComponentPosture, WorthQueryProductBranchCreateError> {
    posture.ok_or(WorthQueryProductBranchCreateError::ComponentsIncomplete)
}
