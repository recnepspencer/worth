//! Caller cancellation during real invariant preparation must not fail a shared row.

use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarQuery;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarReadBinding;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarReadResult;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::principal;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::principal::ConsumerPrincipalBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOutputDemandSource;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;

use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::Application;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::OwnerSchema;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::ANCHOR;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantContext;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantRule;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantVerdict;
use worth_relational::facade::runtime::CustomInvariantExecutionError as WorthQueryApplicationInvariantExecutionError;
use worth_relational::facade::runtime::CustomInvariantLeaseBudget;
use worth_relational::facade::runtime::CustomInvariantPreparationError as WorthQueryApplicationInvariantPreparationError;
pub fn source(
    runtime: &Application,
    phase: &WorthQueryAdvancementPhase<'_>,
    external: &WorthQueryAuthenticatedExternalPrincipal<OwnerSchema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryApplicationOutputDemandSource<PlanarQuery, PlanarReadResult> {
    let selected = runtime
        .on_branch(runtime.current_world())
        .select(phase)
        .unwrap();
    let binding = runtime
        .installed_schema()
        .principal_binding(ConsumerPrincipalBinding::reference())
        .unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            phase,
            &binding,
            external,
            scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let entity = selected
        .resolve_entity(
            phase,
            BodyKey::reference(),
            ANCHOR.to_owned(),
            scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &entity);
    let binding = runtime
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<OwnerSchema>>()
        .unwrap();
    let plan = selected
        .admit_application_query(
            phase,
            binding.query(),
            &access,
            ApplicationQueryParameterSet::new(),
            WorthQueryProductQueryControls::new(binding.limits().maximum_results(), scope),
        )
        .unwrap();
    runtime
        .execute_application_query_one_shot(phase, plan)
        .unwrap()
        .into_admitted_disclosed()
        .into_output_demand_source()
}
pub type Interruption = Arc<Mutex<Option<WorthQueryCancellationSource>>>;
pub struct InterruptingRule {
    pub rule: crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_invariant::PositiveTurnRule<OwnerSchema>,
    pub interruption: Interruption,
}
impl WorthQueryApplicationInvariantRule<OwnerSchema> for InterruptingRule {
    type Refusal =
        <crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_invariant::PositiveTurnRule<OwnerSchema> as WorthQueryApplicationInvariantRule<
            OwnerSchema,
        >>::Refusal;
    type RefusalBinding =
        <crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_invariant::PositiveTurnRule<OwnerSchema> as WorthQueryApplicationInvariantRule<
            OwnerSchema,
        >>::RefusalBinding;
    type Scope =
        <crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_invariant::PositiveTurnRule<OwnerSchema> as WorthQueryApplicationInvariantRule<
            OwnerSchema,
        >>::Scope;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, OwnerSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        let scope = self.rule.prepare_scope_checked(planner, budget);
        // Cancel the real admitted request, after the real rule prepared its scope.
        // The next production safe point decides the native interruption.
        if let Some(source) = self.interruption.lock().unwrap().take() {
            source.cancel();
        }
        scope
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, OwnerSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.rule.prepare_scope(planner)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, OwnerSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Self::Refusal>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        self.rule.evaluate_checked(context, scope, budget)
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, OwnerSchema>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Self::Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.rule.evaluate(context, scope)
    }
}
