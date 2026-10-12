macro_rules! advance_recording {
    ($stops:expr, $demand:expr, $request:expr, $observations:expr, $application:expr, $roots:expr) => {{
        let before_members: Vec<_> = $roots
            .iter()
            .map(|root| $application.producer_contacts_at_root_on_this_thread_for_test(*root))
            .collect();
        let before_publications: Vec<_> = $roots
            .iter()
            .map(|root| $application.producer_publications_at_root_on_this_thread_for_test(*root))
            .collect();
        let before_attempts = $application.producer_contacts_on_this_thread_for_test();
        let before_decisions = binding::decisions_snapshot().len();
        // Only the observing journey performs these extra observations.
        let before = (!$roots.is_empty()).then(|| {
            $application
                .with_host_advancement(|phase| {
                    $request
                        .retain_read(&phase)
                        .unwrap()
                        .selected_commit()
                        .clone()
                })
                .expect("fixture observation owns its host call")
        });
        let advanced = $demand.advance(&$request);
        bounded!();
        match advanced {
            Ok(progress) => Some(progress),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                if $stops.is_empty()
                    && denial.kind() != WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                {
                    return Err(Attempt::Above("the first stop is not the index's refusal"));
                }
                $observations.push(AdvanceObservation {
                    member_attempts: $roots
                        .iter()
                        .zip(before_members)
                        .map(|(root, before)| {
                            $application.producer_contacts_at_root_on_this_thread_for_test(*root)
                                - before
                        })
                        .collect(),
                    member_publications: $roots
                        .iter()
                        .zip(before_publications)
                        .map(|(root, before)| {
                            $application
                                .producer_publications_at_root_on_this_thread_for_test(*root)
                                - before
                        })
                        .collect(),
                    producer_attempts: $application.producer_contacts_on_this_thread_for_test()
                        - before_attempts,
                    decisions: binding::decisions_snapshot()
                        .into_iter()
                        .skip(before_decisions)
                        .map(|(scope, _)| scope)
                        .collect(),
                    published: before.as_ref().is_some_and(|before| {
                        $application
                            .with_host_advancement(|phase| {
                                $request.retain_read(&phase).unwrap().selected_commit() != before
                            })
                            .expect("fixture observation owns its host call")
                    }),
                });
                $stops.push((denial.kind(), denial.recovery_posture()));
                None
            }
            Err(other) => panic!("the advance stops as a demand: {other:?}"),
        }
    }};
}
pub(super) use advance_recording;
