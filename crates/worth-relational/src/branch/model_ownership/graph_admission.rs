//! A descriptive graph may write only under the actual installed ownership meaning.
use super::*;
use worth_execution::MapKernelContext;
impl RelationalModelOwnership {
    pub(super) fn retention_bytes(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<u64, RelationalOwnedModelSelectionDenial> {
        crate::execution::request_stage::checkpoint(context, self.plan.domains.len() as u64 + 1)
            .map_err(|stop| {
                RelationalOwnedModelSelectionDenial::Ownership(
                    RelationalModelOwnershipDenial::Request(
                        worth_execution::WorkCeilingDenial::Stopped(stop),
                    ),
                )
            })?;
        Ok(self.plan.domains.iter().fold(
            128u64.saturating_add((self.plan.relations.len() as u64).saturating_mul(8)),
            |bytes, domain| {
                bytes.saturating_add(64).saturating_add(
                    (domain.entities.len() as u64 + domain.relations.len() as u64)
                        .saturating_mul(8),
                )
            },
        ))
    }
}
impl RelationalOwnedModelGraphSelection {
    pub(crate) fn ownership_matches_installation(
        &self,
        installed: Option<&RelationalModelOwnership>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<bool, worth_execution::MapKernelStop> {
        crate::execution::request_stage::checkpoint(context, 1)?;
        let Some(installed) = installed else {
            return Ok(false);
        };
        let size = |meaning: &RelationalModelOwnership| {
            meaning
                .plan
                .domains
                .iter()
                .fold(meaning.plan.relations.len() as u64 + 1, |count, d| {
                    count
                        .saturating_add(d.entities.len() as u64)
                        .saturating_add(d.relations.len() as u64)
                        .saturating_add(1)
                })
        };
        crate::execution::request_stage::checkpoint(
            context,
            installed.plan.domains.len() as u64
                + self.ownership.plan.domains.len() as u64
                + self.installed_ownership.plan.domains.len() as u64
                + 1,
        )?;
        let work = size(installed)
            .saturating_add(size(&self.installed_ownership))
            .saturating_add(size(&self.ownership));
        crate::execution::request_stage::checkpoint(
            context,
            work.saturating_mul(self.ownership.plan.domains.len() as u64 + 1),
        )?;
        if installed == &self.installed_ownership
            && !self.ownership.plan.domains.is_empty()
            && self
                .ownership
                .plan
                .domains
                .iter()
                .all(|domain| installed.plan.domains.binary_search(domain).is_ok())
        {
            return Ok(true);
        }
        Ok(false)
    }
}
