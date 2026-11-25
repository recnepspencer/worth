//! An installed operation whose decision reads persistent names through the
//! tracked reader: `name_of` on live members and `resolve` on held names,
//! and (`seams`) `relation_name_of` and `resolve_relation` on seams.
//!
//! The decision runs inside the real projected read attempt, so every read
//! is admitted against the operation's realized scope and its dependency
//! facts must seal for the operation to complete.
use std::collections::BTreeMap;
use std::sync::Mutex;

use super::*;
use worth_query_decl::facade::application_naming::{
    NameCandidate, NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_host::facade::application_naming::ExecutionNameResolveDenial;

pub mod births;
pub mod seams;

/// What one probe run observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Observation {
    /// `name_of` on the live member seeded under `key`.
    Named {
        key: String,
        name: WorthQueryName<Member>,
    },
    /// `name_of` refused for `key`.
    NameRefused { key: String, refusal: String },
    /// The member `key` is not live at the probe's basis.
    Absent { key: String },
    /// A held name resolved to the live member currently keyed `key`.
    ResolvedTo { label: String, key: String },
    /// A held name resolved to a live member none of the probe's keys select.
    ResolvedElsewhere { label: String },
    /// A held name's lineage is retired at the probe's basis.
    Retired { label: String },
    /// Resolution refused with the stated typed denial.
    ResolveRefused { label: String, refusal: String },
    /// A held name resolved to two or more live members (by probe key).
    Plural { label: String, keys: Vec<String> },
    /// `Exact` refused with every live candidate name.
    Ambiguous {
        label: String,
        candidates: Vec<WorthQueryName<Member>>,
    },
    /// Resolution refused with every live candidate and the keys of the
    /// models that own it.
    AmbiguousOwned {
        label: String,
        candidates: Vec<(WorthQueryName<Member>, Vec<String>)>,
    },
    /// The requested model owns none of the matching live candidates, listed
    /// with the keys of the models that do.
    NotInOwner {
        label: String,
        candidates: Vec<(WorthQueryName<Member>, Vec<String>)>,
    },
}

/// Where a held name is resolved.
#[derive(Clone)]
enum Scope {
    /// Across every model.
    Anywhere,
    /// Across every model, reporting each ambiguous candidate's owners.
    AnywhereOwned,
    /// Within the model keyed by this value.
    In(String),
}

#[derive(Clone)]
struct Held {
    label: String,
    name: WorthQueryName<Member>,
    selection: NameSelection,
    scope: Scope,
}

#[derive(Default)]
struct Probe {
    held: Vec<Held>,
    observed: Vec<Observation>,
}

static PROBES: Mutex<BTreeMap<String, Probe>> = Mutex::new(BTreeMap::new());

/// Holds `name` under `label` for every later probe run carrying `token`.
pub fn hold(token: &str, label: &str, name: WorthQueryName<Member>) {
    hold_selecting(token, label, name, NameSelection::Exact);
}

/// Holds `name` under `label`, resolved with `selection` by later probes.
pub fn hold_selecting(
    token: &str,
    label: &str,
    name: WorthQueryName<Member>,
    selection: NameSelection,
) {
    hold_scoped(token, label, name, selection, Scope::Anywhere);
}

/// Holds `name` resolved across every model, with each ambiguous
/// candidate's owning models reported.
pub fn hold_owned(
    token: &str,
    label: &str,
    name: WorthQueryName<Member>,
    selection: NameSelection,
) {
    hold_scoped(token, label, name, selection, Scope::AnywhereOwned);
}

/// Holds `name` resolved only within the model keyed `owner`.
pub fn hold_in(
    token: &str,
    label: &str,
    name: WorthQueryName<Member>,
    selection: NameSelection,
    owner: &str,
) {
    hold_scoped(token, label, name, selection, Scope::In(owner.to_owned()));
}

fn hold_scoped(
    token: &str,
    label: &str,
    name: WorthQueryName<Member>,
    selection: NameSelection,
    scope: Scope,
) {
    let mut probes = PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    probes.entry(token.to_owned()).or_default().held.push(Held {
        label: label.to_owned(),
        name,
        selection,
        scope,
    });
}

/// Takes what probe runs carrying `token` observed since the last take.
pub fn take(token: &str) -> Vec<Observation> {
    let mut probes = PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    probes
        .get_mut(token)
        .map(|probe| std::mem::take(&mut probe.observed))
        .unwrap_or_default()
}

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub token: String,
    pub keys: Vec<String>,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "persistent-naming.probe.v1" });
