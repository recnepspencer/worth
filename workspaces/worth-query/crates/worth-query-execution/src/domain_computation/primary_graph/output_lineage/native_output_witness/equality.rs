//! Equal output correspondence and installed revisions across exact settlements.
use super::*;
impl SealedNativeOutputWitness {
    pub(in crate::domain_computation::primary_graph) fn same_output_as(
        &self,
        other: &Self,
    ) -> Result<bool, CompanionPreflightStop> {
        if self.roles.len() != other.roles.len() || self.aspects.len() != other.aspects.len() {
            return Ok(false);
        }
        for (left, right) in self.roles.iter().zip(&other.roles) {
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
