//! Warm2/4-member gestures at1/15 retained steps; real member work and final adoption are separate.
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryGroupedHistoryDocument, WorthQueryHistoryGroupPosture as Posture,
    },
    declaration::application_program::{ApplicationActionReference, ApplicationRootComposition},
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::RetentionHistory,
    host::DocumentRetentionSchema,
    programs::{completion::RetentionCompletion, DocumentRetentionFeature},
    retention_entry::{SetRetentionBinding, SetRetentionInput, SetRetentionIntent},
};
type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
type Group<'a> = WorthQueryGroupedHistoryDocument<'a,
DocumentRetentionSchema,
RetentionHistory,
worth_query_testkit_document_retention::facade::document_retention_model::host::RetentionProgramP0,
worth_query_host::facade::application_entry::WorthQueryHistoryTypedMembers<DocumentRetentionSchema,SetRetentionIntent,worth_query_testkit_document_retention::facade::document_retention_model::host::RetentionProgramP0,Action>>;
type Open = for<'a> fn(&'a App, Document<'a>, u128) -> Group<'a>;

fn open<'a, const N: usize>(host: &'a App, document: Document<'a>, key: u128) -> Group<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .group_edits::<_, _, Action, N>(
            host,
            &host.request(&principal, &scope),
            key,
            std::array::from_fn(|index| {
                (
                    SetRetentionIntent {
                        input: SetRetentionInput {
                            identity: DOCUMENT_IDENTITY.into(),
                            retention_days: if index % 2 == 0 { 7 } else { 3 },
                        },
                    },
                    key as u64 + index as u64 + 1,
                )
            }),
        )
        .unwrap()
}

#[derive(Debug)]
struct Cost {
    probes: [u64; 5],
    work: u64,
    peak: u64,
}
#[derive(Debug)]
struct Sample {
    members: Cost,
    adoption: Cost,
    finish: Cost,
}

fn cost(before: [u64; 5], after: [u64; 5]) -> Cost {
    let native = advancement_observations_on_this_thread_for_test();
    assert!(native.iter().all(|entry| entry.refusal.is_none()));
    Cost {
        probes: std::array::from_fn(|index| after[index] - before[index]),
        work: native.iter().map(|entry| entry.report.charged_work()).sum(),
        peak: native
            .iter()
            .map(|entry| entry.report.physical().peak_charged_memory_bytes())
            .max()
            .unwrap_or(0),
    }
}

fn run(depth: usize, members: usize, open: Open) -> Sample {
    let host = publish_on_first_program();
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let mut document = filled(&host, depth);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let parent = document.current_step(&request).unwrap();
    assert_eq!(value(&host, &document), 3);
    let journal = document.observe_history_enrollment_for_test();
    let probes = || {
        let report = journal.snapshot();
        [
            report.node_probes,
            report.attempt_probes,
            report.custody_probes,
            report.child_index_probes,
            report.selection_accesses,
        ]
    };
    let native_observer = WorthQueryExecutionObservation::install_on_this_thread();
    let _ = advancement_observations_on_this_thread_for_test();
    let before = probes();
    let started = Instant::now();
    let mut group = open(
        &host,
        document,
        0x9182_5b00 + depth as u128 * 100 + members as u128 * 10,
    );
    for _ in 0..96 {
        if group.posture() == Posture::ReadyToComplete {
            break;
        }
        let (document, execution) = group.parts();
        execution.advance(document, &request).unwrap();
    }
    assert_eq!(group.posture(), Posture::ReadyToComplete);
    let member_cost = cost(before, probes());
    assert_eq!(group.completed_members(), members);
    {
        let (document, execution) = group.parts();
        assert_eq!(value(&host, document), 3);
        for index in 0..members {
            let WorthQuerySourceCompletion::Fresh { result, .. } =
                execution.completed_source(index).unwrap().evidence()
            else {
                panic!("each original member must really publish")
            };
            assert_eq!(result.retention_days, if index % 2 == 0 { 7 } else { 3 });
        }
    }
    // Qualification above is outside both measured phases.
    let _ = advancement_observations_on_this_thread_for_test();
    let before = probes();
    let step = {
        let (document, execution) = group.parts();
        execution.complete(document, &request).unwrap()
    };
    let adoption_cost = cost(before, probes());
    let before = probes();
    let finished = group.finish(&request).unwrap();
    let finish_cost = cost(before, probes());
    let elapsed = started.elapsed();
    drop(native_observer);
    drop(journal);
    assert_eq!(finished.step, step);
    assert_eq!(step.parent(), Some(parent.id()));
    assert_eq!(finished.completions.len(), members);
    assert_eq!(value(&host, &finished.document), 3);
    assert_eq!(value(&host, &untouched), 7);
    // One new authored edge, regardless of the actual member publications.
    let limits = worth_query_host::facade::application_entry::WorthQueryHistoryPageLimits {
        maximum_entries: std::num::NonZeroUsize::new(32).unwrap(),
        maximum_examined_entries: std::num::NonZeroUsize::new(32).unwrap(),
        maximum_owned_bytes: std::num::NonZeroUsize::new(64 * 1024).unwrap(),
    };
    assert_eq!(
        finished
            .document
            .history_page(&request, limits, None)
            .unwrap()
            .rows()
            .len(),
        depth + 2
    );
    drop(finished.completions);
    close(&host, finished.document, depth + 2);
    close(&host, untouched, 1);
    let sample = Sample {
        members: member_cost,
        adoption: adoption_cost,
        finish: finish_cost,
    };
    eprintln!("group depth={depth} members={members}; {sample:?}; elapsed={elapsed:?}");
    sample
}

#[test]
fn grouped_member_work_and_one_edge_adoption_scale_with_members_without_visiting_unrelated_history()
{
    let mut samples = Vec::new();
    for depth in [1, 15] {
        samples.push([run(depth, 2, open::<2>), run(depth, 4, open::<4>)]);
    }
    for (short, longer) in samples[0].iter().zip(&samples[1]) {
        for (small, large) in [
            (&short.members, &longer.members),
            (&short.adoption, &longer.adoption),
            (&short.finish, &longer.finish),
        ] {
            assert_eq!(
                small.probes, large.probes,
                "no phase may scan old step records"
            );
            assert!(
                large.work <= small.work * 3,
                "15x old history must not produce linear work amplification: {small:?} {large:?}"
            );
        }
        // Finish's peak is telemetry: release-only execution can report zero.
        for (small, large) in [
            (&short.members, &longer.members),
            (&short.adoption, &longer.adoption),
        ] {
            assert!(
                small.peak > 0 && large.peak > 0,
                "memory accounting must be observed"
            );
            assert!(
                large.peak <= small.peak * 3,
                "history must not be copied into this group's working set"
            );
        }
        assert!(short.members.work > 0 && short.adoption.work > 0);
    }
    for at_depth in &samples {
        let [two, four] = at_depth;
        assert!(
            four.members.work > two.members.work,
            "four real edits do more work than two"
        );
        assert!(
            four.members.work <= two.members.work * 3,
            "member work stays within a linear envelope"
        );
        assert!(
            four.adoption.work <= two.adoption.work * 2,
            "one edge does not replay the member commands"
        );
        assert!(
            four.finish.work <= two.finish.work * 3,
            "disposal may grow with the original owners"
        );
    }
}