worth_query_operation!(pub ProbeNames for Schema, input Input; access {
    read_entities: [Model, Member]; read_fields: [ModelKey::at("ModelKey"), ModelValue::at("ModelValue"), MemberKey::at("MemberKey"),
        HeldTarget::at("HeldTarget"), HeldAnchor::at("HeldAnchor")];
    read_relations: [Seam, ModelMember]; writes: [ModelValue::at("ModelValue")];
});
fn scope(_: &Intent) -> String {
    "a".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "persistent-naming.probe.v1", input Input, operation ProbeNames, result edit::ResultBinding,
    idempotency String, identity "persistent-naming.probe-key.v1", decision WorthQueryInvariantMutationTarget<Schema, Model>, denial edit::DenialBinding,
    handler identity "persistent-naming.probe-handler.v1", program required, outputs NoApplicationMutationOutputs,
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
        let result = (|| {
            let model = reader.resolve_entity(ModelKey::at("ModelKey"), "a".to_owned())?;
            reader.field(&model, ModelValue::at("ModelValue"))?;
            let mut observed = Vec::new();
            let mut refused = None;
            let mut live = Vec::new();
            for key in &input.keys {
                let member =
                    reader.resolve_optional_entity(MemberKey::at("MemberKey"), key.clone())?;
                let Some(member) = member else {
                    observed.push(Observation::Absent { key: key.clone() });
                    continue;
                };
                observed.push(match reader.reader().name_of(&member) {
                    Ok(name) => Observation::Named {
                        key: key.clone(),
                        name,
                    },
                    Err(refusal) => Observation::NameRefused {
                        key: key.clone(),
                        refusal: format!("{refusal:?}"),
                    },
                });
                live.push((key.clone(), member));
            }
            let mut held = PROBES
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&input.token)
                .map(|probe| probe.held.clone())
                .unwrap_or_default();
            // What model `a`'s own record holds, resolved like any held name.
            if let Some(target) = reader.field(&model, HeldTarget::at("HeldTarget"))? {
                held.push(Held {
                    label: "stored-target".into(),
                    name: target.name(),
                    selection: target.selection().clone(),
                    scope: Scope::Anywhere,
                });
            }
            if let Some(anchor) = reader.field(&model, HeldAnchor::at("HeldAnchor"))? {
                held.push(Held {
                    label: "stored-anchor".into(),
                    name: anchor,
                    selection: NameSelection::Exact,
                    scope: Scope::Anywhere,
                });
            }
            let mut models = BTreeMap::from([("a".to_owned(), model.clone())]);
            if held
                .iter()
                .any(|held| !matches!(held.scope, Scope::Anywhere))
            {
                let other = reader.resolve_entity(ModelKey::at("ModelKey"), "b".to_owned())?;
                models.insert("b".to_owned(), other);
            }
            let owned = |candidates: &[NameCandidate<Member, _>]| {
                candidates
                    .iter()
                    .map(|candidate| {
                        let owners = candidate.owners().iter().map(|owner| {
                            let key = models.iter().find(|(_, model)| *model == owner);
                            key.map_or_else(|| "?".to_owned(), |(key, _)| key.clone())
                        });
                        (candidate.name(), owners.collect())
                    })
                    .collect::<Vec<(WorthQueryName<Member>, Vec<String>)>>()
            };
            for Held {
                label,
                name,
                selection,
                scope,
            } in held
            {
                let resolved = match &scope {
                    Scope::In(owner) => {
                        reader
                            .reader()
                            .resolve_in(&models[owner], &name, &selection)
                    }
                    Scope::Anywhere | Scope::AnywhereOwned => {
                        reader.reader().resolve(&name, &selection)
                    }
                };
                // A refused admission is observed, and the run still ends
                // denied by the first one once every read is recorded.
                let resolved = match resolved {
                    Ok(resolved) => resolved,
                    Err(denial) => {
                        observed.push(Observation::ResolveRefused {
                            label,
                            refusal: format!("{denial:?}"),
                        });
                        refused.get_or_insert(HandlerExecutionDenial::new(denial));
                        continue;
                    }
                };
                observed.push(match resolved {
                    Ok(ResolvedName::Exact(identity)) => {
                        live.iter().find(|(_, member)| *member == identity).map_or(
                            Observation::ResolvedElsewhere {
                                label: label.clone(),
                            },
                            |(key, _)| Observation::ResolvedTo {
                                label: label.clone(),
                                key: key.clone(),
                            },
                        )
                    }
                    Ok(ResolvedName::Retired { .. }) => Observation::Retired { label },
                    Ok(ResolvedName::Plural(identities)) => Observation::Plural {
                        label,
                        keys: identities
                            .iter()
                            .map(|identity| {
                                live.iter()
                                    .find(|(_, member)| member == identity)
                                    .map_or_else(|| "?".to_owned(), |(key, _)| key.clone())
                            })
                            .collect(),
                    },
                    Err(ExecutionNameResolveDenial::Resolution(
                        NameResolutionDenial::Ambiguous { candidates },
                    )) if matches!(scope, Scope::Anywhere) => Observation::Ambiguous {
                        label,
                        candidates: candidates.iter().map(NameCandidate::name).collect(),
                    },
                    Err(ExecutionNameResolveDenial::Resolution(
                        NameResolutionDenial::Ambiguous { candidates },
                    )) => Observation::AmbiguousOwned {
                        label,
                        candidates: owned(&candidates),
                    },
                    Err(ExecutionNameResolveDenial::Resolution(
                        NameResolutionDenial::NoCandidateInOwner { candidates },
                    )) => Observation::NotInOwner {
                        label,
                        candidates: owned(&candidates),
                    },
                    Err(refusal) => Observation::ResolveRefused {
                        label,
                        refusal: format!("{refusal:?}"),
                    },
                });
            }
            PROBES
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry(input.token.clone())
                .or_default()
                .observed = observed;
            seams::observe(reader, &input.token, &live)?;
            births::observe(reader, &input.token);
            if let Some(refused) = refused {
                return Err(refused);
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
        let result = writer.projected_entity(&model).and_then(|model| {
            writer.write_field(&model, ModelValue::at("ModelValue"), input.value)
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
        .declared_operation::<ProbeNames>(
            ProbeNames::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
