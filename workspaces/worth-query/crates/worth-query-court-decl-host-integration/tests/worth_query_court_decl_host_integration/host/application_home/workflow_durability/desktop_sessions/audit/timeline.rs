//! A workflow instance's audit trail and its record's causal timeline, both
//! read at one pinned commit after the desktop journey's transitions.
use super::super::super::observation::input as read_input;
use super::*;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::audit::{
    WorthQueryCausalTimeline, WorthQueryCausalTimelineDenial, WorthQueryWorkflowAuditCompleteness,
    WorthQueryWorkflowAuditRequest, WorthQueryWorkflowAuditTrail,
};

#[test]
fn workflow_audit_pages_every_recorded_transition_and_a_timeline_refuses_whole() {
    let trusted = trusted_now();
    let (home, _authentication) = open(&Location::MemoryOnly, &trusted);
    let (operator, reviewer) = principals(&home);
    let operator = home.session(&operator);
    let (original, reached) = publish_start_propose_and_assess(&home);

    let reviewer = home.session(&reviewer);
    let pinned = reviewer.snapshot().unwrap();
    let paged = audit_trail(&home, &pinned, &original, 1, 64);
    assert_eq!(
        paged.completeness(),
        WorthQueryWorkflowAuditCompleteness::Complete
    );
    assert_eq!(paged.transitions().count(), reached.history, "{paged:?}");
    assert_eq!(
        paged.basis().map(|basis| basis.selected_commit()),
        Some(pinned.selected_commit())
    );
    let wide = audit_trail(&home, &pinned, &original, 16, 1);
    assert_eq!(
        occurrences(&paged),
        occurrences(&wide),
        "page width never changes the recorded transitions"
    );
    assert!(paged.transitions().any(|transition| matches!(
        transition.audit(),
        WorthQueryWorkflowHistoryAudit::Visible { .. }
    )));
    assert!(reached.history > 1, "{reached:?}");
    let truncated = audit_trail(&home, &pinned, &original, 1, 1);
    assert_eq!(
        truncated.completeness(),
        WorthQueryWorkflowAuditCompleteness::PageLimitReached
    );

    // No snapshot retained the journey's product parents, so the record walk
    // meets World's retired-history gap: the timeline refuses as a whole
    // even though its workflow stream alone is readable.
    let refused = causal_timeline(&home, &operator, &original);
    assert!(
        matches!(refused, Err(WorthQueryCausalTimelineDenial::Record(_))),
        "{refused:?}"
    );
}

#[test]
fn causal_timeline_reads_both_streams_at_one_pinned_commit() {
    let trusted = trusted_now();
    let (home, _authentication) = open(&Location::MemoryOnly, &trusted);
    let (operator, _) = principals(&home);
    let local = home
        .local_session(local_operator(), &WorthQueryAdmissionClock::system())
        .unwrap();
    let operator = home.session(&operator);
    // Retain each product parent the record walk steps through.
    let mut custody = vec![operator.snapshot().unwrap()];
    let (definition, target) = session_requests::publish(&home, &local, 4601);
    custody.push(operator.snapshot().unwrap());
    let (instance, original) = session_requests::start(&home, &operator, definition, 4602);
    custody.push(operator.snapshot().unwrap());
    session_requests::propose(&home, &operator, instance, &target, 4603);
    custody.push(operator.snapshot().unwrap());
    set_retention(&home, &operator, 5, 4604);
    custody.push(operator.snapshot().unwrap());

    let timeline = causal_timeline(&home, &operator, &original).unwrap();
    let pinned = custody.last().unwrap();
    assert_eq!(timeline.commit(), pinned.selected_commit());
    let independent = frontier_at(pinned, home.vocabulary(), &original);
    assert_eq!(
        timeline.workflow().transitions().count(),
        independent.history,
        "{timeline:?}"
    );
    assert_eq!(
        timeline.record().changes().count(),
        pinned
            .record_history::<GovernedChanges>(change_log())
            .unwrap()
            .changes()
            .count()
    );
    assert!(timeline.record().changes().count() >= 1, "{timeline:?}");
    assert_eq!(
        timeline
            .workflow()
            .basis()
            .map(|basis| basis.selected_commit()),
        Some(timeline.commit())
    );
}

fn audit_trail(
    home: &Home,
    snapshot: &Snapshot<'_, '_>,
    original: &Original,
    page_width: usize,
    maximum_pages: usize,
) -> WorthQueryWorkflowAuditTrail {
    snapshot.with_request(|at| {
        let observation = observed(at, home, original);
        snapshot
            .workflow_audit::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                home.vocabulary(),
                WorthQueryWorkflowAuditRequest {
                    target: observation.target(),
                    subject: read_input(),
                    audit: read_input(),
                    page_width,
                    indexes: WorthQueryWorkflowIndexRead::PublishedOnly,
                    maximum_pages: NonZeroUsize::new(maximum_pages).unwrap(),
                },
            )
            .unwrap()
    })
}

fn causal_timeline(
    home: &Home,
    session: &Session<'_>,
    original: &Original,
) -> Result<WorthQueryCausalTimeline, WorthQueryCausalTimelineDenial> {
    let snapshot = session.snapshot().unwrap();
    snapshot.with_request(|at| {
        let observation = observed(at, home, original);
        snapshot.causal_timeline::<GovernedChanges, _, SubjectRead, AuditRead>(
            change_log(),
            home.vocabulary(),
            WorthQueryWorkflowAuditRequest {
                target: observation.target(),
                subject: read_input(),
                audit: read_input(),
                page_width: 4,
                indexes: WorthQueryWorkflowIndexRead::PublishedOnly,
                maximum_pages: NonZeroUsize::new(16).unwrap(),
            },
        )
    })
}

fn observed(
    at: &WorthQueryApplicationRetainedRequest<
        '_,
        '_,
        '_,
        document::schema::DocumentRetentionSchema,
    >,
    home: &Home,
    original: &Original,
) -> WorthQueryWorkflowObservation {
    match at
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(),
            original,
            read_input(),
            read_input(),
        )
        .unwrap()
    {
        WorthQueryWorkflowInstanceRead::Live { observation, .. }
        | WorthQueryWorkflowInstanceRead::Historical { observation, .. } => observation,
        WorthQueryWorkflowInstanceRead::Missing { .. } => {
            panic!("the original instance is disclosed")
        }
    }
}

fn occurrences(trail: &WorthQueryWorkflowAuditTrail) -> Vec<u64> {
    trail
        .transitions()
        .map(|transition| transition.occurrence())
        .collect()
}
