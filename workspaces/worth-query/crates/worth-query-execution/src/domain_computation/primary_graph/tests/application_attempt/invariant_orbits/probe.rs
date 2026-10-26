//! One installed orbit policy and a read-only, receiver-owned observer.
use super::schema::*;
use crate::facade::application_invariants::*;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};
use worth_query_declaration::{facade::application_schema::*, *};
pub(super) type Entity<const O: bool> = WorthQueryApplicationInvariantEntity<OrbitSchema<O>, Node>;
pub(super) struct Scope<const O: bool> {
    pub(super) proposed: Vec<(u64, Entity<O>)>,
    pub(super) committed: Vec<(u64, Entity<O>)>,
}
impl<const O: bool> Scope<O> {
    pub(super) fn node(&self, key: u64) -> &Entity<O> {
        &self
            .proposed
            .iter()
            .find(|(k, _)| *k == key)
            .expect("authored member")
            .1
    }
}
type Callback<const O: bool> = dyn for<'a, 'b> Fn(
        &WorthQueryApplicationInvariantContext<'a, 'b, OrbitSchema<O>>,
        &Scope<O>,
        &Rule<O>,
    ) + Send
    + Sync
    + std::panic::RefUnwindSafe
    + std::panic::UnwindSafe;
#[derive(Clone)]
pub(super) struct Probe<const O: bool = true> {
    armed: Arc<AtomicBool>,
    callback: Arc<Callback<O>>,
}
impl<const O: bool> Probe<O> {
    pub(super) fn new(
        callback: impl for<'a, 'b> Fn(
                &WorthQueryApplicationInvariantContext<'a, 'b, OrbitSchema<O>>,
                &Scope<O>,
                &Rule<O>,
            ) + Send
            + Sync
            + std::panic::RefUnwindSafe
            + std::panic::UnwindSafe
            + 'static,
    ) -> Self {
        Self {
            armed: Arc::new(AtomicBool::new(false)),
            callback: Arc::new(callback),
        }
    }
    pub(super) fn arm(&self) {
        self.armed.store(true, Ordering::SeqCst);
    }
}
#[derive(Clone, Default)]
pub(super) struct Observation(Arc<[AtomicUsize; 3]>);
impl Observation {
    pub(super) fn arm(&self) {
        for count in self.0.iter() {
            count.store(0, Ordering::SeqCst);
        }
    }
    pub(super) fn contacts(&self) -> [usize; 3] {
        std::array::from_fn(|i| self.0[i].load(Ordering::SeqCst))
    }
}
#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub(super) struct Refusal {
    pub(super) root: u64,
    pub(super) blocked: u64,
    pub(super) ordinal: u64,
}
worth_query_value_binding!(pub(super) RefusalBinding for Refusal { identity: "tests.invariant-orbit-refusal.v1", scalar: String, canonical: ValueDecodeBounds::new(128, 128, 8, 8), });
pub(super) struct OrbitRule;
impl<const O: bool> ApplicationInvariantMarkerIdentity<OrbitSchema<O>> for OrbitRule {
    const IDENTIFIER: &'static str = "orbit-read-policy";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition<const O: bool>(
) -> ApplicationInvariantDefinition<OrbitSchema<O>, OrbitRule> {
    ApplicationInvariantDefinition::new(
        OrbitRule::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [
                ApplicationInvariantScopeTarget::Entity("Node".into()),
                ApplicationInvariantScopeTarget::Relation("Next".into()),
            ],
            [
                ApplicationInvariantScopeTarget::Entity("Node".into()),
                ApplicationInvariantScopeTarget::Relation("Next".into()),
            ],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
pub(super) struct Rule<const O: bool> {
    pub(super) key: WorthQueryApplicationInvariantFieldBinding<OrbitSchema<O>, Node, u64>,
    pub(super) next:
        WorthQueryApplicationInvariantRelationBinding<OrbitSchema<O>, Next, Node, Node>,
    observer: Probe<O>,
    observation: Observation,
    blocked: Vec<u64>,
}
pub(super) fn resolve<const O: bool>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, OrbitSchema<O>>,
    observer: Probe<O>,
    observation: Observation,
    blocked: Vec<u64>,
) -> Result<Rule<O>, String> {
    Ok(Rule {
        key: resolver
            .typed_field(NodeKey::reference())
            .ok_or("key missing")?,
        next: resolver
            .typed_relation(Next::reference())
            .ok_or("next missing")?,
        observer,
        observation,
        blocked,
    })
}
impl<const O: bool> Rule<O> {
    fn nodes(
        &self,
        view: &WorthQueryApplicationInvariantReadView<'_, OrbitSchema<O>>,
    ) -> Result<Vec<(u64, Entity<O>)>, WorthQueryApplicationInvariantPreparationError> {
        let mut nodes = BTreeMap::new();
        let touched = view
            .touched_entities(&self.key)
            .map_err(preparation_error)?;
        for root in &touched {
            if nodes.contains_key(&root.entity_id()) {
                continue;
            }
            let members = if O {
                view.orbit(&self.next, root)
                    .map_err(preparation_error)?
                    .members()
                    .collect::<Vec<_>>()
            } else {
                view.walk(&self.next, RelationDirection::Forward, root)
                    .map_err(preparation_error)?
                    .entities()
                    .collect()
            };
            for entity in members {
                let key = view
                    .field(&self.key, &entity)
                    .map_err(preparation_error)?
                    .ok_or_else(|| {
                        WorthQueryApplicationInvariantPreparationError::new("key absent")
                    })?;
                nodes.insert(entity.entity_id(), (key, entity));
            }
        }
        Ok(nodes.into_values().collect())
    }
}
impl<const O: bool> WorthQueryApplicationInvariantRule<OrbitSchema<O>> for Rule<O> {
    type Scope = Scope<O>;
    type Refusal = Refusal;
    type RefusalBinding = RefusalBinding;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, OrbitSchema<O>>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        if !budget.checkpoint(0) {
            return Some(Err(WorthQueryApplicationInvariantPreparationError::new(
                "orbit preparation stopped",
            )));
        }
        Some(self.prepare_scope(planner))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, OrbitSchema<O>>,
        scope: &Scope<O>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Refusal>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        if !budget.checkpoint(0) {
            return Some(Err(WorthQueryApplicationInvariantExecutionError::new(
                "orbit evaluation stopped",
            )));
        }
        Some(self.evaluate(context, scope))
    }

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, OrbitSchema<O>>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        let scope = Scope {
            proposed: self.nodes(&planner.proposed())?,
            committed: self.nodes(&planner.committed())?,
        };
        self.observation.0[0].fetch_add(1, Ordering::SeqCst);
        Ok(scope)
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, OrbitSchema<O>>,
        scope: &Scope<O>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        context
            .check_live()
            .map_err(|e| WorthQueryApplicationInvariantExecutionError::new(format!("{e:?}")))?;
        self.observation.0[1].fetch_add(1, Ordering::SeqCst);
        if self.observer.armed.load(Ordering::SeqCst) {
            (self.observer.callback)(context, scope, self);
        }
        if O {
            if let Some((_, root)) = scope.proposed.iter().find(|(key, _)| *key == 0) {
                let view = context.proposed();
                let orbit = view.orbit(&self.next, root).map_err(|e| {
                    WorthQueryApplicationInvariantExecutionError::new(e.to_string())
                })?;
                if orbit.len() > 3 {
                    self.observation.0[2].fetch_add(1, Ordering::SeqCst);
                }
                for (ordinal, entity) in orbit.members().enumerate() {
                    let key = view
                        .field(&self.key, &entity)
                        .map_err(|e| {
                            WorthQueryApplicationInvariantExecutionError::new(e.to_string())
                        })?
                        .unwrap();
                    if self.blocked.contains(&key) {
                        return Ok(WorthQueryApplicationInvariantVerdict::Violation(Refusal {
                            root: 0,
                            blocked: key,
                            ordinal: ordinal as u64,
                        }));
                    }
                }
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
fn preparation_error(
    e: WorthQueryInvariantAccessDenial,
) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(e.to_string())
}
