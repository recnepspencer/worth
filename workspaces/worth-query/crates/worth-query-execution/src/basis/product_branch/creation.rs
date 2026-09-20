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
    application_commit_lane: Option<std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitLane,
    >>,
    source_program_resolver: Option<&'runtime dyn WorthQuerySourceProgramResolver>,
}

pub(crate) trait WorthQuerySourceProgramResolver {
    fn source_program(
        &self,
        source: &crate::basis::WorthQueryProductBranchLease,
    ) -> Option<ApplicationProgramRevision>;
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
            application_commit_lane: None,
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
        application_commit_lane: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitLane,
        >,
        source_program_resolver: &'runtime dyn WorthQuerySourceProgramResolver,
    ) -> Self {
        self.output_lineage = Some(output_lineage);
        self.application_commit_lane = Some(application_commit_lane);
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

    pub fn create(self) -> Result<WorthQueryProductBranch, WorthQueryProductBranchCreateError> {
        let Self {
            runtime,
            source,
            intent,
            output_lineage,
            application_commit_lane,
            source_program_resolver,
        } = self;
        let _coordination = application_commit_lane.as_ref().map(|lane| lane.enter());
        let components = intent
            .ok_or(WorthQueryProductBranchCreateError::ComponentsIncomplete)?
            .components();
        let relational = component_plan(components.relational())?;
        let signal = component_plan(components.signal())?;
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
        let source = runtime
            .admit_product_occurrence(source.occurrence())
            .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?;
        let source_program =
            source_program_resolver.and_then(|resolver| resolver.source_program(&source));
        let cancellation = RuntimeWorldCancellationSource::new();
        match runtime
            .create_product_branch(
                &source,
                source_program.as_ref(),
                intent,
                &cancellation.token(),
            )
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
