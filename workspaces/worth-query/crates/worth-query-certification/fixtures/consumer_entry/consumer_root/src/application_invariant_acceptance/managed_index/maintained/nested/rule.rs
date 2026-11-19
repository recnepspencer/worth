//! Actual committed/proposed Source rows judge descendant content atomically.
use super::{prefix, query, schema::*};
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationManagedIndexColdSource},
    application_schema::*,
};
use worth_query_host::facade::application_invariants::*;
pub(in super::super) struct CompleteNestedSource;
impl ApplicationInvariantMarkerIdentity<Catalog> for CompleteNestedSource {
    const IDENTIFIER: &'static str = "NoForbiddenProjectedName";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition() -> ApplicationInvariantDefinition<Catalog, CompleteNestedSource> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Directory".into()),
        ApplicationInvariantScopeTarget::Entity("Item".into()),
        ApplicationInvariantScopeTarget::Relation("Contains".into()),
        ApplicationInvariantScopeTarget::Relation("Children".into()),
    ];
    ApplicationInvariantDefinition::new(
        CompleteNestedSource::reference(),
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
#[derive(Default)]
pub(in super::super) struct Calls {
    pub(super) records: Mutex<Vec<Record>>,
    pub(super) acquired: Mutex<Vec<query::ItemRow>>,
}
#[derive(Debug)]
pub(super) struct Record {
    pub scope: (u32, u64, u32),
    pub before: Option<Vec<query::ItemRow>>,
    pub after: Vec<query::ItemRow>,
    /// The Item rows Native observed changing this scope, as the committed
    /// and the proposed views admit them.
    pub observed: [Vec<(u32, u64, u32)>; 2],
}
/// The retained index and the Item rows Native observed changing its scope,
/// as the committed and the proposed views admit them.
pub(super) type Observed = (
    WorthQueryPreparedInvariantManagedIndex<Catalog, prefix::PrefixIndex>,
    [Vec<(u32, u64, u32)>; 2],
);
pub(super) struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, prefix::PrefixIndex>,
    items: WorthQueryApplicationInvariantEntityBinding<Catalog, Item>,
    calls: Arc<Calls>,
}
pub(super) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Catalog>,
    calls: Arc<Calls>,
) -> Result<Rule, String> {
    Ok(Rule {
        index: resolver
            .typed_managed_index(prefix::PrefixIndex::reference())
            .ok_or("nested Source was not installed")?,
        items: resolver.typed_entity(Item::reference()).ok_or("Item")?,
        calls,
    })
}
impl WorthQueryApplicationInvariantRule<Catalog> for Rule {
    type Scope = Observed;
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError> {
        let prepared = planner
            .retain_managed_index(&self.index)
            .map_err(WorthQueryApplicationInvariantError::from)?;
        // Lent rows are read where preparation admits them.
        let rows = |view: WorthQueryApplicationInvariantReadView<'_, Catalog>| {
            view.observed_source_rows_of(&prepared, &self.items)
                .map(|rows| {
                    rows.iter()
                        .map(|row| {
                            let id = row.entity_id();
                            (
                                id.partition_value(),
                                id.local_slot_value(),
                                id.generation_value(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
        };
        let committed = rows(planner.committed()).map_err(execution)?;
        let proposed = rows(planner.proposed()).map_err(execution)?;
        Ok((prepared, [committed, proposed]))
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
impl Rule {
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        scope: &Observed,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        if !budget.checkpoint(1) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let pair = context.managed_index(&scope.0).map_err(execution)?;
        let probe = query::Probe {
            prefix: String::new(),
        };
        let after = pair.proposed().query(&probe).map_err(execution)?;
        let before = match pair.committed().query(&probe) {
            Ok(rows) => Some(rows),
            Err(cause) if cause.source_unavailable() => None,
            Err(cause) => return Err(execution(cause)),
        };
        let mut invalid = false;
        let mut bytes = size_of::<Record>();
        for rows in before.iter().chain(std::iter::once(&after)) {
            bytes = bytes
                .checked_add(layout::<query::ItemRow>(rows.capacity())?)
                .ok_or(WorthQueryApplicationInvariantError::Allocation)?;
            for row in rows {
                if !budget.checkpoint(1) {
                    return Err(WorthQueryApplicationInvariantError::Stopped);
                }
                bytes = add(bytes, row.identity.capacity())?;
                bytes = add(bytes, row.name.capacity())?;
                bytes = add(
                    bytes,
                    layout::<super::shape::Node>(row.children.capacity())?,
                )?;
                bytes = add(bytes, layout::<super::shape::Node>(row.parents.capacity())?)?;
                for node in row.children.iter().chain(&row.parents) {
                    bytes = add(bytes, size_nodes(node, budget)?)?;
                }
            }
        }
        for row in &after {
            invalid |=
                row.name == "forbidden" || row.children.iter().chain(&row.parents).any(forbidden);
        }
        if !budget.claim_result(bytes as u64) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let held = layout::<(u32, u64, u32)>(scope.1[0].len() + scope.1[1].len())?;
        if !budget.claim_result(held as u64) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let id = scope.0.scope_identity();
        self.calls.records.lock().unwrap().push(Record {
            scope: (
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            before,
            after,
            observed: scope.1.clone(),
        });
        Ok(if invalid {
            WorthQueryApplicationInvariantVerdict::Violation(())
        } else {
            WorthQueryApplicationInvariantVerdict::Pass
        })
    }
}
fn forbidden(node: &super::shape::Node) -> bool {
    node.name == "forbidden" || node.children.iter().any(forbidden)
}
fn size_nodes(
    node: &super::shape::Node,
    budget: &dyn CustomInvariantLeaseBudget,
) -> Result<usize, WorthQueryApplicationInvariantError> {
    if !budget.checkpoint(1) {
        return Err(WorthQueryApplicationInvariantError::Stopped);
    }
    let mut bytes = add(node.identity.capacity(), node.name.capacity())?;
    bytes = add(
        bytes,
        layout::<super::shape::Node>(node.children.capacity())?,
    )?;
    for child in &node.children {
        bytes = bytes
            .checked_add(size_nodes(child, budget)?)
            .ok_or(WorthQueryApplicationInvariantError::Allocation)?;
    }
    Ok(bytes)
}
fn execution(
    cause: impl Into<WorthQueryApplicationInvariantError>,
) -> WorthQueryApplicationInvariantError {
    cause.into()
}

fn add(left: usize, right: usize) -> Result<usize, WorthQueryApplicationInvariantError> {
    left.checked_add(right)
        .ok_or(WorthQueryApplicationInvariantError::Allocation)
}
fn layout<T>(count: usize) -> Result<usize, WorthQueryApplicationInvariantError> {
    count
        .checked_mul(size_of::<T>())
        .ok_or(WorthQueryApplicationInvariantError::Allocation)
}
