//! The optional coordinate tail supplements the unchanged physical carrier fields.
use super::{locator, AggregateRecord};
use crate::aggregate_identity::funding::{AggregateWork, BuildResult, PlanDenial};
use crate::aggregate_identity::projected_storage::{
    encode_domain, COORDINATE_ASPECT, COORDINATE_FIELDS,
};
use crate::graph_member::NativeMemberKey as Member;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

impl AggregateRecord {
    /// Physical rows retain exactly thirteen fields. A projected row writes its
    /// real domain and seed/current cases in the conditional reserved aspect.
    pub(in crate::aggregate_identity) fn fields(
        &self,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<BTreeMap<AspectFieldLocator, AspectValue>> {
        let mut fields = self.base_fields();
        let seed = match &self.seed {
            Member::Entity(_) => {
                if self
                    .current_anchor
                    .as_ref()
                    .is_some_and(|key| key.entity().is_none())
                {
                    return work.invalid(PlanDenial::InvalidStoredIdentity);
                }
                return Ok(fields);
            }
            Member::Projected(seed) => seed,
        };
        let current = match self.current_anchor.as_ref().unwrap_or(&self.seed) {
            Member::Projected(current) if current.domain == seed.domain => current,
            _ => return work.invalid(PlanDenial::InvalidStoredIdentity),
        };
        work.retain(crate::aggregate_identity::retention_extent::scalar_fields(
            COORDINATE_FIELDS.len(),
        ))?;
        let domain = encode_domain(&seed.domain, work)?;
        let bytes = seed
            .case
            .len()
            .checked_add(current.case.len())
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
        work.checkpoint(bytes as u64)?;
        work.retain(bytes as u64)?;
        for (name, value) in [
            ("domain", domain),
            ("seed-case", seed.case.clone()),
            ("current-case", current.case.clone()),
        ] {
            fields.insert(
                locator(COORDINATE_ASPECT, name),
                AspectValue::String(value.into()),
            );
        }
        Ok(fields)
    }
}
