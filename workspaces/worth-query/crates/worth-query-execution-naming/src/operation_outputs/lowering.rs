//! Authoring admission of declared output births and lowering of a bound
//! operation naming program into Relational's `CommitNamingPlan`.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;
use worth_relational::facade::lineage::{
    CommitNamingPlan, LineageRecordTarget, PendingBirthTarget, PendingMergeTarget,
    PersistentNameIdentity, PersistentNameKey, PersistentNameObservationView,
    PersistentNameResolution, PersistentNameSelection, PersistentNameSubjectKind,
    PersistentNameTarget,
};

use super::same_commit::{resolve_same_commit_names, ResolvedOutputName};
use super::{
    BoundOperationNaming, OperationMergeKeeper, OperationNamedOutput, OperationNamingEntry,
    WorthQueryOutputBirth, WorthQueryOutputNameDenial,
};
use crate::birth_plan::{
    ExecutionNameBirthPlanBuilder, ExecutionNamedMergeKeeper, ExecutionPendingEntityBirth,
    ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
use crate::lowering::LoweredExecutionNameSchemeCatalog;

/// Which posture a declared output birth takes part in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputBirthPosture {
    /// A plain birth (`Create`, or `Preserve` regeneration).
    Birth,
    /// A child of a `Split`: always distinguished by a discriminator.
    SplitChild,
    /// A `Merge` survivor minting a new name over its parents: a
    /// split-capable scheme requires a discriminator on every birth that
    /// carries parents.
    MergeSurvivor,
}

/// Admits one declared output birth of `subject_name` against the installed
/// scheme catalog and returns the scheme it is minted under.
///
/// Refuses an unschemed subject, an undeclared role, a missing authored
/// occurrence and a parent-bearing birth without its required discriminator
/// before any effect.
///
/// Complexity: O(log S + log R) for S schemes of R roles.
pub fn admit_output_birth(
    catalog: &LoweredExecutionNameSchemeCatalog,
    subject: ApplicationNameSubjectKind,
    subject_name: &str,
    birth: &WorthQueryOutputBirth,
    posture: OutputBirthPosture,
) -> Result<String, WorthQueryOutputNameDenial> {
    let layout = catalog
        .scheme_for_subject(subject, subject_name)
        .ok_or_else(|| WorthQueryOutputNameDenial::UnschemedOutput {
            subject,
            subject_name: subject_name.to_owned(),
        })?;
    let scheme_id = layout.scheme_id().to_owned();
    let refused = |denial| Err(WorthQueryOutputNameDenial::Assembly(denial));
    if layout.role_rank(birth.role_path()).is_none() {
        return refused(crate::denial::ExecutionNamingDenial::UndeclaredRole {
            scheme_id,
            role_path: birth.role_path().to_owned(),
        });
    }
    if layout.require_authored_occurrence() && birth.occurrence_key().trim().is_empty() {
        return refused(
            crate::denial::ExecutionNamingDenial::MissingAuthoredOccurrenceKey { scheme_id },
        );
    }
    let discriminated = match posture {
        OutputBirthPosture::Birth => false,
        OutputBirthPosture::SplitChild => true,
        OutputBirthPosture::MergeSurvivor => layout.split_capable(),
    };
    if discriminated && !birth.has_discriminator() {
        return refused(
            crate::denial::ExecutionNamingDenial::MissingSplitDiscriminator {
                scheme_id,
                role_path: birth.role_path().to_owned(),
            },
        );
    }
    Ok(scheme_id)
}

/// A held name's identity and the one live record it designates.
type HeldName = (PersistentNameIdentity, LineageRecordTarget);

