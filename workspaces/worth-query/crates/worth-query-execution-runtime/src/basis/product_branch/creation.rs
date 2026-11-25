use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::branch::{
    WorthQueryProductBranchComponentPosture, WorthQueryProductBranchComponents,
    WorthQueryProductBranchForkIntent,
};
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldCancellationSource, SignalBranchCreationPlan,
};

use super::{WorthQueryProductBranch, WorthQueryProductBranches};
mod expected_source;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCreationDenial, WorthQueryProductBranchCreationOutcome,
    WorthQueryProductRuntime,
};
use expected_source::ForkSource;

/// In-progress creation from one explicit product occurrence.
#[must_use = "a product branch fork is configured and created or dropped"]
pub struct WorthQueryProductBranchFork<'runtime> {
    runtime: &'runtime WorthQueryProductRuntime,
    source: ForkSource,
    intent: Option<WorthQueryProductBranchForkIntent>,
    output_lineage: Option<std::sync::Arc<dyn crate::ports::ForkLineagePort>>,
    commit_coordination: Option<&'runtime dyn crate::ports::BranchCommitCoordinationPort>,
    source_program_resolver: Option<&'runtime dyn WorthQuerySourceProgramResolver>,
}

pub trait WorthQuerySourceProgramResolver {
    fn source_program(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Option<ApplicationProgramRevision>, super::WorthQueryProductBranchAdmissionDenial>;
}

#[derive(Debug)]
pub enum WorthQueryProductBranchCreateError {
    ComponentsIncomplete,
    NoEffect(
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationNoEffect,
    ),
    ReleasedOriginal(
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationRelease,
    ),
    SourceAdmission(super::WorthQueryProductBranchAdmissionDenial),
    Creation(WorthQueryProductBranchCreationDenial),
    ProductUnpublished(super::WorthQueryProductBranchCreationRecovery),
}

impl<'runtime> WorthQueryProductBranches<'runtime> {
    pub fn fork(self, source: WorthQueryProductBranch) -> WorthQueryProductBranchFork<'runtime> {
        WorthQueryProductBranchFork {
            runtime: self.runtime,
            source: ForkSource::Current(source),
            intent: None,
            output_lineage: None,
            commit_coordination: None,
            source_program_resolver: None,
        }
    }

    pub fn fork_expected(
        self,
        source: super::WorthQueryProductBranchLease,
    ) -> WorthQueryProductBranchFork<'runtime> {
        let mut fork = self.fork(source.product_branch());
        fork.source = ForkSource::Expected(source);
        fork
    }
}

impl<'runtime> WorthQueryProductBranchFork<'runtime> {
    pub fn with_application_lifecycle(
        mut self,
        output_lineage: std::sync::Arc<dyn crate::ports::ForkLineagePort>,
        commit_coordination: &'runtime dyn crate::ports::BranchCommitCoordinationPort,
        source_program_resolver: &'runtime dyn WorthQuerySourceProgramResolver,
    ) -> Self {
        self.output_lineage = Some(output_lineage);
        self.commit_coordination = Some(commit_coordination);
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

    pub fn create(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryProductBranch, WorthQueryProductBranchCreateError> {
        self.create_retaining_source(phase)
            .map(|(branch, _)| branch)
    }

    pub fn create_retaining_source(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        (WorthQueryProductBranch, super::WorthQueryProductBranchLease),
        WorthQueryProductBranchCreateError,
    > {
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
            commit_coordination,
            source_program_resolver,
        } = self;
        let occurrence = source.branch().occurrence();
        let run = || -> Result<
            (WorthQueryProductBranch, super::WorthQueryProductBranchLease),
            WorthQueryProductBranchCreateError,
        > {
            let components = intent
                .ok_or(WorthQueryProductBranchCreateError::ComponentsIncomplete)?
                .components();
            let relational = component_plan(components.relational())?;
            let signal = component_plan(components.signal())?;
            let ordinal = runtime
                .reserve_public_branch_ordinal()
                .map_err(WorthQueryProductBranchCreationDenial::World)
                .map_err(WorthQueryProductBranchCreateError::Creation)?;
            // ordinal is u64: this fixed ASCII prefix, at most 20 decimal digits,
            // and the fixed component suffixes fit both native name grammars/bounds.
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
            let expected = matches!(&source, ForkSource::Expected(_));
            let source = source
                .admit(runtime)
                .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?;
            let source_program = source_program_resolver
                .map(|resolver| {
                    let _read = runtime.enter_named_read(
                        crate::domain_computation::execution_runtime::product_world::read_consult::NamedRead::SourceProgramResolution,
                    );
                    resolver.source_program(execution, &source)
                })
                .transpose()
                .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?
                .flatten();
            let cancellation = RuntimeWorldCancellationSource::new();
            match runtime
                .create_product_branch(
                    execution,
                    &source,
                    source_program.as_ref(),
                    intent,
                    &cancellation.token(),
                    output_lineage,
                )
                .map_err(WorthQueryProductBranchCreateError::Creation)?
            {
                WorthQueryProductBranchCreationOutcome::NoEffect(actual) => {
                    Err(WorthQueryProductBranchCreateError::NoEffect(actual))
                }
                WorthQueryProductBranchCreationOutcome::ReleasedOriginal(actual) => {
                    Err(WorthQueryProductBranchCreateError::ReleasedOriginal(actual))
                }
                WorthQueryProductBranchCreationOutcome::Performed(observation) => Ok((
                    WorthQueryProductBranch::from_occurrence(observation.lifecycle_incarnation()),
                    source,
                )),
                WorthQueryProductBranchCreationOutcome::ProductUnpublished(recovery) => {
                    let recovery = if expected {
                        recovery.retain_expected_source(source)
                    } else {
                        recovery
                    };
                    Err(WorthQueryProductBranchCreateError::ProductUnpublished(
                        recovery,
                    ))
                }
            }
        };
        if let Some(coord) = commit_coordination {
            let mut run = Some(run);
            let mut outcome = None;
            coord.with_occurrence_coordination(occurrence, &mut || {
                outcome = Some((run.take().expect("run once"))());
            });
            outcome.expect("coordination executes body once")
        } else {
            run()
        }
    }
}

fn component_plan(
    posture: Option<WorthQueryProductBranchComponentPosture>,
) -> Result<WorthQueryProductBranchComponentPosture, WorthQueryProductBranchCreateError> {
    posture.ok_or(WorthQueryProductBranchCreateError::ComponentsIncomplete)
}
