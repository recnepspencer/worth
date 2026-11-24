//! Executes one authored plan through the candidate writer.
use std::collections::BTreeMap;

use super::super::super::*;
use super::{Binding, Held, HeldSeam, Keeper, Plan, SeamKeeper, Step};
use worth_query_host::facade::application_naming::{WorthQueryName, WorthQueryOutputMergeKeeper};

pub(super) fn execute(
    plan: &Plan,
    names: &BTreeMap<String, WorthQueryName<Member>>,
    seam_names: &BTreeMap<String, WorthQueryName<Seam>>,
    model: &WorthQueryInvariantMutationTarget<Schema, Model>,
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let held = |held: &Held| match held {
        Held::Read(key) => *names.get(key).expect("decision read the held member"),
        Held::Name(name) => *name,
    };
    let held_seam = |held: &HeldSeam| match held {
        HeldSeam::Read(label) => *seam_names.get(label).expect("decision read the held seam"),
        HeldSeam::Name(name) => *name,
    };
    let context = writer.projected_entity(model)?;
    let mut members = BTreeMap::new();
    for key in &plan.creates {
        let member_key = WorthQueryApplicationEntityKey::new(key).unwrap();
        let created = if plan.issued {
            writer.create_entity(Member::reference(), member_key)?
        } else {
            writer.create_entity_in_context(&context, Member::reference(), member_key)?
        };
        writer.initialize_field(&created, MemberKey::at("MemberKey"), key.clone())?;
        writer.initialize_field(&created, MemberValue::at("MemberValue"), 7)?;
        // An issued member stays unowned: owning links may not cross
        // creation contexts.
        let named_owner = plan.owners.iter().find(|(member, _)| member == key);
        let named_owner = match named_owner {
            Some((_, owner)) if !plan.issued => {
                Some(writer.resolve_entity(ModelKey::at("ModelKey"), owner.clone())?)
            }
            _ => None,
        };
        if !plan.issued {
            writer.link(
                ModelMember::reference(),
                format!("owns:{key}"),
                named_owner.as_ref().unwrap_or(&context),
                &created,
            )?;
        }
        members.insert(key.clone(), created);
    }
    for key in &plan.reads {
        let existing = writer.resolve_entity(MemberKey::at("MemberKey"), key.clone())?;
        members.entry(key.clone()).or_insert(existing);
    }
    for (key, from, to) in &plan.moves {
        let from = writer.resolve_entity(ModelKey::at("ModelKey"), from.clone())?;
        let to = writer.resolve_entity(ModelKey::at("ModelKey"), to.clone())?;
        writer.unlink(ModelMember::reference(), &from, &members[key])?;
        writer.link(
            ModelMember::reference(),
            format!("moved:{key}"),
            &to,
            &members[key],
        )?;
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
            Step::BirthCutBy {
                key,
                birth,
                side,
                of,
            } => {
                let cutter = Some(writer.same_commit_entity_name(&members[of])?);
                let cut = outputs::FreshMemberCut::Side {
                    side: *side,
                    cutter,
                };
                writer.name_created_entity(&members[key], birth.clone().with_discriminator(&cut))?
            }
            Step::BirthAlong {
                key,
                birth,
                side,
                seam,
            } => {
                let seam = writer.same_commit_relation_name(&seams[seam])?;
                let cut = outputs::CutAlong { side: *side, seam };
                writer.name_created_entity(&members[key], birth.clone().with_discriminator(&cut))?
            }
            Step::BirthSided {
                key,
                birth,
                side,
                of,
                along,
            } => {
                let along = match along {
                    Some(seam) => Some(writer.same_commit_relation_name(&seams[seam])?.into()),
                    None => None,
                };
                let cut = outputs::SidedCut::By {
                    cutter: writer.same_commit_entity_name(&members[of])?.into(),
                    side: *side,
                    along,
                };
                writer.name_created_entity(&members[key], birth.clone().with_discriminator(&cut))?
            }
            Step::SeamBirthOf {
                key,
                birth,
                index,
                of,
            } => {
                let member = writer.same_commit_entity_name(&members[of])?;
                let of = outputs::SeamOf {
                    index: *index,
                    member,
                };
                writer.name_created_relation(&seams[key], birth.clone().with_discriminator(&of))?
            }
            Step::SeamRetire(name) => writer.retire_relation_name(held_seam(name))?,
            Step::SeamSplit(parent, children) => {
                let children = children
                    .iter()
                    .map(|(key, birth)| (&seams[key], birth.clone()))
                    .collect();
                writer.split_relation_name(held_seam(parent), children)?
            }
            Step::SeamMerge(parents, keeper) => {
                let parents: Vec<_> = parents.iter().map(held_seam).collect();
                let keeper = match keeper {
                    SeamKeeper::InPlace(name) => {
                        WorthQueryOutputMergeKeeper::KeepParentInPlace(held_seam(name))
                    }
                    SeamKeeper::Onto(name, key) => WorthQueryOutputMergeKeeper::KeepParentOnto {
                        keeper: held_seam(name),
                        target: &seams[key],
                    },
                    SeamKeeper::MintNew(key, birth) => WorthQueryOutputMergeKeeper::MintNew {
                        target: &seams[key],
                        birth: birth.clone(),
                    },
                };
                writer.merge_relation_names(&parents, keeper)?
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
