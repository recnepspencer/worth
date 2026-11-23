//! One installed operation whose handler executes an authored plan through
//! the ordinary candidate writer: creates, deletes and seam links, plus the
//! persistent-name births and postures the plan declares over them.
//!
//! The plan lives in a token-keyed registry because a court authors it from
//! names held by earlier operations; the handler itself only uses the public
//! decision reader and candidate writer.
use std::collections::BTreeMap;
use std::sync::Mutex;

use super::super::*;
use worth_query_host::facade::application_naming::{
    ExecutionNamingDenial, WorthQueryName, WorthQueryOutputBirth, WorthQueryOutputMergeKeeper,
    WorthQueryOutputNameDenial,
};

/// A held member name: one read by this run's decision, or one a court holds.
#[derive(Clone, Debug)]
pub enum Held {
    Read(String),
    Name(WorthQueryName<Member>),
}

#[derive(Clone, Debug)]
pub enum Keeper {
    InPlace(Held),
    Onto(Held, String),
    MintNew(String, WorthQueryOutputBirth),
}

#[derive(Clone, Debug)]
pub enum Step {
    Birth(String, WorthQueryOutputBirth),
    SeamBirth(String, WorthQueryOutputBirth),
    /// Names a fresh `ModelMember` link (an unschemed relation) to `key`.
    NameOwnerLink(String, WorthQueryOutputBirth),
    Retire(Held),
    Split(Held, Vec<(String, WorthQueryOutputBirth)>),
    Merge(Vec<Held>, Keeper),
}

/// What one run does. `reads` are resolved (and named) in the decision;
/// `deletes` must be among them; `unlinks` remove the seams read `(from, to)`
/// members hold; `seams` link `(key, from, to)` members.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub reads: Vec<String>,
    pub creates: Vec<String>,
    pub deletes: Vec<String>,
    pub unlinks: Vec<(String, String)>,
    pub seams: Vec<(String, String, String)>,
    pub steps: Vec<Step>,
}

#[derive(Default)]
struct Run {
    plan: Plan,
    names: BTreeMap<String, WorthQueryName<Member>>,
    refused: Option<Refusal>,
}

/// The typed output-name refusal a handler met while authoring, read from
/// `WorthQueryApplicationAttemptDenial::persistent_name_denial()`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    Unschemed(String),
    NotCreated,
    MissingSplitDiscriminator { role: String },
    Other(String),
}

impl Refusal {
    fn of(denial: &WorthQueryOutputNameDenial) -> Self {
        match denial {
            WorthQueryOutputNameDenial::UnschemedOutput { subject_name, .. } => {
                Self::Unschemed(subject_name.clone())
            }
            WorthQueryOutputNameDenial::NotCreatedOutput { .. } => Self::NotCreated,
            WorthQueryOutputNameDenial::Assembly(
                ExecutionNamingDenial::MissingSplitDiscriminator { role_path, .. },
            ) => Self::MissingSplitDiscriminator {
                role: role_path.clone(),
            },
            other => Self::Other(other.to_string()),
        }
    }
}

static RUNS: Mutex<BTreeMap<String, Run>> = Mutex::new(BTreeMap::new());

fn runs() -> std::sync::MutexGuard<'static, BTreeMap<String, Run>> {
    RUNS.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Registers `plan` for the next run carrying `token`.
pub fn author(token: &str, plan: Plan) {
    runs().insert(
        token.to_owned(),
        Run {
            plan,
            ..Run::default()
        },
    );
}

