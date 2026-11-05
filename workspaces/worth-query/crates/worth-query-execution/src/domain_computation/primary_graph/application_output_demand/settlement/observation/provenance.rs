//! The retained read must describe the settlement's actual immutable owner.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputSettlementProvenanceDenial as Reason;
use {crate::domain_computation::vocabulary::primary_graph::admitted_capture::checkpoint, crate::domain_computation::primary_graph::output_reuse::read_denial};
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop, WorkCeilingDenial};
use crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure;

impl OriginalSettledProducerObservation {
    pub(in crate::domain_computation::primary_graph::application_output_demand) fn require_settlement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        settled: &WorthQueryOutputDemandSettlement,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut outcome = None;
        let handoff = execution.run_native_with_checkpoint(|context| {
            outcome = Some(self.compare_settlement(context, settled));
        });
        match (handoff, outcome) {
            (Ok(_), Some(result)) => result,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(read_denial(cause.into())),
            (Ok(_), None) => unreachable!("successful provenance stage invokes its comparison"),
        }
    }

    /// Original settlement/disclosure owners compare by retained Arc identity.
    /// Read names and restored producer/role/fact bytes are charged before
    /// semantic comparison; no lineage or unrelated producer is traversed.
    fn compare_settlement(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        settled: &WorthQueryOutputDemandSettlement,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        pay(context, 8)?;
        pay_width(context, self.read.branch_identity().name().as_str().len())?;
        pay_width(
            context,
            settled
                .observation
                .product_observation()
                .branch_identity()
                .name()
                .as_str()
                .len(),
        )?;
        if self.runtime_authority != settled.runtime_authority
            || self.schema != settled.schema_binding
            || !self
                .read
                .matches_observation(settled.observation.product_observation())
            || !Arc::ptr_eq(&self.correspondence, &settled.output_correspondence)
        {
            return Err(foreign(
                "settlement read binding disagrees with its original producer",
            ));
        }
        match &self.source {
            OriginalSettledProducerSource::Committed {
                settlement,
                publication,
                disclosure,
                ..
            } => {
                let receipt = settled
                    .application_commit_receipt()
                    .ok_or_else(|| foreign("committed provenance has no retained receipt"))?;
                if settled.stable.is_some() || settled.restored_source.is_some() {
                    return Err(foreign(
                        "committed provenance has a foreign settlement posture",
                    ));
                }
                require_owner(
                    settlement.as_ref(),
                    receipt.exact_output_settlement(),
                    Reason::OriginalSettlementMissing,
                    Reason::ReceiptSettlementMissing,
                    Reason::ForeignSettlementOwner,
                    "original committed settlement differs from its receipt",
                )?;
                let expected_publication = receipt
                    .committed_product_publication()
                    .observed_product_identity();
                for identity in publication.iter().chain(expected_publication.iter()) {
                    pay_width(context, identity.branch_identity().name().as_str().len())?;
                }
                if *publication != expected_publication {
                    return Err(refused(
                        Reason::PublicationMismatch,
                        "original producer publication differs from its receipt",
                    ));
                }
                let expected_disclosure = receipt
                    .authority_binding()
                    .retain_original_inspection_disclosure();
                require_disclosure_owner(disclosure.as_ref(), expected_disclosure.as_ref())?;
            }
            OriginalSettledProducerSource::StableReused { settlement } => {
                let stable = settled.stable.as_ref().ok_or_else(|| {
                    foreign("reused provenance has no retained stable publication")
                })?;
                if settled.application_commit_receipt().is_some()
                    || settled.restored_source.is_some()
                {
                    return Err(foreign(
                        "reused provenance has a foreign settlement posture",
                    ));
                }
                if !Arc::ptr_eq(settlement, stable.exact_settlement()) {
                    return Err(refused(
                        Reason::ReusedSettlementOwnerMismatch,
                        "original reused settlement differs from its stable publication",
                    ));
                }
            }
            OriginalSettledProducerSource::Restored { checkpoint } => {
                let restored = settled.restored_source.as_ref().ok_or_else(|| {
                    foreign("restored provenance has no retained checkpoint owner")
                })?;
                pay_checkpoint(context, checkpoint)?;
                pay_checkpoint(context, &restored.checkpoint)?;
                if settled.application_commit_receipt().is_some() || settled.stable.is_some() {
                    return Err(foreign(
                        "restored provenance has a foreign settlement posture",
                    ));
                }
                if checkpoint != &restored.checkpoint {
                    return Err(refused(
                        Reason::RestoredCheckpointMismatch,
                        "original restored checkpoint differs from its retained owner",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn pay_checkpoint(
    context: &mut MapKernelContext<'_, '_>,
    identity: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
) -> Result<(), WorthQueryOutputDemandDenial> {
    pay_width(context, identity.producer.len())?;
    for role in identity.roles.iter() {
        pay(context, 1)?;
        pay_width(context, role.role.len())?;
        pay_width(context, role.entity_name.len())?;
    }
    if let Some(facts) = &identity.producer_facts {
        pay_width(context, facts.len())?;
    }
    Ok(())
}

fn pay_width(
    context: &mut MapKernelContext<'_, '_>,
    width: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let width = u64::try_from(width).map_err(|_| {
        read_denial(WorkCeilingDenial::Stopped(MapKernelStop::WorkCounterOverflow).into())
    })?;
    pay(context, width)
}

fn pay(
    context: &mut MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), WorthQueryOutputDemandDenial> {
    checkpoint(context, work).map_err(|cause| read_denial(WorkCeilingDenial::Stopped(cause).into()))
}

fn foreign(subject: &'static str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSettlement, subject)
}

fn refused(reason: Reason, subject: &'static str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::SettlementProvenance(reason),
        subject,
    )
}

fn require_owner<T>(
    original: Option<&Arc<T>>,
    expected: Option<&Arc<T>>,
    missing_original: Reason,
    missing_receipt: Reason,
    foreign: Reason,
    subject: &'static str,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match (original, expected) {
        (Some(original), Some(expected)) => {
            if Arc::ptr_eq(original, expected) {
                Ok(())
            } else {
                Err(refused(foreign, subject))
            }
        }
        (None, Some(_)) => Err(refused(missing_original, subject)),
        (Some(_), None) => Err(refused(missing_receipt, subject)),
        (None, None) => Ok(()),
    }
}

pub fn require_disclosure_owner(
    original: Option<&Arc<OriginalSourceInspectionDisclosure>>,
    expected: Option<&Arc<OriginalSourceInspectionDisclosure>>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    require_owner(
        original,
        expected,
        Reason::OriginalDisclosureMissing,
        Reason::ReceiptDisclosureMissing,
        Reason::ForeignDisclosureOwner,
        "original producer disclosure differs from its receipt",
    )
}
