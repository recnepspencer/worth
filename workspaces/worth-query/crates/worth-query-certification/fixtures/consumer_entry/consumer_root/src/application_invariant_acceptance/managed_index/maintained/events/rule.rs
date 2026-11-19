//! The actual ordered-event Source participates in ordinary invariant dispatch.
use super::{prefix, query, schema::*};
use crate::application_invariant_acceptance::managed_index::maintained::source_observation::SourceObservation;
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::*,
};
use worth_query_host::facade::application_invariants::*;
pub(in super::super) struct CompleteNestedSource;
impl ApplicationInvariantMarkerIdentity<Catalog> for CompleteNestedSource {
    const IDENTIFIER: &'static str = "NoForbiddenProjectedName";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition() -> ApplicationInvariantDefinition<Catalog, CompleteNestedSource> {
    let definition = base_definition();
    if super::VARIANT_SOURCES {
        definition
            .managed_index_source(super::variants::SameScopeIndex::reference())
            .managed_index_source(super::variants::OverlapIndex::reference())
    } else {
        definition
    }
}
fn base_definition() -> ApplicationInvariantDefinition<Catalog, CompleteNestedSource> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Directory".into()),
        ApplicationInvariantScopeTarget::Entity("Item".into()),
        ApplicationInvariantScopeTarget::Relation("Contains".into()),
        ApplicationInvariantScopeTarget::Relation("Previous".into()),
        ApplicationInvariantScopeTarget::Relation("Retires".into()),
        ApplicationInvariantScopeTarget::Relation("Children".into()),
    ];
    ApplicationInvariantDefinition::new(
        CompleteNestedSource::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [
                ApplicationInvariantGroup::SchemaCompliance,
                ApplicationInvariantGroup::RelationIntegrity,
            ],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
    .managed_index_source(prefix::PrefixIndex::reference())
    .managed_index_cold_sources(super::SOURCE_COLD_POLICY)
}
#[derive(Default)]
pub(in super::super) struct Calls {
    pub(super) records: Mutex<Vec<SourceObservation<query::ItemRow>>>,
    /// Each invocation's scope and the Item rows Native observed changing
    /// it, as the committed and the proposed views admit them.
    pub(super) observed: Mutex<Vec<(&'static str, Coordinate, [Vec<Coordinate>; 2])>>,
    /// Set by a cost court to measure the same commit without the loan.
    pub(super) skip_loan: std::sync::atomic::AtomicBool,
    /// Set by a cost court to lend from the ordered Source alone.
    pub(super) only_ordered: std::sync::atomic::AtomicBool,
    /// Each evaluation's invocation scope, the scope entity its preparation
    /// minted, and that entity's DirectoryIdentity read through evaluation.
    pub(super) scopes: Mutex<Vec<(Coordinate, Coordinate, Option<String>)>>,
}
pub(super) type Coordinate = (u32, u64, u32);
pub(super) struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, prefix::PrefixIndex>,
    same_scope: Option<
        WorthQueryApplicationInvariantManagedIndexBinding<Catalog, super::variants::SameScopeIndex>,
    >,
    overlap: Option<
        WorthQueryApplicationInvariantManagedIndexBinding<Catalog, super::variants::OverlapIndex>,
    >,
    items: WorthQueryApplicationInvariantEntityBinding<Catalog, Item>,
    directories: WorthQueryApplicationInvariantEntityBinding<Catalog, Directory>,
    directory_identity: WorthQueryApplicationInvariantFieldBinding<Catalog, Directory, String>,
    calls: Arc<Calls>,
}
/// The retained ordered Source and, when preparation was checked, the
/// invocation's scope minted as a typed Directory.
pub(super) type Prepared = (
    WorthQueryPreparedInvariantManagedIndex<Catalog, prefix::PrefixIndex>,
    Option<WorthQueryApplicationInvariantEntity<Catalog, Directory>>,
);
pub(super) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Catalog>,
    calls: Arc<Calls>,
) -> Result<Rule, String> {
    Ok(Rule {
        index: resolver
            .typed_managed_index(prefix::PrefixIndex::reference())
            .ok_or("nested Source was not installed")?,
        same_scope: resolver.typed_managed_index(super::variants::SameScopeIndex::reference()),
        overlap: resolver.typed_managed_index(super::variants::OverlapIndex::reference()),
        items: resolver.typed_entity(Item::reference()).ok_or("Item")?,
        directories: resolver
            .typed_entity(Directory::reference())
            .ok_or("Directory")?,
        directory_identity: resolver
            .typed_field(DirectoryIdentity::at("DirectoryIdentity"))
            .ok_or("DirectoryIdentity")?,
        calls,
    })
}
impl WorthQueryApplicationInvariantRule<Catalog> for Rule {
    type Scope = Prepared;
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
        Ok((prepared, None))
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError>> {
        Some(if budget.checkpoint(1) {
            self.prepare_scope(planner).and_then(|(prepared, _)| {
                use std::sync::atomic::Ordering::SeqCst;
                // Every declared Source is retained whatever is lent, so a cost
                // court varies only the number of loans.
                let same = self
                    .same_scope
                    .as_ref()
                    .map(|binding| planner.retain_managed_index(binding))
                    .transpose()?;
                let overlap = self
                    .overlap
                    .as_ref()
                    .map(|binding| planner.retain_managed_index(binding))
                    .transpose()?;
                if !self.calls.skip_loan.load(SeqCst) {
                    self.observe(planner, "ordered", &prepared, budget)?;
                    if !self.calls.only_ordered.load(SeqCst) {
                        if let Some(same) = &same {
                            self.observe(planner, "same-scope", same, budget)?;
                        }
                        if let Some(overlap) = &overlap {
                            self.observe(planner, "overlap", overlap, budget)?;
                        }
                    }
                }
                let scope = planner.observed_scope(&prepared, &self.directories)?;
                Ok((prepared, Some(scope)))
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
impl Rule {
    /// Record the rows Native lent this invocation, claiming their storage.
    fn observe<I>(
        &self,
        planner: &WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
        index: &'static str,
        prepared: &WorthQueryPreparedInvariantManagedIndex<Catalog, I>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<(), WorthQueryApplicationInvariantError> {
        let committed = planner.committed_observed_source_rows(prepared, &self.items)?;
        let proposed = planner.proposed_observed_source_rows(prepared, &self.items)?;
        let held = (committed.len() + proposed.len()) * size_of::<Coordinate>();
        if !budget.claim_result(held as u64) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let coordinates = |rows: &[WorthQueryApplicationInvariantEntity<Catalog, Item>]| {
            rows.iter().map(|row| coordinate(row.entity_id())).collect()
        };
        self.calls.observed.lock().unwrap().push((
            index,
            coordinate(prepared.scope_identity()),
            [coordinates(&committed), coordinates(&proposed)],
        ));
        Ok(())
    }
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        (scope, minted): &Prepared,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        if !budget.checkpoint(1) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        // The minted scope is admitted: evaluation reads its own field.
        if let Some(minted) = minted {
            let name = context
                .proposed()
                .field(&self.directory_identity, minted)
                .map_err(execution)?;
            self.calls.scopes.lock().unwrap().push((
                coordinate(scope.scope_identity()),
                coordinate(minted.entity_id()),
                name,
            ));
        }
        let pair = context.managed_index(scope).map_err(execution)?;
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
        let mut bytes = size_of::<SourceObservation<query::ItemRow>>();
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
        let id = scope.scope_identity();
        self.calls.records.lock().unwrap().push(SourceObservation {
            scope: (
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            before,
            after,
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

fn coordinate(id: EntityId) -> Coordinate {
    (
        id.partition_value(),
        id.local_slot_value(),
        id.generation_value(),
    )
}
