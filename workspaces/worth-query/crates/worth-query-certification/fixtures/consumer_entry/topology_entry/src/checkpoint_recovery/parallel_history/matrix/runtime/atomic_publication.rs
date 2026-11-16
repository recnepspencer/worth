//! One final publication replaces the obsolete independently committed prefix.
use super::super::{control::Phase, edit};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeOutcome as Outcome,
        WorthQueryEditStageToken,
    },
    primary_graph::{WorthQueryApplicationUncommitted, WorthQueryOutputDemandDenialKind},
};

pub(super) fn assert_outcome(
    outcome: &Result<Outcome, WorthQueryEditEnvelopeExecutionDenial>,
    tokens: &[WorthQueryEditStageToken<edit::Outcome, edit::Denied>],
    failure: Option<(u64, Phase)>,
    expired: bool,
    publications: usize,
) {
    if expired {
        assert_eq!(publications, 0);
        return;
    }
    if let Some((_, phase)) = failure {
        let Ok(Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial))) = outcome
        else {
            panic!("the required participant retains its original typed refusal: {outcome:?}")
        };
        let cause = denial
            .coupled_producer_denial()
            .expect("the failed producer's cause");
        assert_eq!(
            cause.kind(),
            WorthQueryOutputDemandDenialKind::ProducerDomainDenied
        );
        assert_eq!(
            cause.domain_reason(),
            Some(match phase {
                Phase::Prepare => "matrix prepare refusal",
                Phase::Compute => "matrix compute refusal",
                Phase::Complete => "matrix completion refusal",
            })
        );
        assert_eq!(
            publications, 0,
            "refused candidate publishes no sources or participants"
        );
        return;
    }
    let Ok(Outcome::Committed(receipt)) = outcome else {
        panic!("one grouped edit commits: {outcome:?}")
    };
    assert_eq!(
        publications, 1,
        "source and every required output share one physical publication"
    );
    assert_eq!(receipt.stage_count(), tokens.len());
    for (ordinal, token) in tokens.iter().enumerate() {
        assert_eq!(receipt.result(token).unwrap().revision, 1);
        assert_eq!(receipt.stage_identity(token).unwrap().ordinal(), ordinal);
        assert_eq!(
            receipt.stage_receipt(token).unwrap().commit_reference(),
            receipt.receipt().commit_reference(),
            "every genuine authored stage has the sole final commit identity"
        );
    }
}

/// Fresh delivery must correspond to each real root without repairing the candidate.
pub(super) fn assert_current_outputs(
    app: &super::super::installation::Application,
    world: &super::super::World,
    names: &std::collections::BTreeMap<worth_relational::facade::identity::EntityId, u64>,
    expected: &std::collections::BTreeMap<u64, u64>,
) {
    use super::super::{application::*, facts, installation, operation, producer};
    use worth_query_host::facade::application_entry::{
        WorthQueryApplicationOutputDemandProgress as Progress, WorthQueryApplicationRequestExt,
    };
    let (scope, principal) = installation::authenticate(app);
    let request = app.request(&principal, &scope);
    let (_, writes) = worth_relational::facade::inspection::observe_maintenance_work(1, || {
        for key in world.members.iter().map(|m| m.key).chain([100]) {
            let mut interest = request
                .demand(producer::Demand(key))
                .start_in_program::<Program, Root>(app)
                .unwrap();
            let settled = (0..128)
                .find_map(|_| match interest.advance(&request).unwrap() {
                    Progress::Pending | Progress::Progressed => None,
                    Progress::Settled(value) => Some(value),
                })
                .expect("fresh current output demand settles");
            // A current accepted original may retain Performed receipt provenance.
            // Contact and publication observations establish whether work was reused.
            assert_eq!(
                settled.producer_contacts_in_this_demand(),
                0,
                "fresh authority delivers the actual current output without producer repair"
            );
            let outputs = settled.outputs_of::<operation::Outputs>().unwrap();
            let node = outputs.entity::<operation::NodeOutput<Schema>>().unwrap();
            assert_eq!(
                names[&node.entity_id()],
                key,
                "settlement corresponds to its genuine root"
            );
            let value = request.query(facts::Read { key }).execute().unwrap().rows()[0].value;
            let expected_value = if key == 100 {
                world
                    .requested
                    .iter()
                    .map(|i| expected[&world.members[*i].key])
                    .sum()
            } else {
                expected[&key]
            };
            assert_eq!(
                value, expected_value,
                "fresh current delivery preserves independent member values"
            );
        }
    });
    assert_eq!(
        writes.published_rows.len() + writes.omitted_publications,
        0,
        "currentness verification cannot repair or republish any participant"
    );
}
