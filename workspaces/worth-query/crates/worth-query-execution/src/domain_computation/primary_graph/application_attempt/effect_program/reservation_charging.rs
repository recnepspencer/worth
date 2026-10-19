use super::{
    CandidateItemKind, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationEffectProgramBuilder,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(super) fn charge_candidate_item(
        &mut self,
        kind: CandidateItemKind,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        match self.candidate_reservation.as_mut() {
            Some(reservation) => reservation.charge(kind),
            None => Ok(()),
        }
    }

    pub(super) fn charge_candidate_representation(
        &mut self,
        kind: CandidateItemKind,
        retained_representation_bytes: usize,
        replaced_representation_bytes: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        match self.candidate_reservation.as_mut() {
            Some(reservation) => reservation.charge_retained_representation(
                kind,
                retained_representation_bytes,
                replaced_representation_bytes,
            ),
            None => Ok(()),
        }
    }

    pub(super) fn charge_candidate_items(
        &mut self,
        kind: CandidateItemKind,
        count: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        match self.candidate_reservation.as_mut() {
            Some(reservation) => reservation.charge_many(kind, count, 0, 0),
            None => Ok(()),
        }
    }

    pub(super) fn charge_candidate_correspondence(
        &mut self,
        bytes: usize,
        entries: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some(reservation) = &self.candidate_reservation {
            reservation.check_representation_only(bytes)?;
        }
        let storage = super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::entry_storage_bound(entries)
            .ok_or_else(super::candidate_representation_denial)?;
        self.read_set
            .admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let funded = self
            .candidate_funding
            .as_mut()
            .map_or(Ok(()), |funding| funding.grow_correspondence(storage));
        self.read_set
            .admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        funded?;
        self.charge_candidate_representation_only(bytes)
    }

    pub(super) fn charge_candidate_representation_only(
        &mut self,
        retained_representation_bytes: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        match self.candidate_reservation.as_mut() {
            Some(reservation) => {
                reservation.charge_representation_only(retained_representation_bytes)
            }
            None => Ok(()),
        }
    }
}
