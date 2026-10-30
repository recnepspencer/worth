//! The actual installed per-directory law, with observation-only fixture evidence.
use super::{prefix, query, schema::*};
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationManagedIndexColdSource},
    application_schema::*,
};
use worth_query_host::facade::application_invariants::*;

pub(super) struct ReservedName;
impl ApplicationInvariantMarkerIdentity<Catalog> for ReservedName {
    const IDENTIFIER: &'static str = "AtMostOneTakenName";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition() -> ApplicationInvariantDefinition<Catalog, ReservedName> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Directory".into()),
        ApplicationInvariantScopeTarget::Entity("Item".into()),
        ApplicationInvariantScopeTarget::Relation("Contains".into()),
    ];
    ApplicationInvariantDefinition::new(
        ReservedName::reference(),
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
    .managed_index_source(prefix::PrefixIndex::reference())
    .managed_index_cold_sources(ApplicationManagedIndexColdSource::AdmitRebuild)
}
pub(super) struct Calls {
    pub records: Mutex<Vec<Record>>,
    provider: Arc<prefix::Calls>,
}
impl Calls {
    pub(super) fn new(provider: Arc<prefix::Calls>) -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            provider,
        }
    }
}
#[derive(Debug)]
pub(super) struct Record {
    pub scope: (u32, u64, u32),
    pub before: Option<Vec<query::ItemRow>>,
    pub after: Vec<query::ItemRow>,
    pub updates_before_callback: usize,
}
pub(super) struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, prefix::PrefixIndex>,
    calls: Arc<Calls>,
}
pub(super) fn resolve(
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
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        if !budget.checkpoint(1) {
            return Err(execution("original request stopped"));
        }
        let updates_before_callback = self
            .calls
            .provider
            .updates
            .load(std::sync::atomic::Ordering::SeqCst);
        // compiled-example: begin
        let pair = context
            .managed_index(scope)
            .map_err(|cause| execution(format!("{cause:?}")))?;
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
            .ok_or_else(|| execution("record size overflow"))?;
        if !budget.claim_result(bytes) {
            return Err(execution("observation storage stopped"));
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
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        planner.retain_managed_index(&self.index).map_err(|cause| {
            WorthQueryApplicationInvariantPreparationError::new(format!("{cause:?}"))
        })
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(if budget.checkpoint(1) {
            self.prepare_scope(planner)
        } else {
            Err(WorthQueryApplicationInvariantPreparationError::new(
                "original preparation stopped",
            ))
        })
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        _: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        Err(execution(
            "this application requires the original checked rule budget",
        ))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<()>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        Some(self.judge(context, scope, budget))
    }
}
fn execution(cause: impl std::fmt::Display) -> WorthQueryApplicationInvariantExecutionError {
    WorthQueryApplicationInvariantExecutionError::new(cause.to_string())
}
