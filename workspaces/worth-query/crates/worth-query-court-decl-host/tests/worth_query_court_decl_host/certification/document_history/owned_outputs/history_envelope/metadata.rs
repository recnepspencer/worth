//! Step metadata is bounded by the installed retention resources.
use super::*;

#[test]
fn step_metadata_is_refused_above_the_installed_bound_and_recorded_at_it() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let document = world::attach(&host);
    let before = observe(&host, &document);
    let scope = request_scope();
    let maximum = ApplicationHistoryRetentionResources::DEFAULT.maximum_step_metadata_bytes;
    let stages = Stages {
        key: "bounded",
        model: 2,
        member: 6,
    };
    let (document, denial) = refused(record(
        &host,
        document,
        &scope,
        stages,
        metadata(&vec![7; maximum + 1]),
    ));
    assert!(
        matches!(
            denial,
            Denial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::StepMetadataTooLarge { length, maximum: bound }
            )) if length == maximum + 1 && bound == maximum
        ),
        "{denial:?}"
    );
    assert_eq!(observe(&host, &document), before);
    let largest = vec![7; maximum];
    let (document, step) = recorded(record(&host, document, &scope, stages, metadata(&largest)));
    assert_eq!(step.metadata().bytes(), largest);
    assert_eq!(step.parent(), Some(before.current));
    close(&host, document);
}
