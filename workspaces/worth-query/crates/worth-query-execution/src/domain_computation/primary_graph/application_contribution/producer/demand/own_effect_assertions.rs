//! Courts over the demand's authentic retained receipt; no accessor or minted proof.
use super::own_effect_observation::{self, Stage};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_output_demand::{OutputRowStage, WorthQueryAcceptedOutputAuthority},
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema> + 'static,
    FamilySourceValue<Schema, Family>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                Schema,
                FamilySourceQuery<Schema, Family>,
            > + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
    WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
{
    pub(in crate::domain_computation::primary_graph) fn assert_born_stale_absence_for_test(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: worth_relational::facade::identity::EntityId,
        locator: &worth_foundational::facade::AspectFieldLocator,
        inspect: impl FnOnce(&WorthQueryApplicationCommitReceipt),
        required_disclosure: Option<(
            WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            &WorthQueryRequestScope,
        )>,
    ) {
        let stages = own_effect_observation::take();
        eprintln!("own-effect stages: {stages:?}");
        assert_eq!(
            stages,
            [
                Stage::Published,
                Stage::DeliveryStarted,
                Stage::Delivered,
                Stage::Ready,
                Stage::ReadySuperseded,
                Stage::OwnPublicationContinuation,
                Stage::DisclosureExhausted
            ],
            "identify the exact Pending branch before any successor advance"
        );
        assert!(
            !self.settled && self.settled_at_observation.is_none(),
            "Pending is not completion"
        );
        assert_eq!(self.producer_contacts_in_this_demand, 1);
        assert_eq!(
            runtime.producer_publications_at_root_on_this_thread_for_test(root),
            1
        );
        assert_eq!(
            runtime.producer_recorded_publications_at_root_on_this_thread_for_test(root),
            1
        );
        let interest = self.interest.as_ref().expect("original live Interest");
        let OutputRowStage::Ready(completion) = runtime.output_demands.row_stage(interest) else {
            panic!("the original row must retain its actual Ready, not an inferred commit");
        };
        let WorthQueryAcceptedOutputAuthority::Committed(receipt) = &completion.authority else {
            panic!("performed original receipt, never stable alias or reconstruction");
        };
        let completed = runtime
            .primary_provider
            .observe_completed_application(receipt.commit_reference())
            .unwrap();
        let facts = completed.commit_evidence().rebased_source_facts().unwrap();
        assert!(completed.commit_evidence().source_facts_superseded_by_own_effect().any(|ordinal|
            matches!(&facts[ordinal], Fact::OptionalAspectAbsent { entity_id, locator: read, .. }
                if *entity_id == root && read == locator)), "the original selected-field aspect absence really moved");
        let current = runtime
            .on_branch(receipt.product_branch())
            .select_with_request(execution)
            .unwrap();
        assert!(
            receipt
                .committed_product_publication()
                .is_selected_at(current.product().observation()),
            "own exact publication remains selected"
        );
        let refusal = current
            .require_current_output_receipts(phase, [receipt])
            .unwrap_err();
        assert_eq!(refusal.kind(), WorthQueryOutputDemandDenialKind::Superseded);
        inspect(receipt);
        if let Some((source, principal, scope)) = required_disclosure {
            self.assert_required_admission(runtime, phase, execution, principal, scope, source);
        }
    }
    pub(in crate::domain_computation::primary_graph) fn assert_preserve_successor_for_test(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        root: worth_relational::facade::identity::EntityId,
    ) {
        let stages = own_effect_observation::take();
        eprintln!("successor stages: {stages:?}");
        assert_eq!(stages.iter().filter(|s| **s == Stage::Published).count(), 1);
        assert_eq!(stages.last(), Some(&Stage::ReadySettled));
        assert!(!stages.contains(&Stage::DisclosureExhausted));
        assert_eq!(self.producer_contacts_in_this_demand, 2);
        assert_eq!(
            runtime.producer_publications_at_root_on_this_thread_for_test(root),
            2
        );
        assert_eq!(
            runtime.producer_recorded_publications_at_root_on_this_thread_for_test(root),
            2
        );
        assert!(self.settled);
    }
}
