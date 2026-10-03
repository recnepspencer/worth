//! Admit one real required successor from the existing Fresh disclosure.
//!
//! The disclosure stays intact for Schedule→Execute. A paid shallow source
//! copy moves to the registry's retained readmission owner. This child is
//! registered only with the selected execution continuation.

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::disclosure::FreshOutputDisclosure;
use super::super::disclosure::ValidatedOutputDisclosure;
use super::super::{PreparedRequiredFreshSlot, RequiredFreshOutcome, RequiredFreshProgress};
use super::admission::SourceAdmissionSelection;
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{
        DemandAdmissionKind, OutputRefreshPredecessor, SelectedRequiredRefreshClaim,
        WorthQueryAcceptedOutputAuthority,
    },
    application_query::WorthQueryObservedSourceCloneStop,
    product_operation::SharedSelectedProductOperation,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Continue one admitted required successor without losing its source
    /// disclosure or real Interest on a post-admission refusal.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn continue_required_fresh<
        Family,
    >(
        &self,
        fresh: FreshOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        shared: &SharedSelectedProductOperation<'_, Schema>,
        claim: SelectedRequiredRefreshClaim,
        installed: &super::super::super::registry::InstalledProducerProvider<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        matched_predecessor: Option<MatchedRequiredPredecessor<'_>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequiredFreshProgress<Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
        WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    {
        let mut slot = PreparedRequiredFreshSlot::<Schema, Family>::prepare(
            &self.output_demands,
            claim,
            installed,
            admission,
        )?;
        let mut successor =
            self.admit_required_fresh_successor::<Family>(&fresh, shared, slot.claim(), admission)?;
        // The mode was prepared from this exact claim before registration. It
        // must be on the real admitted demand before any scheduling or effect.
        slot.bind_admitted(&mut successor);
        let transfer = self.output_demands.finish_replaced_interest(
            slot.claim().interest(),
            successor
                .interest
                .as_ref()
                .expect("admitted required successor retains its issued Interest"),
            Family::IDENTITY,
            admission,
        );
        let outcome = match transfer {
            Ok(()) => {
                match u64::try_from(std::mem::size_of::<
                    super::super::super::WorthQueryProducerCommitAuthority,
                >())
                .ok()
                .and_then(|work| admission.charge_external_work(work).ok())
                {
                    Some(()) => {
                        let mode = slot.claim().commit_authority().clone();
                        self.advance_validated_required_fresh_on_selected(
                            &mut successor,
                            principal,
                            request_scope,
                            delivery_branch,
                            ValidatedOutputDisclosure::Fresh(fresh),
                            installed,
                            mode,
                            shared,
                            matched_predecessor,
                            admission,
                        )
                    }
                    None => Err(work_denial()),
                }
            }
            Err(denial) => Err(denial),
        };
        Ok(slot.finish(
            successor,
            match outcome {
                Ok(advance) => RequiredFreshOutcome::Advanced(advance),
                Err(denial) => RequiredFreshOutcome::Refused(denial),
            },
        ))
    }

    /// This returns a real admitted demand and its interest. The caller keeps
    /// the same Fresh proof and prepared claim slot until it has transferred
    /// the predecessor and advanced or refused that demand.
    pub(super) fn admit_required_fresh_successor<Family>(
        &self,
        fresh: &FreshOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        shared: &SharedSelectedProductOperation<'_, Schema>,
        claim: &SelectedRequiredRefreshClaim,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        preclaim_terminal_subject(admission)?;
        // Source accessor, selected Ready, program basis and authority are
        // inspected before the borrowed proof is copied or registration runs.
        admission
            .charge_external_work(4)
            .map_err(|_| work_denial())?;
        let retained_source = fresh
            .source()
            .clone_admitted(&mut |work, bytes| {
                admission.charge_external_work(work)?;
                admission.admit_read_scratch(bytes)
            })
            .map_err(observed_clone_denial)?;
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        // A Ready row can originate in ordinary admission without an older
        // retained program basis. Preserve its authentic optional custody;
        // the selected RequiredFresh path separately binds the current
        // source to `shared` and the claim carries the executed commit mode.
        let retained_basis = claim
            .selected()
            .retained_program_basis()
            .map(std::sync::Arc::clone);
        let predecessor = match &claim.selected().completion().authority {
            WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                OutputRefreshPredecessor::Committed(receipt)
            }
            WorthQueryAcceptedOutputAuthority::Stable(published) => {
                OutputRefreshPredecessor::Stable {
                    interest: claim.interest(),
                    published,
                }
            }
            WorthQueryAcceptedOutputAuthority::Restored(_) => {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "restored predecessor cannot mint required Fresh",
                ));
            }
        };
        self.admit_output_demand_with_source_admitted::<Family>(
            fresh.value(),
            retained_source,
            None,
            Family::profile_kind(fresh.value()),
            claim.selected().limits(),
            None,
            DemandAdmissionKind::Required,
            None,
            Some(predecessor),
            retained_basis,
            SourceAdmissionSelection::RequiredFresh { shared, claim },
            admission,
        )
    }
}

fn observed_clone_denial(
    stop: WorthQueryObservedSourceCloneStop<CompanionPreflightStop>,
) -> WorthQueryOutputDemandDenial {
    match stop {
        WorthQueryObservedSourceCloneStop::AccountingOverflow => work_denial(),
        WorthQueryObservedSourceCloneStop::Admission(
            CompanionPreflightStop::WorkExhausted { .. }
            | CompanionPreflightStop::WorkCounterOverflow,
        ) => work_denial(),
        WorthQueryObservedSourceCloneStop::Admission(_) => denial(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            "required observed source cannot be retained",
        ),
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "required Fresh source exceeds carried request work",
    )
}

fn preclaim_terminal_subject(
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    const SUBJECT_BYTES: usize = "required predecessor has no retained program basis".len();
    let empty_work = || {
        WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
    };
    let empty_capacity = || {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            "",
        )
    };
    admission
        .charge_external_work(u64::try_from(SUBJECT_BYTES).map_err(|_| empty_work())?)
        .map_err(|_| empty_work())?;
    let backing = SUBJECT_BYTES
        .checked_add(std::mem::size_of::<String>())
        .ok_or_else(empty_capacity)?;
    admission
        .admit_read_scratch(u64::try_from(backing).map_err(|_| empty_capacity())?)
        .map_err(|stop| match stop {
            CompanionPreflightStop::WorkExhausted { .. }
            | CompanionPreflightStop::WorkCounterOverflow => empty_work(),
            _ => empty_capacity(),
        })
}
