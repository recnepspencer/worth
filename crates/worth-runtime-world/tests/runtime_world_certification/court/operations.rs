use super::*;
use std::sync::{Arc, OnceLock};
impl CompositeSupplyChainCourt {
    pub fn prepare_cargo(
        &self,
        head: &ProductBranchObservation,
        amount: &str,
    ) -> PreparedCompositePublicationWithoutSignal {
        let candidate = self
            .records
            .candidate(head.basis().relational_basis(), "grain", amount);
        self.world
            .publication_port()
            .prepare_without_signal(
                head.clone(),
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate),
                &RuntimeWorldCancellationSource::new().token(),
                None,
            )
            .expect("World: prepare cargo")
    }
    pub fn publish_cargo(
        &self,
        head: &ProductBranchObservation,
        amount: &str,
    ) -> ConsumedCompositePublication {
        let prepared = self.prepare_cargo(head, amount);
        match self
            .world
            .publication_port()
            .execute_without_signal(prepared, &RuntimeWorldCancellationSource::new().token())
        {
            RuntimeWorldPublicationOutcome::Performed(done) => done.consume(),
            other => panic!("World: healthy cargo publication {other:?}"),
        }
    }
    pub fn context(&self, head: &ProductBranchObservation) -> RoutingContext {
        RoutingContext {
            records: Arc::new(OnceLock::from(
                self.records.read(head.basis().relational_basis()),
            )),
        }
    }
    pub fn partial_after_relational(
        &self,
        head: &ProductBranchObservation,
    ) -> ProductUnpublishedOwnerEffects {
        let candidate = self
            .records
            .candidate(head.basis().relational_basis(), "grain", "5");
        let token = RuntimeWorldCancellationSource::new().token();
        let port = self.world.publication_port();
        let intent =
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate);
        let prepared = port
            .prepare_with_signal(head.clone(), intent, &token, None)
            .unwrap();
        match port.execute_with_signal(prepared, &mut self.context(head), &token, |_| {
            Err(SignalError::InvalidInput {
                message: "court routing denial".into(),
                context: None,
            })
        }) {
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => effects,
            other => {
                panic!("World: settled Relational then denied Signal must remain partial {other:?}")
            }
        }
    }
}
