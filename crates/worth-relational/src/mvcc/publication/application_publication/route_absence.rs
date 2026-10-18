//! Exact native negative observation for an otherwise unindexed recovered route.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::branch::AdmittedRelationalBranchBasis;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};

#[derive(Debug)]
pub struct RelationalRequiredOutputRouteAbsence {
    installed: worth_proof::InstalledApplicationPublicationAuthority,
    basis: AdmittedRelationalBranchBasis,
    meaning: RequiredOutputPublicationMeaning,
    retirement: Option<crate::history::data::CommitId>,
    _funding: ExecutionMemoryReservation,
}

impl RelationalRequiredOutputRouteAbsence {
    pub(crate) fn retirement(&self) -> Option<crate::history::data::CommitId> {
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
    ) -> Result<Option<crate::history::data::CommitId>, TransactionCommitError> {
        use crate::history::data::RequiredOutputRoutePosition as Position;
        let positions = &basis.inner.root.output_publication_positions;
        match positions.get(meaning, context)? {
            Some(Position::Published(_)) => Err(origin_denial(
                "an installed family publication requires its exact predecessor",
            )),
            Some(Position::CodecRetired(commit)) => Ok(Some(commit)),
            None if positions.unresolved_prefix().is_some() => {
                let absence = absence.filter(|absence| absence.covers(installed, basis, meaning))
                    .ok_or_else(|| origin_denial("Birth requires native cold route-absence readmission for the unresolved canonical prefix"))?;
                Ok(absence.retirement())
            }
            None => Ok(None),
        }
    }
}

impl crate::runtime::RelationalRuntime {
    /// Funded cold admission when the recovered selector does not cover its
    /// canonical prefix. A complete warm image uses only its reached route.
    pub fn readmit_required_output_route_absence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<RelationalRequiredOutputRouteAbsence, TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id()
            || basis.descriptor().runtime_instance_id() != self.runtime_instance_id()
            || !basis.is_current()
        {
            return Err(origin_denial(
                "route absence requires the exact installed native basis",
            ));
        }
        let result = execution.in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    let positions = &basis.inner.root.output_publication_positions;
                    use crate::history::data::RequiredOutputRoutePosition as Position;
                    let selected = match positions.get(meaning, context)? {
                        Some(position) => Some(position),
                        None if positions.unresolved_prefix().is_some() => {
                            crate::history::data::readmit_required_output_route(
                                execution, self, basis, meaning,
                            )
                            .map_err(super::custody::cold_stop)?
                        }
                        None => None,
                    };
                    let retirement = match selected {
                        Some(Position::Published(_)) => {
                            return Err(origin_denial(
                                "Birth route already has a complete native publication",
                            ))
                        }
                        Some(Position::CodecRetired(commit)) => Some(commit),
                        None => None,
                    };
                    let bytes = (std::mem::size_of::<RelationalRequiredOutputRouteAbsence>()
                        + meaning.family.len()) as u64;
                    context
                        .checkpoint(bytes)
                        .map_err(super::custody::execution_stop)?;
                    let funding = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
                        .map_err(|stop| {
                            super::custody::execution_stop(crate::execution::denial::lease_denial(
                                stop,
                            ))
                        })?;
                    Ok(RelationalRequiredOutputRouteAbsence {
                        installed: installed.authority.clone(),
                        basis: basis.clone(),
                        meaning: meaning.clone(),
                        retirement,
                        _funding: funding,
                    })
                },
            )
        });
        consult_stage(execution, CommitStage::Admission)?;
        result
            .map_err(super::custody::execution_stop)?
            .map_err(super::custody::execution_stop)?
            .0
    }
}
