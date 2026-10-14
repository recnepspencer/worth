//! One real owning transfer attempt and explicit retained replay feed the independent journal model.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
    WorthQuerySourceOnlyDenial as Denial,
};
impl Observer<'_> {
    pub fn transfer(&mut self, input: &Transfer, posture: Posture) -> Result<Outcome, String> {
        use worth_query_host::facade::primary_graph::{
            advancement_observations_on_this_thread_for_test as observations,
            place_managed_computations_on_this_thread_for_test as place,
            WorthQueryExecutionPlacementForTest as Placement,
        };
        let placement = match posture {
            Posture::Serial => Placement::World,
            Posture::Leased(width) => Placement::Leased(width),
        };
        let previous = place(placement);
        observations();
        let actor = self.world.authenticate(input.actor);
        let recipient = if input.to == 12 { 2 } else { 1 };
        let scope = crate::support::request_scope();
        let request = self.world.world.runtime.request(&actor, &scope);
        let result = self
            .world
            .world
            .runtime
            .mutate(mutations::send_money(SendMoney {
                from: AccountId::new(input.from).unwrap(),
                recipient: principal(recipient),
                amount: Money::from_minor(input.amount).unwrap(),
            }))
            .as_principal(&actor)
            .controls(BankMutationControls::new(scope.clone(), key(input.command)))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("transfer construction: {cause:?}"));
        let result = result.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap());
        self.last = observations();
        let expected_width = match posture {
            Posture::Serial => 1,
            Posture::Leased(width) => width.get(),
        };
        assert!(
            self.last.iter().any(|r| r.workers == Some(expected_width)),
            "the Bank request dispatch reaches its named width"
        );
        assert!(
            self.last.iter().all(|r| r.workers == Some(expected_width)),
            "Bank dispatches preserve the requested width"
        );
        place(previous);
        let events = self
            .last
            .iter()
            .flat_map(|record| &record.applied)
            .collect::<Vec<_>>();
        let expected_events = if matches!(&result, Advance::Completed(_)) {
            2
        } else {
            0
        };
        assert_eq!(
            events.len(),
            expected_events,
            "a new Bank commit has two events; retry and denial have none"
        );
        match result {
            Advance::Completed(completed) => {
                let Completion::Fresh { result, receipt, .. } = completed.evidence() else { panic!("fresh initial completion") };
                use worth_query_host::facade::primary_graph::WorthQueryAdvancementIdentityForTest as Identity;
                let binding = receipt.idempotency_binding();
                let expected = Identity::Mutation { operation: *receipt.installed_operation(), key: *binding.key_identity(), intent: *binding.intent_identity() };
                assert!(self.last.iter().flat_map(|r| &r.applied).all(|event| event.identity == expected), "actual commit event identity");
                self.receipts.insert(input.command, receipt.clone());
                let outcome = self.bindings.committed(input.command, result.journal);
                assert!(completed.close(&request).is_ok());
                Ok(outcome)
            }
            Advance::Progress(operation) if operation.posture() == worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture::RetainedDeliveryPending => {
                let Advance::Completed(completed) = operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap()) else { panic!("original native replay delivery") };
                let Completion::Recovered { receipt, .. } = completed.evidence() else { panic!("replay evidence") };
                assert!(receipt.is_same_authoritative_commit(&self.receipts[&input.command]));
                let journal = self.bindings.commands[&input.command];
                let outcome = Outcome::Committed(self.bindings.journals[&journal]);
                assert!(completed.close(&request).is_ok());
                Ok(outcome)
            }
            Advance::Terminal(terminal) => {
                let insufficient = matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::DomainDenied(BankProposalDenial::InsufficientFunds(_))));
                assert!(terminal.close(&request).is_ok());
                if insufficient { Ok(Outcome::Insufficient) } else { Err("unexpected native terminal".into()) }
            }
            Advance::Refused { operation, denial: Denial::Mutation(WorthQueryApplicationRequestMutationDenial::Authorization(denial)) } => {
                assert_eq!(denial.kind(), WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied));
                let _runtime = operation.into_runtime_for_cleanup();
                Ok(Outcome::Unauthorized)
            }
            Advance::Refused { operation, denial } => {
                let _runtime = operation.into_runtime_for_cleanup();
                Err(format!("native transfer refusal: {denial:?}"))
            }
            Advance::Progress(operation) => {
                let posture = format!("unexpected native progress: {:?}", operation.posture());
                let _runtime = operation.into_runtime_for_cleanup();
                Err(posture)
            }
        }
    }
}
