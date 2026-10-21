//! One selected-source authority for required Fresh admission.
//!
//! The selected Product and exact Ready claim travel together. A different
//! producer must be the installed family's declared Preserve successor.

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredRefreshClaim;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;

use super::super::denial;
use super::super::WorthQueryOutputDemandDenial;
use super::super::WorthQueryOutputDemandDenialKind;

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
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: super::super::WorthQueryProducerOutputFamily<Schema>,
    {
        const SUBJECT: &str = "required successor selected another producer";

        let Self::RequiredFresh { claim, .. } = self else {
            return Ok(());
        };
        // Prepay both text widths and the possible terminal subject before
        // borrowing the predecessor's producer identity for equality.

        let expected = claim.selected().producer_identity();
        let producer = &selected.identity;

        let same_family = claim.selected().key().family_identity() == Family::IDENTITY;
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