impl BoundOperationNaming {
    /// Lowers the program against the commit basis's derived name index.
    ///
    /// Every held name must be born at the basis and designate exactly one
    /// live record there: that record is the subject of its retirement,
    /// split or merge, so the record and the name can never disagree.
    /// Discriminator references resolve to identities born at the basis, or
    /// to the identity of an earlier-role output this commit births in the
    /// same occurrence. Relational then validates every posture against the
    /// commit's performed effects and mints every birth.
    ///
    /// Complexity: O(E log E + N) for E named outputs and N held names, each
    /// held name one O(1) expected index probe plus its O(1) live check, and
    /// O(R + B) per output of R same-commit references and B discriminator
    /// bytes.
    pub fn lower(
        &self,
        catalog: &LoweredExecutionNameSchemeCatalog,
        view: PersistentNameObservationView<'_>,
    ) -> Result<CommitNamingPlan, WorthQueryOutputNameDenial> {
        let mut held = BTreeMap::new();
        let mut identities = Vec::new();
        for entry in self.entries() {
            for (subject, digest) in held_names(entry) {
                let name = resolve_held(&view, subject, digest)?;
                identities.push(name.0);
                held.insert(digest, name);
            }
            for output in entry.named_outputs() {
                for digest in &output.birth.embedded_name_digests {
                    if let Some(identity) = recorded_identity(&view, digest) {
                        identities.push(identity);
                    }
                }
            }
        }
        let outputs = self.entries().iter().flat_map(|entry| {
            let parents = parent_identities(entry, &held);
            let outputs = entry.named_outputs().iter();
            outputs.map(move |output| (output, parents.clone()))
        });
        let resolved = resolve_same_commit_names(catalog, self.candidate(), outputs)?;
        identities.extend(resolved.iter().map(|name| name.identity));
        let same_commit: Vec<_> = resolved.iter().map(|name| name.identity).collect();
        // Consumed in the declaration order the outputs were resolved in.
        let mut resolved = resolved.into_iter();
        let mut next = || resolved.next().expect("one resolved name per named output");
        let mut plan = ExecutionNameBirthPlanBuilder::new(catalog, identities);
        let assembly = WorthQueryOutputNameDenial::Assembly;
        for entry in self.entries() {
            match entry {
                OperationNamingEntry::Birth(output) => add_birth(&mut plan, output, next())?,
                OperationNamingEntry::Retire { name, .. } => {
                    let (identity, target) = held[name];
                    match target {
                        LineageRecordTarget::Entity(id) => plan.add_retire_entity(id, identity),
                        LineageRecordTarget::Relation(id) => plan.add_retire_relation(id, identity),
                    };
                }
                OperationNamingEntry::Split {
                    parent, children, ..
                } => {
                    let (identity, target) = held[parent];
                    let children = children
                        .iter()
                        .map(|child| split_child(child, next()))
                        .collect();
                    plan.add_split_posture(target, identity, children)
                        .map_err(assembly)?;
                }
                OperationNamingEntry::Merge {
                    parents, keeper, ..
                } => {
                    let parents = parents.iter().map(|p| (held[p].1, held[p].0)).collect();
                    let (target, keeper) = match keeper {
                        OperationMergeKeeper::KeepParentInPlace(name) => (
                            PendingMergeTarget::Existing(held[name].1),
                            ExecutionNamedMergeKeeper::KeepParent(held[name].0),
                        ),
                        OperationMergeKeeper::KeepParentOnto { keeper, target } => (
                            PendingMergeTarget::Created(target.clone()),
                            ExecutionNamedMergeKeeper::KeepParent(held[keeper].0),
                        ),
                        OperationMergeKeeper::MintNew(output) => {
                            let name = next();
                            (
                                PendingMergeTarget::Created(output.target.clone()),
                                ExecutionNamedMergeKeeper::MintNew {
                                    scheme_id: output.scheme_id.clone(),
                                    occurrence_key: output.birth.occurrence_key.clone(),
                                    role_path: output.birth.role_path.clone(),
                                    canonical_discriminator: name.canonical_discriminator,
                                    embedded_name_digests: name.embedded_name_digests,
                                    same_occurrence_roles: name.same_occurrence_roles,
                                },
                            )
                        }
                    };
                    plan.add_merge_posture(parents, target, keeper)
                        .map_err(assembly)?;
                }
                OperationNamingEntry::GroupPartition { .. } => {}
            }
        }
        let mut lowered = plan.build().into_relational_plan();
        super::group_lowering::lower_group_partitions(
            self.entries(),
            catalog,
            &view,
            self.candidate(),
            &same_commit,
            &mut lowered,
        )?;
        Ok(lowered)
    }
}

fn held_names(
    entry: &OperationNamingEntry<PendingBirthTarget>,
) -> Vec<(ApplicationNameSubjectKind, [u8; 32])> {
    match entry {
        OperationNamingEntry::Birth(_) | OperationNamingEntry::GroupPartition { .. } => Vec::new(),
        OperationNamingEntry::Retire { subject, name } => vec![(*subject, *name)],
        OperationNamingEntry::Split {
            subject, parent, ..
        } => vec![(*subject, *parent)],
        OperationNamingEntry::Merge {
            subject,
            parents,
            keeper,
        } => {
            let mut names: Vec<_> = parents.iter().map(|p| (*subject, *p)).collect();
            if let OperationMergeKeeper::KeepParentInPlace(name)
            | OperationMergeKeeper::KeepParentOnto { keeper: name, .. } = keeper
            {
                names.push((*subject, *name));
            }
            names
        }
    }
}

