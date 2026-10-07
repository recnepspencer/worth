//! Native content attributed to one sealed consumed output, without currentness.
use super::*;

impl SealedNativeOutputWitness {
    /// Reading a consumed entity's installed content does not make it an
    /// independent consumer fact. Its consumed edge still requires currentness.
    pub(in crate::domain_computation::primary_graph) fn covers_content_fact(
        &self,
        fact: &Fact,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        let (entity, aspect) = match fact {
            Fact::SourceEntity { entity_id } | Fact::Entity { entity_id, .. } => (*entity_id, None),
            Fact::SourceAspectRevision {
                entity_id, aspect, ..
            } => (*entity_id, Some(aspect)),
            Fact::SourceFieldRevision {
                entity_id, locator, ..
            }
            | Fact::Field {
                entity_id, locator, ..
            }
            | Fact::AbsentField {
                entity_id, locator, ..
            } => (*entity_id, Some(locator.aspect().aspect_key())),
            _ => return Ok(false),
        };
        for role in &self.roles {
            admission.charge_external_work(1)?;
            if role.entity != Some(entity) {
                continue;
            }
            let Some(aspect) = aspect else {
                return Ok(true);
            };
            for owned in &self.aspects[role.first_aspect..role.end_aspect] {
                admission.charge_external_work(1 + aspect.as_str().len() as u64)?;
                if &owned.aspect == aspect {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
