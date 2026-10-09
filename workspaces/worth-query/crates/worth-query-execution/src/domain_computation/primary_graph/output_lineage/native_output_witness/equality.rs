//! Equal output correspondence and installed revisions across exact settlements.
use super::*;
impl SealedNativeOutputWitness {
    pub(in crate::domain_computation::primary_graph) fn same_output_as(
        &self,
        other: &Self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        admission.charge_external_work(2)?;
        if self.roles.len() != other.roles.len() || self.aspects.len() != other.aspects.len() {
            return Ok(false);
        }
        for (left, right) in self.roles.iter().zip(&other.roles) {
            let work = left
                .role
                .len()
                .checked_add(right.role.len())
                .and_then(|n| n.checked_add(left.entity_name.len()))
                .and_then(|n| n.checked_add(right.entity_name.len()))
                // Posture plus the retirement tag and three lifecycle fields.
                .and_then(|n| n.checked_add(11))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            admission.charge_external_work(work)?;
            if left.role != right.role
                || left.entity_name != right.entity_name
                || left.kind != right.kind
                || left.posture != right.posture
                || left.retirement != right.retirement
                || left.entity.is_none()
                || left.entity != right.entity
                || left.first_aspect != right.first_aspect
                || left.end_aspect != right.end_aspect
            {
                return Ok(false);
            }
        }
        for (left, right) in self.aspects.iter().zip(&other.aspects) {
            admission.charge_external_work(
                u64::try_from(
                    left.aspect
                        .as_str()
                        .len()
                        .checked_add(right.aspect.as_str().len())
                        .and_then(|n| n.checked_add(2))
                        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
                )
                .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
            if left.aspect != right.aspect
                || left.revision.is_none()
                || left.revision != right.revision
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
