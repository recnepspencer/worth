//! One rule evaluator consumes the real installed Native Source loan.
use super::super::{prefix, query, schema::*};

use std::sync::{Arc, Mutex};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::application_invariants::*;
pub(crate) struct Calls {
    pub records: Mutex<Vec<Record>>,
    pub after_first_observation: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    provider: Arc<prefix::Calls>,
}
impl Calls {
    pub(crate) fn new(provider: Arc<prefix::Calls>) -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            after_first_observation: Mutex::new(None),
            provider,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Record {
    pub scope: (u32, u64, u32),
    pub before: Option<Vec<query::ItemRow>>,
    pub after: Vec<query::ItemRow>,
    pub updates_before_callback: usize,
}
pub(crate) struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, prefix::PrefixIndex>,
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
        calls,
    })
}
impl Rule {
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
            .try_fold(size_of::<Record>() as u64, |bytes, row| {
                bytes.checked_add(
                    (size_of::<query::ItemRow>() + row.identity.capacity() + row.name.capacity())
                        as u64,
                )
            })
            .ok_or(WorthQueryApplicationInvariantError::Allocation)?;
        if !budget.claim_result(bytes) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        self.calls.records.lock().unwrap().push(Record {
            scope: (
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            before,
            after,
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
