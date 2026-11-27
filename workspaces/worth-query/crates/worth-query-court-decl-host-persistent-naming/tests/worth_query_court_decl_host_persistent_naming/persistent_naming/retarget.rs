//! #362: a `#[derive(WorthQueryAction)]` action whose input struct holds a
//! `NamedTarget<Member>` field directly, on stock codecs: no delegate, no
//! codec, no attribute on the field. Its decision resolves the target (exact
//! or keep-side), keeps the resolved member's name as the model's anchor and
//! the target itself as the model's held target. A retired or ambiguous
//! target refuses in the domain, a keep-side pick through an ended name never
//! seals, and each commits nothing; the target's kind-tagged stored text
//! refuses a name of another kind.
use super::outputs::sculpt::{Plan, Step};
use super::outputs::{birth, keys, named, piece, read, resolution, resolved_to, MemberCut, World};
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_naming::{
    NameSelection, NamedTarget, ResolvedName, WorthQueryName,
};
use worth_query_decl::facade::{worth_query_identity_namespace, WorthQueryAction};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// The persistent-naming world's binding context for derived actions.
pub struct Context;
impl ApplicationBindingContext<Schema> for Context {
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = String;
    type DenialBinding = edit::DenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "persistent-naming.retarget-key.v1";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
}

const RETARGET: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("persistent-naming.retarget.v1");

/// Has model `model` hold `target`, resolved now.
#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(schema = Schema, context = Context, namespace = RETARGET,
    input = RetargetInput, operation = RetargetMember, binding = Binding,
    result = edit::ResultBinding, decision = Decision,
    access(read_entities: [Model, Member];
        read_fields: [ModelKey::at("ModelKey"), HeldTarget::at("HeldTarget"),
            HeldAnchor::at("HeldAnchor"), MemberKey::at("MemberKey")];
        read_relations: [ModelMember];
        writes: [HeldTarget::at("HeldTarget"), HeldAnchor::at("HeldAnchor")];))]
pub struct Retarget {
    #[worth_query(scope = ModelKey)]
    pub model: String,
    pub target: NamedTarget<Member>,
    /// Keys what the decision resolved (`resolved`).
    pub run: u64,
}

pub struct Decision {
    model: WorthQueryInvariantMutationTarget<Schema, Model>,
    anchor: WorthQueryName<Member>,
}

/// What the decision's resolution of the target answered, by run.
static RESOLVED: Mutex<BTreeMap<u64, String>> = Mutex::new(BTreeMap::new());

fn resolved(run: u64) -> String {
    RESOLVED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&run)
        .unwrap_or_default()
}

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Retarget,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, edit::Denial> {
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), input.model.clone())?;
            reader.field(&model, HeldTarget::at("HeldTarget"))?;
            reader.field(&model, HeldAnchor::at("HeldAnchor"))?;
            let names = reader.reader();
            let answer = names
                .resolve(&input.target.name(), input.target.selection())
                .map_err(HandlerExecutionDenial::new)?;
            let record = |what: String| {
                RESOLVED
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(input.run, what);
            };
            let Ok(ResolvedName::Exact(member)) = answer else {
                record(format!("{answer:?}"));
                return Ok(Err(edit::Denial::ModelMismatch));
            };
            let anchor = reader
                .reader()
                .name_of(&member)
                .map_err(HandlerExecutionDenial::new)?;
            record("Exact".to_owned());
            Ok(Ok(Decision {
                model: reader.mutation_target(&model)?,
                anchor,
            }))
        })();
        match result {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(denial)) => HandlerResult::DomainDenied(denial),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Retarget,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let result = writer.projected_entity(&decision.model).and_then(|model| {
            let target = Some(input.target.clone());
            writer.write_optional_field(&model, HeldTarget::at("HeldTarget"), target)?;
            let anchor = Some(decision.anchor);
            writer.write_optional_field(&model, HeldAnchor::at("HeldAnchor"), anchor)
        });
        match result {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<RetargetMember>(
            RetargetMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}

impl World {
    /// Runs the derived action; an execution refusal comes back rendered.
    fn retarget(
        &self,
        run: u64,
        target: NamedTarget<Member>,
    ) -> (Result<outputs::Outcome, String>, String) {
        let outcome = self
            .request()
            .advanced()
            .mutate(Retarget {
                model: "a".into(),
                target,
                run,
            })
            .without_source()
            .idempotency(&format!("retarget:{run}"))
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|cause| format!("{cause:?}"));
        (outcome, resolved(run))
    }
}

fn committed(outcome: &Result<outputs::Outcome, String>) -> bool {
    matches!(outcome, Ok(outputs::Outcome::Committed { .. }))
}

