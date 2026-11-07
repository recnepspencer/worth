//! A directory may have at most two edge enclosures covering the fixed point.
use super::super::{bounds::Coordinates, schema::Catalog};
use super::{provider, query};
use std::sync::{atomic::Ordering, Arc, Mutex};
use worth_query_decl::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationManagedIndexColdSource},
    application_schema::*,
};
use worth_query_host::facade::application_invariants::*;

pub struct ProbeContact;
impl ApplicationInvariantMarkerIdentity<Catalog> for ProbeContact {
    const IDENTIFIER: &'static str = "AtMostTwoProbeEnclosures";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition() -> ApplicationInvariantDefinition<Catalog, ProbeContact> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Directory".into()),
        ApplicationInvariantScopeTarget::Entity("Edge".into()),
        ApplicationInvariantScopeTarget::Entity("Vertex".into()),
        ApplicationInvariantScopeTarget::Relation("ContainsEdge".into()),
        ApplicationInvariantScopeTarget::Relation("Starts".into()),
        ApplicationInvariantScopeTarget::Relation("Ends".into()),
    ];
    ApplicationInvariantDefinition::new(
        ProbeContact::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
    .managed_index_source(provider::BoxIndex::reference())
    .managed_index_cold_sources(ApplicationManagedIndexColdSource::AdmitRebuild)
}
pub(super) fn probe() -> Coordinates {
    Coordinates::new(50, 0, 50, 0)
}
#[derive(Debug)]
pub struct Record {
    pub scope: (u32, u64, u32),
    pub before: Option<Vec<query::Row>>,
    pub after: Vec<query::Row>,
    pub updates_before_callback: usize,
    pub builds_before_callback: usize,
    pub source_before: Option<Vec<query::Row>>,
    pub source_after: Option<Vec<query::Row>>,
}
pub struct Calls {
    pub records: Mutex<Vec<Record>>,
    pub provider: Arc<provider::Calls>,
    pub capture_complete_source: std::sync::atomic::AtomicBool,
    pub after_first_observation: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl Calls {
    pub fn new(provider: Arc<provider::Calls>) -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            provider,
            capture_complete_source: std::sync::atomic::AtomicBool::new(false),
            after_first_observation: Mutex::new(None),
        }
    }
}
pub struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, provider::BoxIndex>,
    calls: Arc<Calls>,
}
pub(super) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Catalog>,
    calls: Arc<Calls>,
) -> Result<Rule, String> {
    Ok(Rule {
        index: resolver
            .typed_managed_index(provider::BoxIndex::reference())
            .ok_or("missing geometry Source")?,
        calls,
    })
}
impl Rule {
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        scope: &WorthQueryPreparedInvariantManagedIndex<Catalog, provider::BoxIndex>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        if !budget.checkpoint(1) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let updates_before_callback = self.calls.provider.updates.load(Ordering::SeqCst);
        let builds_before_callback = self.calls.provider.builds.load(Ordering::SeqCst);
        let pair = context.managed_index(scope).map_err(execution)?;
        let after = pair.proposed().query(&probe()).map_err(execution)?;
        let before = match pair.committed().query(&probe()) {
            Ok(rows) => Some(rows),
            Err(cause) if cause.source_unavailable() => None,
            Err(cause) => return Err(execution(cause)),
        };
        let (source_before, source_after) =
            if self.calls.capture_complete_source.load(Ordering::SeqCst) {
                let all = Coordinates::new(-1, -1, 100, 1);
                let proposed = pair.proposed().query(&all).map_err(execution)?;
                let committed = match pair.committed().query(&all) {
                    Ok(rows) => Some(rows),
                    Err(cause) if cause.source_unavailable() => None,
                    Err(cause) => return Err(execution(cause)),
                };
                (committed, Some(proposed))
            } else {
                (None, None)
            };
        let violation = after.len() > 2;
        let bytes = before
            .iter()
            .chain(std::iter::once(&after))
            .chain(source_before.iter())
            .chain(source_after.iter())
            .flat_map(|rows| rows.iter())
            .try_fold(size_of::<Record>() as u64, |bytes, row| {
                bytes.checked_add((size_of::<query::Row>() + row.identity.capacity()) as u64)
            })
            .ok_or(WorthQueryApplicationInvariantError::Allocation)?;
        if !budget.claim_result(bytes) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let scope = scope.scope_identity();
        self.calls.records.lock().unwrap().push(Record {
            scope: (
                scope.partition_value(),
                scope.local_slot_value(),
                scope.generation_value(),
            ),
            before,
            after,
            updates_before_callback,
            builds_before_callback,
            source_before,
            source_after,
        });
        if let Some(observe) = self.calls.after_first_observation.lock().unwrap().take() {
            observe();
        }
        Ok(if violation {
            WorthQueryApplicationInvariantVerdict::Violation(())
        } else {
            WorthQueryApplicationInvariantVerdict::Pass
        })
    }
}
impl WorthQueryApplicationInvariantRule<Catalog> for Rule {
    type Scope = WorthQueryPreparedInvariantManagedIndex<Catalog, provider::BoxIndex>;
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError> {
        planner
            .retain_managed_index(&self.index)
            .map_err(WorthQueryApplicationInvariantError::from)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError>> {
        Some(if budget.checkpoint(1) {
            self.prepare_scope(planner)
        } else {
            Err(WorthQueryApplicationInvariantError::Stopped)
        })
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        _: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        Err(WorthQueryApplicationInvariantError::CheckedHookRequired)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>,
    > {
        Some(self.judge(context, scope, budget))
    }
}
fn execution(
    cause: impl Into<WorthQueryApplicationInvariantError>,
) -> WorthQueryApplicationInvariantError {
    cause.into()
}