/// The typed output-name refusal the handler met while authoring `token`.
pub fn authoring_refusal(token: &str) -> Option<Refusal> {
    runs().get(token).and_then(|run| run.refused.clone())
}

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub token: String,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.sculpt.v1" });
worth_query_operation!(pub Sculpt for Schema, input Input; access {
    read_entities: [Model, Member]; read_fields: [ModelKey::at("ModelKey"), MemberKey::at("MemberKey")];
    read_relations: [Seam]; writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    creates: [Member]; deletes: [Member]; links: [ModelMember, Seam]; unlinks: [Seam];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.sculpt.v1", input Input, operation Sculpt, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.sculpt-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "persistent-naming.sculpt-handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value scope,
);

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, edit::Denial> {
        let plan = runs().get(&input.token).map(|run| run.plan.clone());
        let plan = plan.unwrap_or_default();
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
            let mut names = BTreeMap::new();
            for key in plan.reads {
                let member = reader.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
                reader.entity(&member, Member::reference())?;
                if plan.unlinks.iter().any(|(from, _)| *from == key) {
                    reader
                        .relations_from(Seam::reference(), &member)
                        .map_err(HandlerExecutionDenial::new)?;
                }
                let name = reader
                    .reader()
                    .name_of(&member)
                    .map_err(HandlerExecutionDenial::new)?;
                names.insert(key, name);
            }
            if let Some(run) = runs().get_mut(&input.token) {
                run.names = names;
            }
            reader.mutation_target(&model)
        })();
        match result {
            Ok(model) => HandlerResult::Completed(model),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Intent,
        model: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        let (plan, names) = {
            let runs = runs();
            let run = runs.get(&input.token).expect("authored plan");
            (run.plan.clone(), run.names.clone())
        };
        match execute(&plan, &names, &model, writer) {
            Ok(()) => HandlerResult::Completed(edit::Edited),
            Err(cause) => {
                if let Some(run) = runs().get_mut(&input.token) {
                    run.refused = cause.persistent_name_denial().map(Refusal::of);
                }
                HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        }
    }
}

fn execute(
    plan: &Plan,
    names: &BTreeMap<String, WorthQueryName<Member>>,
    model: &WorthQueryInvariantMutationTarget<Schema, Model>,
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let held = |held: &Held| match held {
        Held::Read(key) => *names.get(key).expect("decision read the held member"),
        Held::Name(name) => *name,
    };
    let context = writer.projected_entity(model)?;
    let mut members = BTreeMap::new();
    for key in &plan.creates {
        let created = writer.create_entity_in_context(
            &context,
            Member::reference(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        )?;
        writer.initialize_field(&created, MemberKey::at("MemberKey"), key.clone())?;
        writer.initialize_field(&created, MemberValue::at("MemberValue"), 7)?;
        writer.link(
            ModelMember::reference(),
            format!("owns:{key}"),
            &context,
            &created,
        )?;
        members.insert(key.clone(), created);
    }
    for key in &plan.reads {
        let existing = writer.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
        members.entry(key.clone()).or_insert(existing);
    }
    for (from, to) in &plan.unlinks {
        writer.unlink(Seam::reference(), &members[from], &members[to])?;
    }
    for key in &plan.deletes {
        writer.delete_entity(Member::reference(), &members[key])?;
    }
    let mut seams = BTreeMap::new();
    for (key, from, to) in &plan.seams {
        let seam = writer.link_for_initialization(
            Seam::reference(),
            key.clone(),
            &members[from],
            &members[to],
        )?;
        seams.insert(key.clone(), seam);
    }
    for step in &plan.steps {
        match step {
            Step::Birth(key, birth) => writer.name_created_entity(&members[key], birth.clone())?,
            Step::SeamBirth(key, birth) => {
                writer.name_created_relation(&seams[key], birth.clone())?
            }
            Step::NameOwnerLink(key, birth) => {
                let link = writer.link_for_initialization(
                    ModelMember::reference(),
                    format!("named-owner:{key}"),
                    &context,
                    &members[key],
                )?;
                writer.name_created_relation(&link, birth.clone())?
            }
            Step::Retire(name) => writer.retire_entity_name(held(name))?,
            Step::Split(parent, children) => {
                let children = children
                    .iter()
                    .map(|(key, birth)| (&members[key], birth.clone()))
                    .collect();
                writer.split_entity_name(held(parent), children)?
            }
            Step::Merge(parents, keeper) => {
                let parents: Vec<_> = parents.iter().map(held).collect();
                let keeper = match keeper {
                    Keeper::InPlace(name) => {
                        WorthQueryOutputMergeKeeper::KeepParentInPlace(held(name))
                    }
                    Keeper::Onto(name, key) => WorthQueryOutputMergeKeeper::KeepParentOnto {
                        keeper: held(name),
                        target: &members[key],
                    },
                    Keeper::MintNew(key, birth) => WorthQueryOutputMergeKeeper::MintNew {
                        target: &members[key],
                        birth: birth.clone(),
                    },
                };
                writer.merge_entity_names(&parents, keeper)?
            }
        }
    }
    Ok(())
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Sculpt>(
            Sculpt::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
