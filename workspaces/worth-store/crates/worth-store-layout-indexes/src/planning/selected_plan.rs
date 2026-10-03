use super::candidates::EligibleStrategyOperation;
use super::cost::derive_access_plan_cost;
use super::decision::{
    DegradedScanSelectionGrant, LsmCompactionSelectionGrant, LsmLookupSelectionGrant,
    LsmPublicationSelectionGrant, LsmReplaySelectionGrant,
};
use super::plan_identity::AccessPlanIdentityBasis;
use super::{
    AccessPlanCostEstimate, AccessPlanIdentity, DeterministicSelectionRule, SelectionCandidateAudit,
};
use crate::access::budget::PlannedCounterEnvelope;
use crate::access::AdmittedAccessIntent;
use crate::artifact_family::AdmittedPhysicalArtifactFamily;
use crate::catalog::ArtifactFamilyLifecycleAdmission;
use crate::keyspace::{
    AdmittedPhysicalAccessIdentity, AdmittedPhysicalKeyDomain, PhysicalKeyDomainWitness,
};
use crate::materialization::AdmittedLayoutMaterialization;
use crate::strategy::registry::LayoutStrategyRegistrySnapshot;
use crate::strategy::{AdmittedLayoutStrategy, LayoutStrategyFamily};
use worth_store_budgets::{
    pre_execution_budget_admission, PreExecutionBudgetAdmissionReceipt, PreExecutionBudgetEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlanSelectionBasis {
    pub(super) family: AdmittedPhysicalArtifactFamily,
    pub(super) key_domain: AdmittedPhysicalKeyDomain,
    pub(super) request_identity: AdmittedPhysicalAccessIdentity,
    pub(super) materialization: Option<AdmittedLayoutMaterialization>,
    pub(super) strategy_admission: Option<LayoutStrategyRegistrySnapshot>,
    pub(super) selected_family: LayoutStrategyFamily,
    pub(super) selected_operation: Option<EligibleStrategyOperation>,
    pub(super) intent: AdmittedAccessIntent,
    pub(super) planned_counter_envelope: PlannedCounterEnvelope,
    pub(super) selection_rule: DeterministicSelectionRule,
    pub(super) primary_candidate: SelectionCandidateAudit,
    pub(super) secondary_candidate: SelectionCandidateAudit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CostedAccessPlan {
    basis: PlanSelectionBasis,
    cost_estimate: AccessPlanCostEstimate,
}

impl CostedAccessPlan {
    fn derive(basis: PlanSelectionBasis) -> Result<Self, super::AccessPlanCostDenial> {
        let cost_estimate = derive_access_plan_cost(
            basis.selected_operation,
            basis.intent,
            basis.planned_counter_envelope,
            basis.materialization.clone(),
        )?;
        Ok(Self {
            basis,
            cost_estimate,
        })
    }

    const fn budget_request(&self) -> worth_store_budgets::PreExecutionBudgetRequest {
        self.cost_estimate
            .to_budget_request(AccessPlanCostEstimate::budget_scope_for(self.basis.intent))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SelectedAccessPlanBasis {
    identity: AccessPlanIdentity,
    selected_operation: Option<EligibleStrategyOperation>,
    budget_receipt: PreExecutionBudgetAdmissionReceipt,
}

impl SelectedAccessPlanBasis {
    fn from_budget_admission(
        costed: CostedAccessPlan,
        budget_receipt: PreExecutionBudgetAdmissionReceipt,
    ) -> Self {
        let CostedAccessPlan {
            basis,
            cost_estimate,
        } = costed;
        let identity = AccessPlanIdentity::new(AccessPlanIdentityBasis {
            admitted_family: basis.family,
            strategy_family: basis.selected_family,
            intent: basis.intent,
            key_domain: basis.key_domain,
            request_identity: basis.request_identity,
            materialization: basis.materialization,
            strategy_admission: basis.strategy_admission,
            planned_counter_envelope: basis.planned_counter_envelope,
            selection_rule: basis.selection_rule,
            primary_candidate: basis.primary_candidate,
            secondary_candidate: basis.secondary_candidate,
            cost_estimate,
            budget_request: budget_receipt.request(),
            budget_envelope: budget_receipt.admitted_envelope(),
        });
        Self {
            identity,
            selected_operation: basis.selected_operation,
            budget_receipt,
        }
    }

    pub fn lifecycle(&self) -> ArtifactFamilyLifecycleAdmission {
        self.identity.lifecycle()
    }
    pub fn admitted_family(&self) -> AdmittedPhysicalArtifactFamily {
        self.identity.admitted_family()
    }
    pub fn key_domain(&self) -> PhysicalKeyDomainWitness {
        self.identity.key_domain()
    }
    pub fn admitted_key_domain(&self) -> AdmittedPhysicalKeyDomain {
        self.identity.admitted_key_domain()
    }
    pub fn request_identity(&self) -> AdmittedPhysicalAccessIdentity {
        self.identity.request_identity()
    }

    pub fn materialization(&self) -> Option<&AdmittedLayoutMaterialization> {
        self.identity.materialization()
    }

    pub fn admitted_strategy(&self) -> Option<&AdmittedLayoutStrategy> {
        self.identity.admitted_strategy()
    }

    pub fn strategy_admission(&self) -> Option<&LayoutStrategyRegistrySnapshot> {
        self.identity.strategy_admission()
    }

    pub fn selected_family(&self) -> LayoutStrategyFamily {
        self.identity.family()
    }
    pub(super) const fn selected_operation(&self) -> Option<EligibleStrategyOperation> {
        self.selected_operation
    }
    pub fn intent(&self) -> AdmittedAccessIntent {
        self.identity.intent()
    }
    pub const fn fingerprint(&self) -> &AccessPlanIdentity {
        &self.identity
    }
    pub fn cost_estimate(&self) -> &AccessPlanCostEstimate {
        self.identity.cost_estimate()
    }
    pub fn planned_counter_envelope(&self) -> PlannedCounterEnvelope {
        self.identity.planned_counter_envelope()
    }
    pub const fn budget_receipt(&self) -> PreExecutionBudgetAdmissionReceipt {
        self.budget_receipt
    }
    pub fn selection_rule(&self) -> DeterministicSelectionRule {
        self.identity.selection_rule()
    }
    pub fn primary_candidate(&self) -> &SelectionCandidateAudit {
        self.identity.primary_candidate()
    }
    pub fn secondary_candidate(&self) -> &SelectionCandidateAudit {
        self.identity.secondary_candidate()
    }
}

pub(super) fn admit_selected_plan_budget(
    basis: PlanSelectionBasis,
    envelope: PreExecutionBudgetEnvelope,
) -> Result<SelectedAccessPlanBasis, super::AccessPlanSelectionDenied> {
    let costed =
        CostedAccessPlan::derive(basis).map_err(super::AccessPlanSelectionDenied::CostDenied)?;
    let request = costed.budget_request();
    let receipt = pre_execution_budget_admission()
        .admit(request, envelope)
        .into_result()
        .map_err(super::AccessPlanSelectionDenied::BudgetDenied)?;
    debug_assert_eq!(receipt.request(), request);
    Ok(SelectedAccessPlanBasis::from_budget_admission(
        costed, receipt,
    ))
}

macro_rules! impl_selected_operation_common {
    ($name:ident) => {
        impl $name {
            pub fn lifecycle(&self) -> ArtifactFamilyLifecycleAdmission {
                self.basis.lifecycle()
            }
            pub fn admitted_family(&self) -> AdmittedPhysicalArtifactFamily {
                self.basis.admitted_family()
            }
            pub fn key_domain(&self) -> PhysicalKeyDomainWitness {
                self.basis.key_domain()
            }
            pub fn admitted_key_domain(&self) -> AdmittedPhysicalKeyDomain {
                self.basis.admitted_key_domain()
            }
            pub fn request_identity(&self) -> AdmittedPhysicalAccessIdentity {
                self.basis.request_identity()
            }
            pub fn selected_family(&self) -> LayoutStrategyFamily {
                self.basis.selected_family()
            }
            pub fn intent(&self) -> AdmittedAccessIntent {
                self.basis.intent()
            }
            pub fn fingerprint(&self) -> &AccessPlanIdentity {
                self.basis.fingerprint()
            }
            pub fn cost_estimate(&self) -> &AccessPlanCostEstimate {
                self.basis.cost_estimate()
            }
            pub fn planned_counter_envelope(&self) -> PlannedCounterEnvelope {
                self.basis.planned_counter_envelope()
            }
            pub const fn budget_receipt(&self) -> PreExecutionBudgetAdmissionReceipt {
                self.basis.budget_receipt()
            }
            pub fn selection_rule(&self) -> DeterministicSelectionRule {
                self.basis.selection_rule()
            }
            pub fn primary_candidate(&self) -> &SelectionCandidateAudit {
                self.basis.primary_candidate()
            }
            pub fn secondary_candidate(&self) -> &SelectionCandidateAudit {
                self.basis.secondary_candidate()
            }
        }
    };
}

macro_rules! define_materialized_degraded_operation {
    ($name:ident, $grant:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            basis: SelectedAccessPlanBasis,
            materialization: AdmittedLayoutMaterialization,
        }

        impl $name {
            pub(super) fn from_decision(basis: SelectedAccessPlanBasis, _grant: $grant) -> Self {
                let materialization = required_materialization(&basis);
                Self {
                    basis,
                    materialization,
                }
            }

            pub const fn materialization(&self) -> &AdmittedLayoutMaterialization {
                &self.materialization
            }
            pub fn admitted_strategy(&self) -> Option<&AdmittedLayoutStrategy> {
                self.basis.admitted_strategy()
            }
            pub fn strategy_admission(&self) -> Option<&LayoutStrategyRegistrySnapshot> {
                self.basis.strategy_admission()
            }
        }

        impl_selected_operation_common!($name);
    };
}

macro_rules! define_strategy_operation {
    ($name:ident, $grant:ty, $materialization:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            basis: SelectedAccessPlanBasis,
            strategy_admission: LayoutStrategyRegistrySnapshot,
            $materialization: define_strategy_operation!(@field $materialization),
        }

        impl $name {
            pub(super) fn from_decision(basis: SelectedAccessPlanBasis, _grant: $grant) -> Self {
                let strategy_admission = required_strategy_admission(&basis);
                let $materialization = define_strategy_operation!(@value $materialization, &basis);
                Self { basis, strategy_admission, $materialization }
            }

            pub fn admitted_strategy(&self) -> &AdmittedLayoutStrategy {
                self.strategy_admission.admitted_strategy()
            }
            pub const fn strategy_admission(&self) -> &LayoutStrategyRegistrySnapshot {
                &self.strategy_admission
            }
            define_strategy_operation!(@accessor $materialization);
        }

        impl_selected_operation_common!($name);
    };
    (@field materialization) => { AdmittedLayoutMaterialization };
    (@field no_materialization) => { () };
    (@value materialization, $basis:expr) => { required_materialization($basis) };
    (@value no_materialization, $basis:expr) => { () };
    (@accessor materialization) => {
        pub const fn materialization(&self) -> &AdmittedLayoutMaterialization {
            &self.materialization
        }
    };
    (@accessor no_materialization) => {};
}

fn required_strategy_admission(basis: &SelectedAccessPlanBasis) -> LayoutStrategyRegistrySnapshot {
    basis
        .strategy_admission()
        .cloned()
        .expect("strategy-selected operation must retain registry admission")
}

fn required_materialization(basis: &SelectedAccessPlanBasis) -> AdmittedLayoutMaterialization {
    basis
        .materialization()
        .cloned()
        .expect("materialized operation is issued only from an admitted read or recovery request")
}

define_materialized_degraded_operation!(SelectedDegradedExactScan, DegradedScanSelectionGrant);
define_strategy_operation!(SelectedLsmLookup, LsmLookupSelectionGrant, materialization);
define_strategy_operation!(
    SelectedLsmRunPublication,
    LsmPublicationSelectionGrant,
    no_materialization
);
define_strategy_operation!(
    SelectedLsmReplayRecovery,
    LsmReplaySelectionGrant,
    materialization
);
define_strategy_operation!(
    SelectedLsmCompaction,
    LsmCompactionSelectionGrant,
    no_materialization
);
