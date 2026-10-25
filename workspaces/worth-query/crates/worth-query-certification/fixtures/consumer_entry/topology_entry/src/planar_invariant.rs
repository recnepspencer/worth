use super::runtime_api::application_invariants::*;
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[path = "planar_invariant/evaluation.rs"]
mod evaluation;
#[path = "planar_invariant/preparation.rs"]
mod preparation;
#[path = "planar_invariant/read_observation.rs"]
mod read_observation;
pub use read_observation::{PlanarInvariantReadEvent, PlanarInvariantReadObserver};

pub const INVARIANT_PROBE_STANDARD: usize = 0;
pub const INVARIANT_PROBE_UNDECLARED_ACCESS: usize = 1;
pub const INVARIANT_PROBE_EVALUATION_EXPANSION: usize = 2;

pub(crate) fn resolve_rule<Schema: TopologyModelBinding>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
    probe: Arc<AtomicUsize>,
    mode: Arc<AtomicUsize>,
    observer: Option<PlanarInvariantReadObserver>,
) -> Result<PositiveTurnRule<Schema>, String> {
    let declared_successor = PlanarSuccessor::reference::<Schema>();
    let forged_endpoints = worth_query_decl::facade::application_schema::ApplicationRelationRef::<
        Schema,
        PlanarSuccessor,
        Body,
        Body,
    >::from_schema_identifiers(
        declared_successor.name(),
        "NotBody",
        declared_successor.to(),
        declared_successor.integrity(),
    );
    if resolver.typed_relation(forged_endpoints).is_some() {
        return Err("relation binding accepted forged endpoint names".to_owned());
    }
    let x = resolver
        .typed_field(PositionX::reference::<Schema>())
        .ok_or("missing position x")?;
    let y = resolver
        .typed_field(PositionY::reference::<Schema>())
        .ok_or("missing position y")?;
    let successor = resolver
        .typed_relation(declared_successor)
        .ok_or("missing successor")?;
    let undeclared = resolver
        .typed_relation(UndeclaredPlanarRelation::reference::<Schema>())
        .ok_or("missing undeclared-access probe relation")?;
    Ok(PositiveTurnRule {
        successor,
        undeclared,
        x,
        y,
        probe,
        mode,
        observer,
    })
}

pub struct PositiveTurnRule<Schema: TopologyModelBinding> {
    successor: WorthQueryApplicationInvariantRelationBinding<Schema, PlanarSuccessor, Body, Body>,
    undeclared:
        WorthQueryApplicationInvariantRelationBinding<Schema, UndeclaredPlanarRelation, Body, Body>,
    x: WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    y: WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    probe: Arc<AtomicUsize>,
    mode: Arc<AtomicUsize>,
    observer: Option<PlanarInvariantReadObserver>,
}

impl<Schema: TopologyModelBinding> WorthQueryApplicationInvariantRule<Schema>
    for PositiveTurnRule<Schema>
{
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    type Scope = Vec<[WorthQueryApplicationInvariantEntity<Schema, Body>; 3]>;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare_with_budget(planner, None)
    }

    fn supports_checked_preparation(&self) -> bool {
        true
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare_with_budget(planner, Some(budget)))
    }

    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<()>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        Some(self.evaluate_with_budget(context, scope, Some(budget)))
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.evaluate_with_budget(context, scope, None)
    }
}

pub(crate) const PLANAR_CANDIDATE_WORK: u64 = 1;

impl<Schema: TopologyModelBinding> PositiveTurnRule<Schema> {
    fn observe(&self, event: PlanarInvariantReadEvent) {
        if let Some(observer) = &self.observer {
            observer(event);
        }
    }
}
