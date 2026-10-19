//! Request-local executable input sealing after actual producer selection.
use std::any::Any;

use super::super::super::registry::InstalledProducerEdition;
use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

/// Owns the admitted source until the selected executor borrows it. Its edition
/// is issued once from the final demand, independently of source-admission provenance.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PreparedProducerInput<
    Query,
    Value,
> {
    edition: InstalledProducerEdition,
    disclosure: ValidatedOutputDisclosure<Query, Value>,
}

/// Borrowed, single-consumption transport inspected before runtime erasure opens.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct ValidatedProducerInput<
    'a,
> {
    edition: InstalledProducerEdition,
    proof: &'a mut dyn Any,
}

impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    /// O(source identity + installed identity width), no payload clone or allocation.
    /// Only the actual admitted demand selects the seal's installed edition.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn prepare_disclosed_input(
        &self,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        PreparedProducerInput<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
        WorthQueryOutputDemandDenial,
    > {
        let entry = self.installed_entry.as_ref();
        admission
            .charge_external_work(12 + 4 * 32)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        let identity_work = entry
            .declaration
            .identity
            .len()
            .checked_add(self.selected.identity.len())
            .and_then(|n| n.checked_add(entry.declaration.output_family.len()))
            .and_then(|n| n.checked_add(Family::IDENTITY.len()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        admission
            .charge_external_work(identity_work)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        // Price each reached applicability comparison before consulting its
        // variable-width profile. This is the original admission's meter.
        let mut admitted_applicability = false;
        for applicability in &entry.declaration.applicability {
            admission
                .charge_external_work(3)
                .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
            let work = applicability
                .profile_kind()
                .len()
                .checked_add(self.selected.applicability.profile_kind().len())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
            admission
                .charge_external_work(work)
                .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
            if *applicability == self.selected.applicability {
                admitted_applicability = true;
                break;
            }
        }
        let source = disclosure.source();
        let same_admitted_source = self.matches_observed_source_admitted(source, admission)?;
        if !same_admitted_source
            || source.runtime_authority != self.runtime_authority
            || source.schema_binding != self.schema_binding
            || entry.declaration.source_type != TypeId::of::<Family::Source>()
            || entry.declaration.output_family_type != TypeId::of::<Family>()
            || entry.declaration.output_family != Family::IDENTITY
            || entry.declaration.identity != self.selected.identity
            || !admitted_applicability
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            ));
        }
        Ok(PreparedProducerInput {
            edition: entry.edition,
            disclosure,
        })
    }
}

impl<Query: 'static, Value: 'static> PreparedProducerInput<Query, Value> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn with_erased<
        Result,
    >(
        self,
        execute: impl FnOnce(ValidatedProducerInput<'_>) -> Result,
    ) -> Result {
        let mut proof = Some(self.disclosure);
        execute(ValidatedProducerInput {
            edition: self.edition,
            proof: &mut proof,
        })
    }
}

impl ValidatedProducerInput<'_> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn take<
        Query: 'static,
        Value: 'static,
    >(
        self,
        installed: InstalledProducerEdition,
    ) -> Option<ValidatedOutputDisclosure<Query, Value>> {
        if self.edition != installed {
            return None;
        }
        self.proof
            .downcast_mut::<Option<ValidatedOutputDisclosure<Query, Value>>>()?
            .take()
    }
}

#[cfg(test)]
#[path = "producer_input/transport_court.rs"]
mod transport_court;
