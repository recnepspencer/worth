//! Diagnostic reads have their own installed rule and frozen admission frame.
use super::{observation::Entity, schema::*};
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_query_declaration::facade::application_schema::*;
mod preparation;

pub(super) struct WalkProbe;
impl ApplicationInvariantMarkerIdentity<WalkSchema> for WalkProbe {
    const IDENTIFIER: &'static str = "graph-walk-probe";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(super) fn definition() -> ApplicationInvariantDefinition<WalkSchema, WalkProbe> {
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

pub(super) struct Scope {
    pub(super) proposed: Vec<(u64, Entity)>,
    pub(super) committed: Vec<(u64, Entity)>,
}
impl Scope {
    pub(super) fn node(&self, key: u64) -> &Entity {
        &self
            .proposed
            .iter()
            .find(|(k, _)| *k == key)
            .expect("authored node")
            .1
    }
}

type Callback = dyn for<'a, 'b> Fn(&WorthQueryApplicationInvariantContext<'a, 'b, WalkSchema>, &Scope, &Rule)
    + Send
    + Sync
    + std::panic::RefUnwindSafe
    + std::panic::UnwindSafe;

#[derive(Clone)]
pub(super) struct Probe {
    enabled: bool,
    armed: Arc<AtomicBool>,
    callback: Arc<Callback>,
}
impl Probe {
    pub(super) fn new(
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
    pub(super) fn arm(&self) {
        self.armed.store(true, Ordering::SeqCst);
    }

    pub(super) fn disabled() -> Self {
        let mut probe = Self::new(|_, _, _| {});
        probe.enabled = false;
        probe
    }
}

pub(super) struct Rule {
    probe: Probe,
    pub(super) key: WorthQueryApplicationInvariantFieldBinding<WalkSchema, Node, u64>,
    membership: WorthQueryApplicationInvariantRelationBinding<WalkSchema, GroupRoot, Node, Node>,
    pub(super) next: WorthQueryApplicationInvariantRelationBinding<WalkSchema, Next, Node, Node>,
}
pub(super) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, WalkSchema>,
    probe: Probe,
) -> Result<Rule, String> {
    Ok(Rule {
        probe,
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
