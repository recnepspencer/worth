//! Exact native negative observation for an otherwise unindexed recovered route.
use worth_relational_contracts::internal_port as contracts;
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use worth_relational_branch::internal_port::branch::basis::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _, AdmittedRelationalBranchBasisOwnerPort as _};
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};

#[derive(Debug)]
pub struct RelationalRequiredOutputRouteAbsence {
    installed: worth_proof::InstalledApplicationPublicationAuthority,
    basis: AdmittedRelationalBranchBasis,
    meaning: RequiredOutputPublicationMeaning,
    retirement: Option<contracts::history::data::coordinates::CommitId>,
    _funding: ExecutionMemoryReservation,
}

impl RelationalRequiredOutputRouteAbsence {
    pub(crate) fn retirement(&self) -> Option<contracts::history::data::coordinates::CommitId> {
        self.retirement
    }
    pub(crate) fn covers(
        &self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> bool {
        self.installed.same_publication_owner(&installed.authority)
            && self.basis.descriptor() == basis.descriptor()
            && &self.meaning == meaning
    }
}

impl RequiredOutputPublicationAdmission {
    pub(crate) fn validate_before_preparation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), TransactionCommitError> {
        let RequiredOutputPublication::Birth {
            meaning, absence, ..
        } = &self.publication
        else {
            return Ok(());
        };
        let result = execution.in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    Self::birth_retirement_in_context(
                        &self.installed,
                        basis,
                        meaning,
                        absence.as_ref(),
                        context,
                    )
                    .map(|_| ())
                },
            )
        });
        consult_stage(execution, CommitStage::Admission)?;
        result
            .map_err(super::custody::execution_stop)?
            .map_err(super::custody::execution_stop)?
            .0
    }

    pub(crate) fn birth_retirement_in_context(
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
        absence: Option<&RelationalRequiredOutputRouteAbsence>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<contracts::history::data::coordinates::CommitId>, TransactionCommitError>
    {
        use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition as Position;
        let positions = &basis.owner_inner().root.output_publication_positions;
        match positions.get(meaning, context)? {
            Some(Position::Published(_)) => Err(origin_denial(
                "an installed family publication requires its exact predecessor",
            )),
            Some(Position::CodecRetired(commit)) => Ok(Some(commit)),
            Some(Position::HistoricalAbsence(_)) => Ok(None),
            None if positions.unresolved_prefix().is_some() => {
                let absence = absence.filter(|absence| absence.covers(installed, basis, meaning))
                    .ok_or_else(|| origin_denial("Birth requires native cold route-absence readmission for the unresolved canonical prefix"))?;
                Ok(absence.retirement())
            }
            None => Ok(None),
        }
    }
}

pub fn readmit_required_output_route_absence(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RelationalRequiredOutputRouteAbsence>, TransactionCommitError> {
    if installed.runtime_instance_id != inputs.projection.runtime_identity
        || basis.descriptor().runtime_instance_id() != inputs.projection.runtime_identity
        || !basis.is_current()
    {
        return Err(origin_denial(
            "route absence requires the exact installed native basis",
        ));
    }
    if installed
        .source_scope_ownership
        .scope_kind(&meaning.family)
        .is_some()
    {
        installed.require_current_ownership_storage(basis, meaning.source_scope)?;
    }
    let current = if installed.source_scope_ownership.is_empty()
        || basis
            .owner_inner()
            .root
            .output_publication_positions
            .unresolved_prefix()
            .is_none()
    {
        None
    } else {
        crate::publication::ownership::route_selection::read_current(execution, basis, meaning)?
    };
    let result = execution.in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    let positions = &basis.owner_inner().root.output_publication_positions;
                    use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition as Position;
                    let selected = match positions.get(meaning, context)?.or(current) {
                        Some(position) => Some(position),
                        None if positions.unresolved_prefix().is_some() => {
                            let historical=worth_relational_branch::internal_port::history::data::codec_retirement_closure::route_selection::readmit_required_output_route(
                                execution, inputs, basis, meaning,
                            ).map_err(super::custody::cold_stop)?;
                            if installed.source_scope_ownership.is_empty() {historical} else {
                                worth_relational_branch::internal_port::history::data::required_output_positions::presence::history_fallback::require_current_owned_route(
                                    execution,inputs.canonical_history,basis,meaning,historical,
                                )?
                            }
                        }
                        None => None,
                    };
                    let retirement = match selected {
                        Some(Position::Published(_)) => return Ok(None),
                        Some(Position::CodecRetired(commit)) => Some(commit),
                        Some(Position::HistoricalAbsence(_)) => None,
                        None => None,
                    };
                    let bytes = (std::mem::size_of::<RelationalRequiredOutputRouteAbsence>()
                        + meaning.family.len()) as u64;
                    context
                        .checkpoint(bytes)
                        .map_err(super::custody::execution_stop)?;
                    let funding = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
                        .map_err(|stop| {
                            super::custody::execution_stop(contracts::execution::denial::lease_denial(
                                stop,
                            ))
                        })?;
                    Ok(Some(RelationalRequiredOutputRouteAbsence {
                        installed: installed.authority.clone(),
                        basis: basis.clone(),
                        meaning: meaning.clone(),
                        retirement,
                        _funding: funding,
                    }))
                },
            )
        });
    consult_stage(execution, CommitStage::Admission)?;
    result
        .map_err(super::custody::execution_stop)?
        .map_err(super::custody::execution_stop)?
        .0
}
