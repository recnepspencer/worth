//! One rule evaluator consumes the real installed Native Source loan.
use super::super::{prefix, query, schema::*};
use crate::application_invariant_acceptance::managed_index::maintained::{
    rule_observation::StagedSourceObservation, source_observation::SourceObservation,
};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::application_invariants::*;
type Coordinate = (u32, u64, u32);
pub(crate) struct Calls {
    pub records: Mutex<Vec<StagedSourceObservation<query::ItemRow>>>,
    /// Each invocation's scope and the rows Native lent as changed, on the
    /// committed and the proposed side.
    pub lent: Mutex<Vec<(Coordinate, [Vec<Coordinate>; 2])>>,
    pub after_first_observation: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    provider: Arc<prefix_observation::Calls>,
}
impl Calls {
    pub(crate) fn new(provider: Arc<prefix_observation::Calls>) -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            lent: Mutex::new(Vec::new()),
            after_first_observation: Mutex::new(None),
            provider,
        }
    }
}
pub(crate) struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, prefix::PrefixIndex>,
    items: WorthQueryApplicationInvariantEntityBinding<Catalog, Item>,
    calls: Arc<Calls>,
}
pub(crate) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Catalog>,
    calls: Arc<Calls>,
) -> Result<Rule, String> {
    Ok(Rule {
        index: resolver
            .typed_managed_index(prefix::PrefixIndex::reference())
            .ok_or("missing installed prefix Source requirement")?,
        items: resolver.typed_entity(Item::reference()).ok_or("Item")?,
        calls,
    })
}
impl Rule {
    /// Record the rows Native lent this invocation, their storage claimed first.
    fn lend(
        &self,
        planner: &WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
        prepared: &WorthQueryPreparedInvariantManagedIndex<Catalog, prefix::PrefixIndex>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<(), WorthQueryApplicationInvariantError> {
        let committed = planner.committed_observed_source_rows(prepared, &self.items);
        let committed = match committed {
            Ok(rows) => rows,
            // A new scope has no committed root; its committed side is empty.
            Err(WorthQueryApplicationInvariantError::ManagedIndex(_)) => Vec::new(),
            Err(cause) => return Err(cause),
        };
        let proposed = planner.proposed_observed_source_rows(prepared, &self.items)?;
        let held = (committed.len() + proposed.len()) * size_of::<Coordinate>();
        if !budget.claim_result(held as u64) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let coordinates = |rows: &[WorthQueryApplicationInvariantEntity<Catalog, Item>]| {
            rows.iter()
                .map(|row| {
                    let id = row.entity_id();
                    (
                        id.partition_value(),
                        id.local_slot_value(),
                        id.generation_value(),
                    )
                })
                .collect()
        };
        let id = prepared.scope_identity();
        self.calls.lent.lock().unwrap().push((
            (
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            [coordinates(&committed), coordinates(&proposed)],
        ));
        Ok(())
    }
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        scope: &WorthQueryPreparedInvariantManagedIndex<Catalog, prefix::PrefixIndex>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        if !budget.checkpoint(1) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let updates_before_callback = self
            .calls
            .provider
            .updates
            .load(std::sync::atomic::Ordering::SeqCst);
        // compiled-example: begin
        let pair = context.managed_index(scope).map_err(execution)?;
        let probe = query::Probe {
            prefix: "taken".into(),
        };
        let after = pair.proposed().query(&probe).map_err(execution)?;
        // compiled-example: end
        // Before is additional evidence, never a missing-as-empty domain input.
        // A newly created scope genuinely has no committed root.
        let before = match pair.committed().query(&probe) {
            Ok(rows) => Some(rows),
            Err(cause) if cause.source_unavailable() => None,
            Err(cause) => return Err(execution(cause)),
        };
        let violation = after
            .iter()
            .filter(|row| row.name == "taken")
            .take(2)
            .count()
            == 2;
        let id = scope.scope_identity();
        let bytes = before
            .iter()
            .chain(std::iter::once(&after))
            .flat_map(|rows| rows.iter())
            .try_fold(
                size_of::<StagedSourceObservation<query::ItemRow>>() as u64,
                |bytes, row| {
                    bytes.checked_add(
                        (size_of::<query::ItemRow>()
                            + row.identity.capacity()
                            + row.name.capacity()) as u64,
                    )
                },
            )
            .ok_or(WorthQueryApplicationInvariantError::Allocation)?;
        if !budget.claim_result(bytes) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        self.calls
            .records
            .lock()
            .unwrap()
            .push(StagedSourceObservation {
                source: SourceObservation {
                    scope: (
                        id.partition_value(),
                        id.local_slot_value(),
                        id.generation_value(),
                    ),
                    before,
                    after,
                },
                updates_before_callback,
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
    type Scope = WorthQueryPreparedInvariantManagedIndex<Catalog, prefix::PrefixIndex>;
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
            self.prepare_scope(planner).and_then(|prepared| {
                self.lend(planner, &prepared, budget)?;
                Ok(prepared)
            })
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
