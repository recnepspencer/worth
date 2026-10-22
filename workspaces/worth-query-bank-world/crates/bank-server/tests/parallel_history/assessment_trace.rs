//! Each judged public phase contributes its owner report and application order.
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test as observations,
    WorthQueryAdvancementEventKindForTest as Kind,
    WorthQueryAdvancementIdentityForTest as Identity,
};

// Installed-operation seals use the installation's package authority key.
// The native seal/symbol mapping must be bijective inside this real history;
// canonical command keys/intents and every producer binding stay exact.
#[derive(Default)]
pub(super) struct Trace {
    operations: std::collections::BTreeMap<&'static str, [u8; 32]>,
}
enum SemanticIdentity {
    Mutation {
        operation: &'static str,
        key: [u8; 32],
        intent: [u8; 32],
    },
    Producer(Identity),
}
impl std::fmt::Debug for SemanticIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mutation {
                operation,
                key,
                intent,
            } => f
                .debug_struct("Mutation")
                .field("operation", operation)
                .field("key", key)
                .field("intent", intent)
                .finish(),
            Self::Producer(identity) => identity.fmt(f),
        }
    }
}
impl Trace {
    fn bind(&mut self, name: &'static str, identity: &Identity) -> SemanticIdentity {
        match identity {
            Identity::Mutation {
                operation,
                key,
                intent,
            } => {
                let symbol = match name {
                    "definition" | "propose" => "authoring",
                    "start" => "instance-start",
                    "accept" => "advance",
                    _ => panic!("no mutation declared in {name}"),
                };
                assert!(
                    !self
                        .operations
                        .iter()
                        .any(|(other, seal)| *other != symbol && seal == operation),
                    "distinct declared operations have distinct native seals"
                );
                if let Some(previous) = self.operations.insert(symbol, *operation) {
                    assert_eq!(
                        previous, *operation,
                        "one declared operation retains its exact native seal"
                    );
                }
                SemanticIdentity::Mutation {
                    operation: symbol,
                    key: *key,
                    intent: *intent,
                }
            }
            Identity::Producer { producer, .. } => {
                assert_eq!(
                    producer, "worth.bank.approved-payment-assessment.producer.v1",
                    "the assessment's declared producer owns its event"
                );
                SemanticIdentity::Producer(identity.clone())
            }
        }
    }
    pub(super) fn phase<R>(
        &mut self,
        name: &'static str,
        width: usize,
        action: impl FnOnce() -> R,
    ) -> R {
        observations();
        let result = action();
        let records = observations();
        // Settlement may cross real Progressed boundaries. Judge all of its
        // admitted scopes together; exactly one commit/publication pair still
        // belongs to the original producer. A settled repeat admits one scope.
        if name == "settle" {
            assert!(
                !records.is_empty(),
                "settlement admits its actual advancement scopes"
            );
        } else {
            assert_eq!(
                records.len(),
                if name == "retry" { 1 } else { 2 },
                "the public phase's admitted scopes are observed exactly"
            );
        }
        let expected_events =
            if ["definition", "start", "propose", "settle", "accept"].contains(&name) {
                2
            } else {
                0
            };
        assert_eq!(records.iter().map(|record|record.applied.len()).sum::<usize>(),expected_events,"each declared assessment write has one commit/publication pair; preparation and retry have none");
        for record in &records {
            for pair in record.applied.chunks(2) {
                assert_eq!(
                    pair.len(),
                    2,
                    "assessment application is a commit/publication pair"
                );
                assert_eq!(
                    (pair[0].kind, pair[1].kind),
                    (Kind::Commit, Kind::Publication)
                );
                assert_eq!(
                    pair[0].identity, pair[1].identity,
                    "assessment publication names its commit by value"
                );
            }
        }
        assert!(
            records.iter().all(|record| record.workers == Some(width)),
            "every public assessment preparation and execution scope has its named width"
        );
        let judged = records
            .iter()
            .map(|record| {
                (
                    record.report.charged_work(),
                    record.report.charged_span(),
                    record.refusal,
                    record
                        .applied
                        .iter()
                        .map(|event| (event.kind, self.bind(name, &event.identity)))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        println!("BANK_ASSESSMENT:{name}:{judged:?}");
        result
    }
}
