//! Diagnostic reads have their own installed rule and frozen admission frame.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::observation::Entity;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use worth_relational::facade::runtime::CustomInvariantLeaseBudget;
use worth_foundational::facade::RelationDirection;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantContext;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantEntityBinding;
use worth_relational::facade::runtime::CustomInvariantExecutionError as WorthQueryApplicationInvariantExecutionError;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantFieldBinding;
use worth_relational::facade::runtime::CustomInvariantPreparationError as WorthQueryApplicationInvariantPreparationError;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantReadView;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantRelationBinding;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantRule;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantSchemaResolver;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantVerdict;
use worth_query_execution::facade::application_invariants::WorthQueryEmptyInvariantRefusal;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantAccessDenial;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::*;

pub struct WalkProbe;
impl ApplicationInvariantMarkerIdentity<WalkSchema> for WalkProbe {
    const IDENTIFIER: &'static str = "graph-walk-probe";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn definition() -> ApplicationInvariantDefinition<WalkSchema, WalkProbe> {
    ApplicationInvariantDefinition::new(
        WalkProbe::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [
                ApplicationInvariantScopeTarget::Entity("Node".into()),
                ApplicationInvariantScopeTarget::Relation("GroupRoot".into()),
                ApplicationInvariantScopeTarget::Relation("Next".into()),
            ],
            [
                ApplicationInvariantScopeTarget::Entity("Node".into()),
                ApplicationInvariantScopeTarget::Relation("GroupRoot".into()),
                ApplicationInvariantScopeTarget::Relation("Next".into()),
            ],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

pub struct Scope {
    pub proposed: Vec<(u64, Entity)>,
    pub committed: Vec<(u64, Entity)>,
}
impl Scope {
    pub fn node(&self, key: u64) -> &Entity {
        &self
            .proposed
            .iter()
            .find(|(k, _)| *k == key)
            .expect("authored node")
            .1
    }
}

pub type Callback = dyn for<'a, 'b> Fn(&WorthQueryApplicationInvariantContext<'a, 'b, WalkSchema>, &Scope, &Rule)
    + Send
    + Sync
    + std::panic::RefUnwindSafe
    + std::panic::UnwindSafe;

#[derive(Clone)]
pub struct Probe {
    pub enabled: bool,
    pub armed: Arc<AtomicBool>,
    pub callback: Arc<Callback>,
}
impl Probe {
    pub fn new(
        callback: impl for<'a, 'b> Fn(
                &WorthQueryApplicationInvariantContext<'a, 'b, WalkSchema>,
                &Scope,
                &Rule,
            ) + Send
            + Sync
            + std::panic::RefUnwindSafe
            + std::panic::UnwindSafe
            + 'static,
    ) -> Self {
        Self {
            enabled: true,
            armed: Arc::new(AtomicBool::new(false)),
            callback: Arc::new(callback),
        }
    }
    pub fn arm(&self) {
        self.armed.store(true, Ordering::SeqCst);
    }

    pub fn disabled() -> Self {
        let mut probe = Self::new(|_, _, _| {});
        probe.enabled = false;
        probe
    }
}

pub struct Rule {
    pub probe: Probe,
    pub node: WorthQueryApplicationInvariantEntityBinding<WalkSchema, Node>,
    pub key: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, u64>,
    pub membership:
        WorthQueryApplicationInvariantRelationBinding<WalkSchema, GroupRoot, Node, Node>,
    pub next: WorthQueryApplicationInvariantRelationBinding<WalkSchema, Next, Node, Node>,
}
pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, WalkSchema>,
    probe: Probe,
) -> Result<Rule, String> {
    Ok(Rule {
        probe,
        node: resolver
            .typed_entity(Node::reference())
            .ok_or("node kind missing")?,
        key: resolver
            .typed_field(NodeKey::reference())
            .ok_or("node key missing")?,
        membership: resolver
            .typed_relation(GroupRoot::reference())
            .ok_or("membership missing")?,
        next: resolver
            .typed_relation(Next::reference())
            .ok_or("path missing")?,
    })
}

impl WorthQueryApplicationInvariantRule<WalkSchema> for Rule {
    type Scope = Scope;
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
    ) -> Result<Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare(planner, None)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare(planner, Some(budget)))
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        context
            .check_live()
            .map_err(|e| WorthQueryApplicationInvariantExecutionError::new(format!("{e:?}")))?;
        if self.probe.enabled && self.probe.armed.load(Ordering::SeqCst) {
            (self.probe.callback)(context, scope, self);
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<()>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        if !budget.checkpoint(0) {
            return Some(Err(WorthQueryApplicationInvariantExecutionError::new(
                "probe stopped",
            )));
        }
        Some(self.evaluate(context, scope))
    }
}

#[path = "preparation/mod.rs"]
pub mod preparation;
