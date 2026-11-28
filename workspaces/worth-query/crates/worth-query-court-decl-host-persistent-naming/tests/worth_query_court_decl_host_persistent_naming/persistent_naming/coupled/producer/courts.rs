//! The coupled producer's outputs: named exactly as a direct operation names
//! them, kept by its Preserve, refused whole when it leaves one unnamed or
//! mints the name its own edit minted, and followed by history navigation.
use super::*;
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
