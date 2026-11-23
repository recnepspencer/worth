//! Select retirement candidates from the authenticated live ownership roster.
use crate::history::data::codec_retirement_closure::{
    authored_cascade::stopped,
    charge,
    scope_dependents::linked_owners,
    source_scope_closure::{from_current, select},
    SelectedOutputPublication,
};
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture as Posture;
use crate::schema::installed_source_scopes::InstalledSourceScopeOwnership;
use std::sync::Arc;
use worth_execution::MapKernelContext;

/// Current ownership is selected solely from authenticated current region facts.
pub(crate) fn current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    ownership: Option<&InstalledSourceScopeOwnership>,
    records: &[crate::transactions::data::primitives::RecordRef],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<
    Vec<SelectedOutputPublication>,
    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    use crate::transactions::data::primitives::RecordRef;
    use std::collections::BTreeSet;
    let roots = records
        .iter()
        .filter_map(|r| {
            if let RecordRef::Entity(id) = r {
                Some(*id)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut frontier = records.to_vec();
    let mut visited = BTreeSet::new();
    let mut latest: Vec<SelectedOutputPublication> = Vec::new();
    while !frontier.is_empty() {
        let mut selectors =
            crate::publication::ownership::linked_selectors(execution, basis, &frontier, context)?;
        selectors.append(&mut frontier);
        let roster = Arc::new(crate::publication::ownership::read(
            execution, basis, &selectors, context,
        )?);
        for publication in from_current(roster, ownership, context)? {
            charge(context, latest.len() + 1).map_err(stopped)?;
            if !latest
                .iter()
                .any(|p| p.origin.same_publication(&publication.origin))
            {
                latest.push(publication);
            }
        }
        // This is the existing closure owner. Only publications selected under
        // the actual cascade roots, or their authenticated required consumers, may
        // contribute another ownership frontier. No unrelated member expands it.
        let retiring = select(&roots, &latest, context).map_err(stopped)?;
        let consuming = linked_owners(&roots, &retiring, &latest, context).map_err(stopped)?;
        for publication in &consuming {
            let source = publication.origin.meaning.source_scope;
            charge(context, 1).map_err(stopped)?;
            if visited.insert(source) {
                frontier.push(RecordRef::Entity(source));
            }
        }
        for publication in retiring {
            for member in &publication.origin.members {
                charge(context, 1).map_err(stopped)?;
                if member.posture != Posture::Retire && visited.insert(member.id) {
                    frontier.push(RecordRef::Entity(member.id));
                }
            }
        }
    }
    Ok(latest)
}
