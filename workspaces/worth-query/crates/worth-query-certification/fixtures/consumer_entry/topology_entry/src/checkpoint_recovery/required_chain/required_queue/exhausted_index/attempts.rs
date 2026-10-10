//! Real refused advances count every chain member at the handler boundary.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryApplicationProducerProvider;

#[test]
fn a_refused_consumer_registration_performs_each_chain_member_once() {
    let _guard = checkpoint_recovery_test_guard();
    // A and D are roots; B and C each make one authoritative chain decision.
    let chain = CHAIN;
    let consumers = &chain[1..chain.len() - 1];
    // The provider declares B's upstream A and C's upstream B. Roots read none.
    let upstreams: Vec<Vec<String>> = chain
        .iter()
        .map(|member| {
            if !consumers.contains(member) {
                return Vec::new();
            }
            <ChainProvider as WorthQueryApplicationProducerProvider<
                CheckpointSchema,
                ChainProducer<CheckpointSchema>,
            >>::operation_input(
                &ChainProvider,
                &PlanarOutputReadResult {
                    body_key: (*member).to_owned(),
                    successor_body_key: String::new(),
                    value: length(1),
                },
            )
            .upstreams
            .into_iter()
            .map(|upstream| upstream.key)
            .collect()
        })
        .collect();
    let ample = chain_journey(128 * 1024 * 1024).unwrap();
    search(
        "justified handler entries at refused registration",
        1,
        ample.peak_retained_bytes as usize,
        support::capacity_region::Goal::Hit,
        |capacity| {
            let journey = match observed_chain_journey(capacity as u64, true) {
                Ok(journey) => journey,
                Err(answer) => return answer,
            };
            if journey.stops.is_empty() {
                return Attempt::Above("index admitted");
            }
            assert_index_stops(capacity as u64, &journey.stops);
            let advances: Vec<_> = journey
                .refused_advances
                .iter()
                .filter(|advance| advance.published && !advance.decisions.is_empty())
                .collect();
            if advances.is_empty() {
                return Attempt::Below("no post-effect consumer refusal");
            }
            for advance in advances {
                assert_eq!(advance.member_attempts.len(), chain.len());
                assert_eq!(
                    advance.member_attempts.iter().sum::<u64>(),
                    advance.producer_attempts
                );
                assert_eq!(advance.member_publications.len(), chain.len());
                let mut entry_budget = 0;
                for (((member, entries), publications), direct_upstreams) in chain
                    .iter()
                    .zip(&advance.member_attempts)
                    .zip(&advance.member_publications)
                    .zip(&upstreams)
                {
                    assert!(
                        *publications <= 1,
                        "{capacity}: member {member} published {publications} times"
                    );
                    // The observer does not report entry endings. Allow one unfinished
                    // entry per declared upstream; its staleness is not observed.
                    let allowed_awaits = direct_upstreams.len() as u64;
                    let member_budget = publications + allowed_awaits;
                    assert!(
                        *entries <= member_budget,
                        "{capacity}: member {member} has {entries} handler entries, \
                         {publications} publications, and direct upstreams {direct_upstreams:?}"
                    );
                    entry_budget += member_budget;
                }
                assert!(
                    advance.producer_attempts <= entry_budget,
                    "{capacity}: total handler entries exceed publications plus \
                     declared upstream awaits: {advance:?}"
                );
                // Each of these handlers constructs at most one candidate, so
                // one decision per member bounds authoritative publications too.
                assert!(
                    advance.decisions.len() <= consumers.len(),
                    "{capacity}: {advance:?}"
                );
                for member in consumers {
                    assert!(
                        advance
                            .decisions
                            .iter()
                            .filter(|scope| scope == member)
                            .count()
                            <= 1,
                        "{capacity}: member {member} completed more than one chain decision: {advance:?}"
                    );
                }
            }
            Attempt::Hit
        },
    )
    .require_hit("post-effect consumer registration refusal");
}

/// A handler that stops before publication is not a performed member. This
/// separate proof counts real commits after registration has actually refused.
#[test]
fn a_refused_registration_cannot_publish_a_chain_member_twice_in_one_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let ample = chain_journey(128 * 1024 * 1024).unwrap();
    search(
        "publications at refused registration",
        1,
        ample.peak_retained_bytes as usize,
        support::capacity_region::Goal::Hit,
        |capacity| {
            let journey = match observed_chain_journey(capacity as u64, true) {
                Ok(journey) => journey,
                Err(answer) => return answer,
            };
            if journey.stops.is_empty() {
                return Attempt::Above("index admitted");
            }
            assert_index_stops(capacity as u64, &journey.stops);
            let advances: Vec<_> = journey
                .refused_advances
                .iter()
                .filter(|advance| {
                    advance.member_publications.iter().any(|count| *count > 0)
                        && !advance.decisions.is_empty()
                })
                .collect();
            if advances.is_empty() {
                return Attempt::Below("no post-effect consumer refusal");
            }
            for advance in advances {
                assert_eq!(advance.member_publications.len(), CHAIN.len());
                for (member, publications) in CHAIN.iter().zip(&advance.member_publications) {
                    assert!(
                        *publications <= 1,
                        "{capacity}: member {member} published {publications} times"
                    );
                }
            }
            Attempt::Hit
        },
    )
    .require_hit("post-effect consumer registration refusal");
}

/// This public settlement reaches the production publication handoff, not a
/// direct call to the record. A no-op there must leave the seal count behind.
#[test]
fn a_producer_publication_records_its_member_before_checkpointing() {
    let _guard = checkpoint_recovery_test_guard();
    let (application, _) = limited_application(4 * 1024 * 1024, 128 * 1024 * 1024, WINDOW);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let root = request
        .query(PlanarRead {
            body_key: CHAIN[0].to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .root_entity_for_test();
    let before = application.producer_publications_at_root_on_this_thread_for_test(root);
    let recorded_before =
        application.producer_recorded_publications_at_root_on_this_thread_for_test(root);
    let mut demand = request
        .demand(PlanarOutputDemand::new(CHAIN[0]))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    assert!(matches!(
        demand.settle(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let publications =
        application.producer_publications_at_root_on_this_thread_for_test(root) - before;
    assert!(
        publications > 0,
        "the production handoff must have published"
    );
    assert_eq!(
        application.producer_recorded_publications_at_root_on_this_thread_for_test(root)
            - recorded_before,
        publications,
        "every production publication must seal its actual member before checkpointing"
    );
}