/// The parent identities every output `entry` names is born over: none for
/// a plain birth, the split parent, or every merge parent.
fn parent_identities(
    entry: &OperationNamingEntry<PendingBirthTarget>,
    held: &BTreeMap<[u8; 32], HeldName>,
) -> Vec<PersistentNameIdentity> {
    match entry {
        OperationNamingEntry::Birth(_)
        | OperationNamingEntry::Retire { .. }
        | OperationNamingEntry::GroupPartition { .. } => Vec::new(),
        OperationNamingEntry::Split { parent, .. } => vec![held[parent].0],
        OperationNamingEntry::Merge { parents, .. } => {
            parents.iter().map(|parent| held[parent].0).collect()
        }
    }
}

fn persistent_subject(subject: ApplicationNameSubjectKind) -> PersistentNameSubjectKind {
    match subject {
        ApplicationNameSubjectKind::Entity => PersistentNameSubjectKind::Entity,
        ApplicationNameSubjectKind::Relation => PersistentNameSubjectKind::Relation,
        ApplicationNameSubjectKind::Group => PersistentNameSubjectKind::Group,
    }
}

fn key_identity(key: PersistentNameKey) -> Option<PersistentNameIdentity> {
    match key {
        PersistentNameKey::Entity(identity) | PersistentNameKey::Relation(identity) => {
            Some(identity)
        }
        PersistentNameKey::Group(_) => None,
    }
}

fn recorded_identity(
    view: &PersistentNameObservationView<'_>,
    digest: &[u8; 32],
) -> Option<PersistentNameIdentity> {
    [
        PersistentNameSubjectKind::Entity,
        PersistentNameSubjectKind::Relation,
    ]
    .into_iter()
    .find_map(|subject| view.recorded_name(subject, digest).and_then(key_identity))
}

fn resolve_held(
    view: &PersistentNameObservationView<'_>,
    subject: ApplicationNameSubjectKind,
    digest: [u8; 32],
) -> Result<HeldName, WorthQueryOutputNameDenial> {
    let key = view
        .recorded_name(persistent_subject(subject), &digest)
        .ok_or(WorthQueryOutputNameDenial::UnknownHeldName { subject, digest })?;
    let identity =
        key_identity(key).ok_or(WorthQueryOutputNameDenial::UnknownHeldName { subject, digest })?;
    let not_live = WorthQueryOutputNameDenial::HeldNameNotLive { subject, digest };
    match view.resolve(key, &PersistentNameSelection::Exact).result {
        Ok(PersistentNameResolution::Exact(target)) if target.key() == key => match target {
            PersistentNameTarget::Entity { entity, .. } => {
                Ok((identity, LineageRecordTarget::Entity(entity)))
            }
            PersistentNameTarget::Relation { relation, .. } => {
                Ok((identity, LineageRecordTarget::Relation(relation)))
            }
            PersistentNameTarget::Group { .. } => Err(not_live),
        },
        _ => Err(not_live),
    }
}

fn add_birth(
    plan: &mut ExecutionNameBirthPlanBuilder<'_>,
    output: &OperationNamedOutput<PendingBirthTarget>,
    name: ResolvedOutputName,
) -> Result<(), WorthQueryOutputNameDenial> {
    let birth = &output.birth;
    let result = match &output.target {
        PendingBirthTarget::CreatedEntity(created) => {
            let mut pending = ExecutionPendingEntityBirth::new(
                created.clone(),
                output.scheme_id.clone(),
                birth.occurrence_key.clone(),
                birth.role_path.clone(),
            );
            pending.canonical_discriminator = name.canonical_discriminator;
            pending.embedded_name_digests = name.embedded_name_digests;
            pending.same_occurrence_roles = name.same_occurrence_roles;
            plan.add_entity_birth(pending).map(|_| ())
        }
        PendingBirthTarget::CreatedRelation(created) => {
            let mut pending = ExecutionPendingRelationBirth::new(
                created.clone(),
                output.scheme_id.clone(),
                birth.occurrence_key.clone(),
                birth.role_path.clone(),
            );
            pending.canonical_discriminator = name.canonical_discriminator;
            pending.embedded_name_digests = name.embedded_name_digests;
            pending.same_occurrence_roles = name.same_occurrence_roles;
            plan.add_relation_birth(pending).map(|_| ())
        }
    };
    result.map_err(WorthQueryOutputNameDenial::Assembly)
}

fn split_child(
    output: &OperationNamedOutput<PendingBirthTarget>,
    name: ResolvedOutputName,
) -> ExecutionSplitChildSpec {
    ExecutionSplitChildSpec {
        target: output.target.clone(),
        scheme_id: output.scheme_id.clone(),
        occurrence_key: output.birth.occurrence_key.clone(),
        role_path: output.birth.role_path.clone(),
        canonical_discriminator: name.canonical_discriminator,
        embedded_name_digests: name.embedded_name_digests,
        same_occurrence_roles: name.same_occurrence_roles,
    }
}
