//! Genuine selected Ready claim and required successor admission, without effects.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::{
    disclosure::{validate_disclosure, validate_required_successor, ValidatedOutputDisclosure},
    PreparedRequiredFreshSlot,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
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
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn assert_required_admission(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        source: WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
    ) {
        let mut admission = runtime.demand_request_admission();
        let original = self.installed_entry.as_ref();
        let ValidatedOutputDisclosure::Fresh(fresh) = validate_disclosure(
            execution,
            runtime,
            self,
            principal,
            scope,
            runtime.current_world(),
            false,
            source,
            original.edition,
        )
        .unwrap() else {
            panic!("actual current authenticated disclosure");
        };
        let ready = runtime
            .output_demands
            .interest_ready_readmission(self.interest.as_ref().unwrap(), &mut admission)
            .unwrap()
            .expect("the actual required Interest pins its exact Ready/source/member");
        assert!(ready
            .matches_interest(self.interest.as_ref().unwrap(), &mut admission)
            .unwrap());
        let claim = runtime
            .output_demands
            .claim_required_ready_refresh(&ready, &mut admission)
            .unwrap()
            .expect("authentic current Ready claim");
        let mut slot = PreparedRequiredFreshSlot::<Schema, Family>::prepare(
            &runtime.output_demands,
            claim,
            original,
            &mut admission,
        )
        .unwrap();
        let current = runtime
            .on_branch(runtime.current_world())
            .select(phase)
            .unwrap();
        let shared = current
            .prepare_shared_query_basis(&mut admission)
            .unwrap_or_else(|(_, stop)| {
                panic!("actual selected basis admission refused: {stop:?}")
            });
        let mut successor = runtime
            .admit_required_fresh_successor::<Family>(
                execution,
                &fresh,
                &shared,
                slot.claim(),
                &mut admission,
            )
            .unwrap();
        slot.bind_admitted(&mut successor);
        runtime
            .output_demands
            .finish_replaced_interest(
                slot.claim().interest(),
                successor.interest.as_ref().unwrap(),
                Family::IDENTITY,
                &mut admission,
            )
            .unwrap();
        assert_eq!(successor.selected.applicability.lifecycle(),
            crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture::Preserve);
        assert_ne!(original.edition, successor.installed_entry.edition);
        let fresh =
            validate_required_successor(fresh, slot.claim(), original, &successor, &mut admission)
                .unwrap();
        successor
            .progression_provenance
            .validate_for_execution(
                slot.claim().commit_authority(),
                &successor.installed_entry.edition,
                &mut admission,
            )
            .unwrap();
        successor
            .prepare_disclosed_input(ValidatedOutputDisclosure::Fresh(fresh), &mut admission)
            .unwrap()
            .assert_transport_guards(original.edition);
        assert_eq!(self.producer_contacts_in_this_demand, 1);
        let root = self.observed_source.source_root();
        assert_eq!(
            runtime.producer_publications_at_root_on_this_thread_for_test(root),
            1
        );
        assert_eq!(
            runtime.producer_recorded_publications_at_root_on_this_thread_for_test(root),
            1
        );
        // No execution or publication is claimed. The real slot/demand Drop owners release custody.
        drop(successor);
        drop(slot);
    }
}
