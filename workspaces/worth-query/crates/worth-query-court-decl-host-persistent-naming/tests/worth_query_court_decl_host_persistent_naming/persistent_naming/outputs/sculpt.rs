//! One installed operation whose handler executes an authored plan through
//! the ordinary candidate writer: creates, deletes and seam links, plus the
//! persistent-name births and postures the plan declares over them. Its
//! decision reads the names it acts on: members by `name_of`, seams by
//! `relation_name_of`.
//!
//! The plan lives in a token-keyed registry because a court authors it from
//! names held by earlier operations; the handler itself only uses the public
//! decision reader and candidate writer.
use std::collections::BTreeMap;
use std::sync::Mutex;

use super::super::*;
use worth_query_host::facade::application_naming::{
    ExecutionNamingDenial, WorthQueryName, WorthQueryOutputBirth, WorthQueryOutputNameDenial,
};

mod execute;
mod scoped;
pub use scoped::ScopeStep;

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

/// A held seam name: one this run's decision read under a label, or one a
/// court holds.
#[derive(Clone, Debug)]
pub enum HeldSeam {
    Read(String),
    Name(WorthQueryName<Seam>),
}

#[derive(Clone, Debug)]
pub enum SeamKeeper {
    InPlace(HeldSeam),
    Onto(HeldSeam, String),
    MintNew(String, WorthQueryOutputBirth),
}

#[derive(Clone, Debug)]
pub enum Step {
    Birth(String, WorthQueryOutputBirth),
    SeamBirth(String, WorthQueryOutputBirth),
    /// Names created member `key` as the `side` piece cut by `of`, a member
    /// this run created and named: the birth references its same-commit name.
    BirthCutBy {
        key: String,
        birth: WorthQueryOutputBirth,
        side: u64,
        of: String,
    },
    /// Names created member `key` as the `side` piece cut along `seam`, a
    /// seam this run linked and named.
    BirthAlong {
        key: String,
        birth: WorthQueryOutputBirth,
        side: u64,
        seam: String,
    },
    /// Names created member `key` as the `side` piece cut by `of`, a member
    /// this run created and named, optionally along seam `along`, which this
    /// run linked and named. Both are referenced by their same-commit names.
    BirthSided {
        key: String,
        birth: WorthQueryOutputBirth,
        side: outputs::CutSide,
        of: String,
        along: Option<String>,
    },
    /// Names created seam `key`, told apart by created member `of`.
    SeamBirthOf {
        key: String,
        birth: WorthQueryOutputBirth,
        index: u64,
        of: String,
    },
    SeamRetire(HeldSeam),
    SeamSplit(HeldSeam, Vec<(String, WorthQueryOutputBirth)>),
    SeamMerge(Vec<HeldSeam>, SeamKeeper),
    /// Names a fresh `ModelMember` link (an unschemed relation) to `key`.
    NameOwnerLink(String, WorthQueryOutputBirth),
    Retire(Held),
    Split(Held, Vec<(String, WorthQueryOutputBirth)>),
    Merge(Vec<Held>, Keeper),
    /// Declares `steps` through a naming scope over this occurrence.
    Scoped(
        worth_query_host::facade::application_naming::OccurrencePath,
        Vec<ScopeStep>,
    ),
}

/// What one run does. `reads` are resolved (and named) in the decision;
/// `deletes` must be among them; `unlinks` remove the seams read `(from, to)`
/// members hold; `seams` link `(key, from, to)` members; `seam_reads` name
/// the seam between read `(label, from, to)` members in the decision.
/// `owners` list a created `(member, model)` under that model instead of
/// model `a`; `moves` re-own a read `(member, from model, to model)`;
/// `issued` creates outside model `a`'s partition.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    /// Created without a context entity, in the partition the commit issues.
    pub issued: bool,
    pub owners: Vec<(String, String)>,
    pub moves: Vec<(String, String, String)>,
    pub seam_reads: Vec<(String, String, String)>,
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
    seam_names: BTreeMap<String, WorthQueryName<Seam>>,
    refused: Option<Refusal>,
}

/// The typed output-name refusal a handler met while authoring, read from
/// `WorthQueryApplicationAttemptDenial::persistent_name_denial()`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    Unschemed(String),
    NotCreated,
    MissingSplitDiscriminator {
        role: String,
    },
    /// A same-commit reference to a role that is not strictly earlier.
    NonPrecedingRole {
        role: String,
        referenced: String,
    },
    /// A same-commit reference into another occurrence.
    CrossOccurrence,
    /// A same-commit name requested for an output the run has not named.
    UnnamedReference(String),
    /// A least-keeper merge listed no parents.
    EmptyMergeParents,
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
            WorthQueryOutputNameDenial::Assembly(
                ExecutionNamingDenial::NonPrecedingSameOccurrenceRole {
                    role_path,
                    referenced_role,
                    ..
                },
            ) => Self::NonPrecedingRole {
                role: role_path.clone(),
                referenced: referenced_role.clone(),
            },
            WorthQueryOutputNameDenial::CrossOccurrenceReference { .. } => Self::CrossOccurrence,
            WorthQueryOutputNameDenial::UnnamedOutputReference { subject_name, .. } => {
                Self::UnnamedReference(subject_name.clone())
            }
            WorthQueryOutputNameDenial::EmptyMergeParents { .. } => Self::EmptyMergeParents,
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
    read_relations: [Seam, ModelMember]; writes: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")];
    creates: [Member]; deletes: [Member]; links: [ModelMember, Seam]; unlinks: [Seam, ModelMember];
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
            for (_, owner) in &plan.owners {
                reader.resolve_entity(ModelKey::at("ModelKey"), owner.clone())?;
            }
            for (_, from, to) in &plan.moves {
                let from = reader.resolve_entity(ModelKey::at("ModelKey"), from.clone())?;
                reader
                    .relations_from(ModelMember::reference(), &from)
                    .map_err(HandlerExecutionDenial::new)?;
                reader.resolve_entity(ModelKey::at("ModelKey"), to.clone())?;
            }
            let mut names = BTreeMap::new();
            let mut members = BTreeMap::new();
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
                names.insert(key.clone(), name);
                members.insert(key, member);
            }
            let mut seam_names = BTreeMap::new();
            for (label, from, to) in plan.seam_reads {
                let seam = reader
                    .relations_from(Seam::reference(), &members[&from])
                    .map_err(HandlerExecutionDenial::new)?
                    .into_iter()
                    .find(|seam| seam.to() == &members[&to])
                    .unwrap_or_else(|| panic!("no seam `{label}` from `{from}` to `{to}`"));
                let name = reader
                    .reader()
                    .relation_name_of(&seam)
                    .map_err(HandlerExecutionDenial::new)?;
                seam_names.insert(label, name);
            }
            if let Some(run) = runs().get_mut(&input.token) {
                run.names = names;
                run.seam_names = seam_names;
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
        let (plan, names, seam_names) = {
            let runs = runs();
            let run = runs.get(&input.token).expect("authored plan");
            (run.plan.clone(), run.names.clone(), run.seam_names.clone())
        };
        match execute::execute(&plan, &names, &seam_names, &model, writer) {
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
