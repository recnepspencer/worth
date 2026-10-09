//! One selected-source authority for required Fresh admission.
//!
//! The selected Product and exact Ready claim travel together. A different
//! producer must be the installed family's declared Preserve successor.

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    application_output_demand::SelectedRequiredRefreshClaim,
    output_lineage::invalidation::InvalidationEditAdmission,
    product_operation::SharedSelectedProductOperation,
};

use super::super::{denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
enum SourceAdmissionSelection<'selected, 'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    Ordinary,
    RequiredFresh {
        shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
        claim: &'selected SelectedRequiredRefreshClaim,
    },
}

impl<Schema> SourceAdmissionSelection<'_, '_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn selected_product(&self) -> Option<&SharedSelectedProductOperation<'_, Schema>> {
        match self {
            Self::Ordinary => None,
            Self::RequiredFresh { shared, .. } => Some(shared),
        }
    }

    /// This check is reached after the existing selector returns and before
    /// the registry creates a successor interest. A fresh required output is
    /// remains tied to its exact predecessor's compiled output family.
    pub(super) fn admit_selected_producer<Family>(
        &self,
        selected: &super::super::super::WorthQuerySelectedApplicationProducer,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: super::super::WorthQueryProducerOutputFamily<Schema>,
    {
        const SUBJECT: &str = "required successor selected another producer";
        let empty_work = || denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "");
        let empty_capacity = || {
            denial(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "",
            )
        };
        admission
            .charge_external_work(1)
            .map_err(|_| empty_work())?;
        let Self::RequiredFresh { claim, .. } = self else {
            return Ok(());
        };
        // Prepay both text widths and the possible terminal subject before
        // borrowing the predecessor's producer identity for equality.
        admission
            .charge_external_work(2)
            .map_err(|_| empty_work())?;
        let expected = claim.selected().producer_identity();
        let producer = &selected.identity;
        let comparison_work = producer
            .len()
            .checked_add(expected.len())
            .and_then(|bytes| bytes.checked_add(SUBJECT.len()))
            .ok_or_else(empty_work)?;
        admission
            .charge_external_work(u64::try_from(comparison_work).map_err(|_| empty_work())?)
            .map_err(|_| empty_work())?;
        let denial_backing = SUBJECT
            .len()
            .checked_add(std::mem::size_of::<String>())
            .ok_or_else(empty_capacity)?;
        admission
            .admit_read_scratch(u64::try_from(denial_backing).map_err(|_| empty_capacity())?)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    empty_work()
                }
                _ => empty_capacity(),
            })?;
        let same_family = claim.selected().key().family_type() == std::any::TypeId::of::<Family>();
        let preserve = selected.applicability.lifecycle()
            == crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerLifecyclePosture::Preserve;
        let same_profile = selected.applicability.profile_kind()
            == claim.selected().key().applicability().profile_kind();
        let initial_predecessor = claim.selected().key().applicability().lifecycle()
            == crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerLifecyclePosture::Initial;
        if !same_family
            || (producer != expected && (!preserve || !same_profile || !initial_predecessor))
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                SUBJECT,
            ));
        }
        Ok(())
    }
}
