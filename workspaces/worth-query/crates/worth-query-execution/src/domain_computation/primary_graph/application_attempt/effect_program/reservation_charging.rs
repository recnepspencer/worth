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
