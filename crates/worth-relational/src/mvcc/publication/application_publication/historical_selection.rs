//! Exact selected-basis producer custody; a caller supplies no ownership roster.
use super::*;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::{RecordRef, TransactionCommitError},
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod coverage;
pub(crate) use coverage::require_graph_coverage;

/// Prepared from the Native current-fact writers at both retained roots. This
/// capsule retains original versions and grants no ordinary mutation authority.
#[derive(Clone, Debug)]
pub struct RelationalHistoricalOutputSelection {
    pub(crate) current: AdmittedRelationalBranchBasis,
    pub(crate) selected: AdmittedRelationalBranchBasis,
    pub(crate) publication: RelationalRequiredOutputPublication,
    pub(crate) predecessor: Option<RelationalRequiredOutputPublication>,
}
impl RelationalHistoricalOutputSelection {
    pub fn publication(&self) -> &RelationalRequiredOutputPublication {
        &self.publication
    }
}

impl RelationalRuntime {
    pub fn prepare_historical_output_selection(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        route: &RequiredOutputPublicationMeaning,
    ) -> Result<RelationalHistoricalOutputSelection, TransactionCommitError> {
        let subject = RecordRef::Entity(route.source_scope);
        if current.identity().runtime_instance_id() != self.runtime_instance_id()
            || selected.identity().runtime_instance_id() != self.runtime_instance_id()
            || installed.runtime_instance_id != self.runtime_instance_id()
            || installed.schema_digest != current.inner.root.schema_authority().authority_digest()
            || current.inner.root.schema_authority().authority_digest()
                != selected.inner.root.schema_authority().authority_digest()
            || !current.is_current()
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(
                    Cause::HistoricalSelectionBasisChanged,
                    subject,
                ),
            ));
        }
        installed.require_current_ownership_storage(current, route.source_scope)?;
        installed.require_current_ownership_storage(selected, route.source_scope)?;
        execution
            .run_native_with_checkpoint(|context| {
                let selected_roster = Arc::new(crate::publication::ownership::read(
                    execution,
                    selected,
                    std::slice::from_ref(&subject),
                    context,
                )?);
                let current_roster = Arc::new(crate::publication::ownership::read(
                    execution,
                    current,
                    std::slice::from_ref(&subject),
                    context,
                )?);
                let mut capsule = |roster: Arc<
                    crate::publication::ownership::CurrentPublicationRoster,
                >,
                                   basis: &AdmittedRelationalBranchBasis|
                 -> Result<
                    Option<RelationalRequiredOutputPublication>,
                    TransactionCommitError,
                > {
                    let mut found = None;
                    for (origin, ordinal) in roster.coordinates() {
                        context
                            .checkpoint(
                                (origin.meaning.family.len() + route.family.len() + 65) as u64,
                            )
                            .map_err(custody::execution_stop)?;
                        if origin.meaning.same_route(route) {
                            let source_kind = basis
                                .inner
                                .root
                                .get_current_partition(origin.meaning.source_scope.partition_id)
                                .and_then(|region| region.entity(origin.meaning.source_scope))
                                .filter(|slot| slot.is_live())
                                .and_then(|slot| slot.kind_id());
                            if found.is_some()
                                || installed
                                    .source_scope_ownership
                                    .scope_kind(&origin.producer)
                                    != source_kind
                                || !origin.source_scope_owned
                                || installed
                                    .source_scope_ownership
                                    .scope_kind(&origin.producer)
                                    .is_none()
                            {
                                return Err(refuse(origin.commit.commit_id, subject.clone()));
                            }
                            found = Some((Arc::clone(origin), ordinal));
                        }
                    }
                    Ok(
                        found.map(|(origin, ordinal)| RelationalRequiredOutputPublication {
                            runtime_instance_id: installed.runtime_instance_id,
                            authority: installed.authority.seal_required_output_publication_at(
                                origin.commit.commit_id.0,
                                ordinal,
                            ),
                            evidence: crate::publication::ownership::PublicationEvidence::Current(
                                roster,
                            ),
                            origin,
                            readmitted_at: Some(basis.clone()),
                        }),
                    )
                };
                let publication = capsule(selected_roster, selected)?.ok_or_else(|| {
                    TransactionCommitError::publication(PublicationError::authored_retirement(
                        Cause::HistoricalSelectionRouteMissing,
                        subject.clone(),
                    ))
                })?;
                let predecessor = capsule(current_roster, current)?;
                Ok(RelationalHistoricalOutputSelection {
                    current: current.clone(),
                    selected: selected.clone(),
                    publication,
                    predecessor,
                })
            })
            .map_err(custody::execution_stop)?
            .0
    }
}

pub(crate) fn refuse(
    publication: crate::history::data::CommitId,
    subject: RecordRef,
) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        Cause::HistoricalSelectionNotOwned { publication },
        subject,
    ))
}
