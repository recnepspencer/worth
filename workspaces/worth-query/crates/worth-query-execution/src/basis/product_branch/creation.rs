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
    expected: Option<std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>>,
    intent: Option<WorthQueryProductBranchForkIntent>,
    output_lineage: Option<std::sync::Arc<std::sync::Mutex<
        crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage,
    >>>,
    application_provider: Option<&'runtime crate::domain_computation::primary_graph::WorthQueryPrimaryGraphProvider>,
    source_program_resolver: Option<&'runtime dyn WorthQuerySourceProgramResolver>,
}

pub(crate) trait WorthQuerySourceProgramResolver {
    fn retained_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    ) -> Result<super::WorthQueryProductBranchLease, super::WorthQueryProductBranchAdmissionDenial>;

    fn source_program(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Option<ApplicationProgramRevision>, super::WorthQueryProductBranchAdmissionDenial>;
}

#[derive(Debug)]
pub enum WorthQueryProductBranchCreateError {
    /// The expected source's application lane is currently owned elsewhere.
    CoordinationBusy,
    /// The original request refused the expected-source operation.
    Request(worth_execution::WorkCeilingDenial),
    /// Actual occurrence-lane storage admission refused before owner effects.
    CoordinationStorage(worth_execution::ExecutionOrderedMapDenial),
    /// The new lane's physical backing cannot be represented.
    CoordinationSizeOverflow,
    /// The new lane's lookup work cannot be represented.
    CoordinationWorkCounterOverflow,
    ComponentsIncomplete,
    IdentityExhausted,
    SourceAdmission(super::WorthQueryProductBranchAdmissionDenial),
    Creation(WorthQueryProductBranchCreationDenial),
    BranchCoordinationCapacityExhausted,
    ProductUnpublished(super::WorthQueryProductBranchCreationRecovery),
}

impl<'runtime> WorthQueryProductBranches<'runtime> {
    pub fn fork(self, source: WorthQueryProductBranch) -> WorthQueryProductBranchFork<'runtime> {
        WorthQueryProductBranchFork {
            runtime: self.runtime,
            source,
            expected: None,
            intent: None,
            output_lineage: None,
            application_provider: None,
            source_program_resolver: None,
        }
    }
}

impl<'runtime> WorthQueryProductBranchFork<'runtime> {
    pub(crate) fn with_expected_read(
        mut self,
        expected: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Self {
        self.expected = Some(expected);
        self
    }

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
            expected,
            intent,
            output_lineage,
            application_provider,
            source_program_resolver,
        } = self;
        let retained_source = expected
            .as_ref()
            .map(|expected| {
                source_program_resolver
                    .ok_or(WorthQueryProductBranchCreateError::SourceAdmission(
                        super::WorthQueryProductBranchAdmissionDenial::Refused(
                            AdmissionRefusal::ForeignOwner,
                        ),
                    ))?
                    .retained_source(execution, expected)
                    .map_err(WorthQueryProductBranchCreateError::SourceAdmission)
            })
            .transpose()?;
        if expected.is_some() {
            execution
                .consult()
                .map_err(WorthQueryProductBranchCreateError::Request)?;
        }
        let application_commit_lane = application_provider
            .map(|provider| {
                if expected.is_some() {
                    provider
                        .application_branch_commit_lane_for_request(source.occurrence(), *execution)
                        .map_err(coordination_denial)
                } else {
                    Ok(provider.application_branch_commit_lane_for_occurrence(source.occurrence()))
                }
            })
            .transpose()?;
        let _coordination = match application_commit_lane.as_ref() {
            Some(lane) if expected.is_some() => Some(
                lane.try_enter()
                    .ok_or(WorthQueryProductBranchCreateError::CoordinationBusy)?,
            ),
            Some(lane) => Some(lane.enter()),
            None => None,
        };
        if expected.is_some() {
            execution
                .consult()
                .map_err(WorthQueryProductBranchCreateError::Request)?;
        }
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
        let source = match retained_source {
            Some(source) => source,
            None => runtime
                .admit_product_occurrence(source.occurrence())
                .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?,
        };
        let source_program = source_program_resolver
             .map(|resolver| {
                let _read = runtime.enter_named_read(crate::domain_computation::execution_runtime::product_world::read_consult::NamedRead::SourceProgramResolution);
                resolver.source_program(execution, &source)
            })
            .transpose()
            .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?
            .flatten();
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

fn coordination_denial(
    cause: crate::domain_computation::primary_graph::BranchCoordinationDenial,
) -> WorthQueryProductBranchCreateError {
    use crate::domain_computation::primary_graph::BranchCoordinationDenial as Cause;
    match cause {
        Cause::Busy => WorthQueryProductBranchCreateError::CoordinationBusy,
        Cause::SizeOverflow => WorthQueryProductBranchCreateError::CoordinationSizeOverflow,
        Cause::WorkCounterOverflow => {
            WorthQueryProductBranchCreateError::CoordinationWorkCounterOverflow
        }
        Cause::Request(cause) => WorthQueryProductBranchCreateError::Request(cause),
        Cause::Storage(cause) => WorthQueryProductBranchCreateError::CoordinationStorage(cause),
    }
}