#[test]
fn a_derived_action_holds_and_resolves_a_named_target_input() {
    let token = "retarget";
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(&["base", "solo"]),
            steps: vec![
                Step::Birth("base".into(), birth("retarget-base", "authored")),
                Step::Birth("solo".into(), birth("retarget-solo", "authored")),
            ],
            ..Plan::default()
        },
    );
    let observed = world.probe(token, &["base", "solo"], 0);
    let (base, solo) = (named(&observed, "base"), named(&observed, "solo"));

    // Exact: resolved, recorded and committed; a later probe resolves what
    // the model now holds.
    let (outcome, answer) = world.retarget(36201, NamedTarget::exact(solo));
    assert!(committed(&outcome), "{outcome:?}");
    assert_eq!(answer, "Exact");
    let observed = world.probe(token, &["solo"], 1);
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "solo")
    );
    assert_eq!(
        resolution(&observed, "stored-anchor"),
        &resolved_to("stored-anchor", "solo")
    );

    // Split `base` into two pieces: `Exact` refuses `Ambiguous` and commits
    // nothing. A keep-side selection resolves to its piece through the ended
    // `base`, which a later `Restore` could revive: the decision seals on
    // that history as a name-history fact and commits, and the stored target
    // resolves to the piece. Holding the piece's own name commits too.
    world.committed(
        &format!("{token}-split"),
        Plan {
            reads: keys(&["base"]),
            deletes: keys(&["base"]),
            creates: keys(&["n1", "n2"]),
            steps: vec![Step::Split(
                read("base"),
                vec![
                    ("n1".into(), piece("retarget-cut", 1, None)),
                    ("n2".into(), piece("retarget-cut", 2, None)),
                ],
            )],
            ..Plan::default()
        },
    );
    let (outcome, answer) = world.retarget(36202, NamedTarget::exact(base));
    assert!(!committed(&outcome), "{outcome:?}");
    assert!(answer.contains("Ambiguous"), "{answer}");
    let observed = world.probe(token, &["solo"], 2);
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "solo"),
        "the refused retarget committed nothing"
    );
    let side = MemberCut::Side {
        side: 1,
        cutter: None,
    };
    let keep = NameSelection::keep_side(&side).expect("a declared side");
    let (outcome, answer) = world.retarget(36203, NamedTarget::new(base, keep));
    assert_eq!(answer, "Exact", "the keep-side selection picked its piece");
    assert!(committed(&outcome), "{outcome:?}");
    let observed = world.probe(token, &["n1"], 3);
    assert_eq!(
        resolution(&observed, "stored-target"),
        &resolved_to("stored-target", "n1"),
        "the stored keep-side target resolves through the ended base"
    );
    let n1 = named(&observed, "n1");
    let (outcome, answer) = world.retarget(36205, NamedTarget::exact(n1));
    assert!(committed(&outcome), "{outcome:?}");
    assert_eq!(answer, "Exact");
    let observed = world.probe(token, &["n1"], 4);
    assert_eq!(
        resolution(&observed, "stored-anchor"),
        &resolved_to("stored-anchor", "n1")
    );

    // A retired target refuses `Retired` and commits nothing.
    world.committed(
        &format!("{token}-retire"),
        super::coupled::candidate::retired("solo", solo),
    );
    let (outcome, answer) = world.retarget(36204, NamedTarget::exact(solo));
    assert!(!committed(&outcome), "{outcome:?}");
    assert!(answer.contains("Retired"), "{answer}");
}

/// The action input's target is captured as its kind-tagged stored text,
/// the one canonical value of `NamedTarget<K>`: text stored for another kind
/// is outside the field's grammar and refuses typed, never reinterpreted.
#[test]
fn a_named_target_input_refuses_stored_text_of_another_kind() {
    let world = World::install();
    let direct = named(&world.probe("retarget-kind", &["direct"], 0), "direct");
    let seam = super::coupled::candidate::seamed("retarget-kind").seam_name(
        "retarget-kind-seam",
        "direct",
        "nested",
    );
    let bounds = ValueDecodeBounds::new(4096, 256, 16, 16);
    let member = NamedTarget::exact(direct);
    let text = encode_canonical_value(&member, bounds).expect("a member target encodes");
    let decoded: NamedTarget<Member> = decode_canonical_value(&text, bounds).expect("round trip");
    assert_eq!(decoded, member);
    let seam_text = encode_canonical_value(&NamedTarget::exact(seam), bounds).unwrap();
    assert!(
        decode_canonical_value::<NamedTarget<Member>>(&seam_text, bounds).is_err(),
        "a seam target never decodes as a member target"
    );
    assert!(
        decode_canonical_value::<NamedTarget<Seam>>(&text, bounds).is_err(),
        "a member target never decodes as a seam target"
    );
}
