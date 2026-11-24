//! The coupled producer's outputs: named exactly as a direct operation names
//! them, kept by its Preserve, refused whole when it leaves one unnamed or
//! mints the name its own edit minted, and followed by history navigation.
use super::super::super::history::{navigate, probe, probe_refused, record_edit, Navigate};
use super::*;
use probe::Observation;
use worth_query_host::facade::application_naming::LineageRecordTarget;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

type ShapeAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    shape::Binding,
    Completion,
>;

impl World {
    /// Edits model `key` to `value`; the producer runs in the same envelope.
    pub fn shape(
        &self,
        key: &str,
        value: u64,
        collide: bool,
        idempotency: &str,
    ) -> outputs::Outcome {
        self.request()
            .advanced()
            .mutate(shape::Intent {
                key: key.to_owned(),
                value,
                collide,
            })
            .without_source()
            .idempotency(&idempotency.to_owned())
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_or_else(|cause| panic!("{idempotency}: {cause:?}"))
    }

    pub(crate) fn member_value(&self, key: &str) -> u64 {
        self.request()
            .query(query::member::Read(key.to_owned()))
            .execute()
            .unwrap()
            .rows()[0]
            .value
    }

    pub(crate) fn model_value(&self, key: &str) -> u64 {
        self.request()
            .query(query::model::Read(key.to_owned()))
            .execute()
            .unwrap()
            .rows()[0]
            .value
    }
}

/// The member and seam names a model's producer outputs carry.
fn outputs_of(world: &World, token: &str, model: &str, ordinal: u64) -> Vec<String> {
    let (primary, secondary) = (primary_key(model), secondary_key(model));
    probe::seams::name(token, "seam", &primary, &secondary);
    let keys = [primary.as_str(), secondary.as_str()];
    let (seams, _) = world.probe_seams(token, &keys, ordinal);
    let members = world.names(token, &keys, ordinal + 100);
    let mut rendered: Vec<String> = members.iter().map(|name| format!("{name:?}")).collect();
    rendered.push(format!("{:?}", outputs::seam_named(&seams, "seam")));
    rendered
}

#[test]
fn a_coupled_producer_names_its_outputs_as_a_direct_operation_does() {
    let coupled = World::install();
    let outcome = coupled.shape("a", 5, false, "producer-initial");
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    let minted = outputs_of(&coupled, "producer-coupled", "a", 0);

    let direct = World::install();
    let (primary, secondary) = (primary_key("a"), secondary_key("a"));
    direct.committed(
        "producer-direct",
        Plan {
            creates: vec![primary.clone(), secondary.clone()],
            steps: vec![scoped(
                &occurrence("a"),
                vec![
                    ScopeStep::Name(primary.clone(), MemberRole::Authored),
                    ScopeStep::CutBy {
                        key: secondary.clone(),
                        side: outputs::CutSide::Positive,
                        of: primary.clone(),
                    },
                    ScopeStep::LinkSeamOf {
                        key: seam_key("a"),
                        from: primary.clone(),
                        to: secondary.clone(),
                        index: 0,
                        of: primary.clone(),
                    },
                ],
            )],
            ..Plan::default()
        },
    );
    assert_eq!(
        outputs_of(&direct, "producer-direct-names", "a", 0),
        minted,
        "the producer's members and seam carry the names a direct operation mints"
    );

    // A later edit runs the Preserve: values move, names stay.
    let outcome = coupled.shape("a", 9, false, "producer-preserve");
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    assert_eq!(
        (
            coupled.member_value(&primary),
            coupled.member_value(&secondary)
        ),
        (9, 10)
    );
    assert_eq!(outputs_of(&coupled, "producer-preserved", "a", 1), minted);
}

#[test]
fn a_coupled_producer_leaving_an_output_unnamed_refuses_with_no_effect() {
    let world = World::install();
    name_outputs("b", Naming::LeaveSecondaryUnnamed);
    let outcome = world.shape("b", 5, false, "producer-unnamed");
    match outputs::naming_refusal(&outcome) {
        Some(LineageNameDenial::MissingBirth {
            target: LineageRecordTarget::Entity(_),
            scheme_id,
        }) => assert_eq!(scheme_id, MEMBER_SCHEME),
        other => panic!("the unnamed output refuses MissingBirth: {other:?} / {outcome:?}"),
    }
    assert!(!world.live(&primary_key("b")) && !world.live(&secondary_key("b")));
    assert_eq!(
        world.model_value("b"),
        1,
        "the edit's own effect never committed"
    );
}

#[test]
fn an_edit_and_its_producer_minting_one_name_refuse_duplicate_birth() {
    let world = World::install();
    let outcome = world.shape("a", 5, true, "producer-collide");
    assert!(
        matches!(
            outputs::naming_refusal(&outcome),
            Some(LineageNameDenial::DuplicateBirth { .. })
        ),
        "{outcome:?}"
    );
    for key in [
        primary_key("a"),
        secondary_key("a"),
        "shape-a-collide".to_owned(),
    ] {
        assert!(!world.live(&key), "{key}: zero effects");
    }
    assert_eq!(world.model_value("a"), 1);
}

#[test]
fn producer_named_outputs_follow_checkout_and_redo() {
    let token = "producer-history";
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.current_step(&request).unwrap();
    let intent = shape::Intent {
        key: "a".into(),
        value: 5,
        collide: false,
    };
    let document = record_edit!(
        &host,
        document,
        &request,
        intent,
        ShapeAction,
        "producer-history-1"
    );
    let shaped = document.current_step(&request).unwrap();
    let (primary, secondary) = (primary_key("a"), secondary_key("a"));
    let keys = [primary.as_str(), secondary.as_str()];
    let (document, observed) = probe(&host, document, &request, token, &keys, 0);
    let (p, s) = (named(&observed, &primary), named(&observed, &secondary));
    probe::hold(token, "p", p);
    probe::hold(token, "s", s);

    // Before the edit neither output exists: their held names are retired.
    navigate(
        &document,
        &request,
        Navigate::Checkout(before.id()),
        0x3b21_0001,
    );
    let (document, observed, _) = probe_refused(&host, document, &request, token, &[], 1);
    assert_eq!(
        outputs::resolution(&observed, "p"),
        &Observation::Retired { label: "p".into() }
    );
    assert_eq!(
        outputs::resolution(&observed, "s"),
        &Observation::Retired { label: "s".into() }
    );

    // Back on the edit, both outputs carry their exact names again.
    navigate(
        &document,
        &request,
        Navigate::Redo(shaped.id()),
        0x3b21_0002,
    );
    let (document, observed) = probe(&host, document, &request, token, &keys, 2);
    assert_eq!(
        (named(&observed, &primary), named(&observed, &secondary)),
        (p, s)
    );
    assert_eq!(
        outputs::resolution(&observed, "p"),
        &outputs::resolved_to("p", &primary)
    );
    assert_eq!(
        outputs::resolution(&observed, "s"),
        &outputs::resolved_to("s", &secondary)
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
