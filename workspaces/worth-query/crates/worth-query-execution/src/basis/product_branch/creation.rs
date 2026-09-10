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
        }
    }
}

impl WorthQueryProductBranchFork<'_> {
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
        let components = self
            .intent
            .ok_or(WorthQueryProductBranchCreateError::ComponentsIncomplete)?
            .components();
        let relational = component_plan(components.relational())?;
        let signal = component_plan(components.signal())?;
        let ordinal = self
            .runtime
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
        let source = self
            .runtime
            .admit_product_occurrence(self.source.occurrence())
            .map_err(WorthQueryProductBranchCreateError::SourceAdmission)?;
        let cancellation = RuntimeWorldCancellationSource::new();
        match self
            .runtime
            .create_product_branch(&source, intent, &cancellation.token())
            .map_err(WorthQueryProductBranchCreateError::Creation)?
        {
            RuntimeWorldBranchCreationOutcome::Performed(observation) => Ok(
                WorthQueryProductBranch::from_occurrence(observation.lifecycle_incarnation()),
            ),
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                Err(WorthQueryProductBranchCreateError::ProductUnpublished(
                    super::WorthQueryProductBranchCreationRecovery::new(
                        effects,
                        self.runtime.owner.recovery_port(),
                        self.runtime.clone(),
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
