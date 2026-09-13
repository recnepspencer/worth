use super::super::routing::{INPUT, ROUTED};
use super::driver::ModelRun;
use super::*;
use std::sync::{Arc, OnceLock};

impl ModelRun {
    pub fn publish(&mut self, amount: Option<u64>, signal: bool) {
        self.steps.push(format!("publish {amount:?}/{signal}"));
        let head = self.heads["work"].clone();
        self.model.publish("work", amount, signal);
        let port = self.court.world.publication_port();
        let token = RuntimeWorldCancellationSource::new().token();
        let candidate = amount.map(|amount| {
            self.court.records.candidate(
                head.basis().relational_basis(),
                "grain",
                &amount.to_string(),
            )
        });
        let outcome = if signal {
            let mut intent = CompositePublicationIntent::with_signal(
                amount.map(|_| RelationalTransactionIntent::ordinary()),
            );
            if let Some(candidate) = candidate {
                intent = intent.with_prepared_relational_candidate(candidate);
            }
            let prepared = port
                .prepare_with_signal(head.clone(), intent, &token, None)
                .unwrap();
            let records = Arc::new(OnceLock::new());
            let mut context = RoutingContext {
                records: Arc::clone(&records),
            };
            let runtime = &self.court.records.runtime;
            let branch = head.basis().relational_basis().identity().clone();
            let nodes = self.court.nodes;
            let expected = self.model.commits[&self.model.branches["work"].head]
                .cargo
                .route();
            port.execute_with_signal(prepared, &mut context, &token, |tx| {
                // This isolated sequential branch has exactly one writer. Read
                // the real settled Relational occurrence, never a predicted basis.
                let basis = runtime.observe_branch(&branch).unwrap().1;
                records.set(self.court.records.read(&basis)).unwrap();
                tx.mark_changed(nodes.source, INPUT)?;
                let routed_version = tx.read(nodes.route, &|view| nodes.evaluate(view))?;
                assert_eq!(
                    routed_version.get(ROUTED),
                    expected,
                    "independent routed cargo"
                );
                Ok(())
            })
        } else {
            let intent =
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate.unwrap());
            let prepared = port
                .prepare_without_signal(head.clone(), intent, &token, None)
                .unwrap();
            port.execute_without_signal(prepared, &token)
        };
        let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
            panic!("pure model predicts successful publication");
        };
        drop(done.consume());
        self.heads.insert("work".into(), self.court.observe(&head));
        drop(head);
        self.check();
    }
    pub fn cancel(&mut self) {
        self.steps.push("prepare/cancel".into());
        self.model.prepare(0, "work");
        let prepared = self.court.prepare_cargo(&self.heads["work"], "3");
        self.check();
        let source = RuntimeWorldCancellationSource::new();
        source.cancel();
        self.model.finish_attempt(0);
        let outcome = self
            .court
            .world
            .publication_port()
            .execute_without_signal(prepared, &source.token());
        assert!(
            matches!(outcome, RuntimeWorldPublicationOutcome::NoEffect(no) if no.cause() == NoEffectCause::CancelledBeforeEffect)
        );
        self.check();
    }
    pub fn stale(&mut self) {
        self.steps.push("reserve stale contender".into());
        self.model.prepare(0, "work");
        let prepared = self.court.prepare_cargo(&self.heads["work"], "2");
        self.check();
        self.publish(Some(7), false);
        self.steps.push("execute stale contender".into());
        self.model.finish_attempt(0);
        let outcome = self
            .court
            .world
            .publication_port()
            .execute_without_signal(prepared, &RuntimeWorldCancellationSource::new().token());
        assert!(
            matches!(outcome, RuntimeWorldPublicationOutcome::NoEffect(no) if no.cause() == NoEffectCause::StaleExpectedProductHead)
        );
        self.check();
    }
    pub fn partial_cleanup(&mut self) {
        self.steps
            .push("Relational effect with Signal denial".into());
        self.model.prepare(0, "work");
        self.model.partial(0, "work");
        let effects = self.court.partial_after_relational(&self.heads["work"]);
        let handle = effects.recovery_handle();
        let keys = [
            RuntimeWorldRetentionKey::relational(effects.successor_basis().unwrap()),
            RuntimeWorldRetentionKey::signal(effects.successor_basis().unwrap()),
        ];
        assert_eq!(effects.owner_effect_count(), 1);
        assert_eq!(
            self.court
                .records
                .read(effects.successor_basis().unwrap().relational_basis())
                .records["grain"],
            "5"
        );
        self.observer.check_unpublished(
            &self.court,
            &self.model,
            self.model.partials[&0],
            &effects,
        );
        self.check();
        self.steps.push("cleanup-only continuation".into());
        self.court
            .world
            .recovery_port()
            .continue_effects(effects)
            .unwrap();
        self.check();
        self.steps.push("release partial".into());
        let unpublished = self.model.cleanup(0);
        self.court
            .world
            .recovery_port()
            .release_effects(&handle, 0)
            .unwrap();
        self.check();
        self.steps.push("reclaim unpublished successor".into());
        assert!(self.model.reclaim(unpublished));
        let identity = self.observer.commit(unpublished);
        let reclaimed = self
            .court
            .world
            .lifecycle_port()
            .reclaim_history(CompositeHistoryReclamationRequest::new(
                self.court.world.owner_identity(),
                vec![identity.clone()],
                1,
            ))
            .unwrap();
        assert_eq!(reclaimed.reclaimed_commits(), &[identity]);
        self.court
            .world
            .lifecycle_port()
            .reclaim_retention(&keys, 2)
            .unwrap();
        self.check();
    }
}
