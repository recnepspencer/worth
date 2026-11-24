//! Executes plan steps through a naming scope (`writer.naming_scope`), the
//! occurrence bound once: the counterpart of the manual steps the rest of
//! the plan declares through the candidate writer.
use std::collections::BTreeMap;

use super::super::super::*;
use super::{Binding, Held, HeldSeam};
use worth_query_host::facade::application_naming::{
    NameRef, OccurrencePath, WorthQueryLeastMergeKeeper, WorthQueryName,
};

type Members = BTreeMap<String, WorthQueryApplicationEffectEntity<Schema, Member>>;
type Seams = BTreeMap<String, WorthQueryApplicationCreatedRelation<Schema, Seam, Member, Member>>;

/// One declaration through a naming scope.
#[derive(Clone, Debug)]
pub enum ScopeStep {
    /// Names created member `key` under `role`.
    Name(String, MemberRole),
    /// Names created member `key` the `side` piece cut by created member
    /// `of`, referenced by its same-commit name (`NameRef::InCommit`).
    CutBy {
        key: String,
        side: outputs::CutSide,
        of: String,
    },
    /// Links seam `key` from `from` to `to` and names it in one call, told
    /// apart by created member `of`'s same-commit name.
    LinkSeamOf {
        key: String,
        from: String,
        to: String,
        index: u64,
        of: String,
    },
    /// Declares `steps` in the child occurrence `segment`.
    Child(String, Vec<ScopeStep>),
    /// Declares `steps` in the indexed child occurrence `name[index]`.
    Index(String, u32, Vec<ScopeStep>),
    /// Merges held members keeping the least; onto created member `onto`
    /// when given, else in place.
    MergeLeast(Vec<Held>, Option<String>),
    /// Merges held seams keeping the least.
    SeamMergeLeast(Vec<HeldSeam>, Option<String>),
}

/// What the plan already created and read, for scoped steps to act on.
pub(super) struct Scoped<'p> {
    pub members: &'p Members,
    pub seams: &'p mut Seams,
    pub names: &'p dyn Fn(&Held) -> WorthQueryName<Member>,
    pub seam_names: &'p dyn Fn(&HeldSeam) -> WorthQueryName<Seam>,
}

pub(super) fn execute(
    occurrence: &OccurrencePath,
    steps: &[ScopeStep],
    plan: &mut Scoped<'_>,
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let mut scope = writer.naming_scope(occurrence.clone());
    declare(&mut scope, steps, plan)
}

fn declare(
    scope: &mut NamingScope<'_, '_, Schema, Binding>,
    steps: &[ScopeStep],
    plan: &mut Scoped<'_>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    for step in steps {
        match step {
            ScopeStep::Name(key, role) => {
                let birth = scope.birth(*role);
                scope.name_entity(&plan.members[key], birth)?
            }
            ScopeStep::CutBy { key, side, of } => {
                let cut = outputs::SidedCut::By {
                    cutter: scope.same_commit_entity_name(&plan.members[of])?.into(),
                    side: *side,
                    along: None::<NameRef<Seam>>,
                };
                let birth = scope.birth(MemberRole::Piece).with_discriminator(&cut);
                scope.name_entity(&plan.members[key], birth)?
            }
            ScopeStep::LinkSeamOf {
                key,
                from,
                to,
                index,
                of,
            } => {
                let member = scope.same_commit_entity_name(&plan.members[of])?;
                let birth = scope
                    .birth(SeamRole::Seam)
                    .with_discriminator(&outputs::SeamOf {
                        index: *index,
                        member,
                    });
                let seam = scope.link_named(
                    Seam::reference(),
                    key.clone(),
                    &plan.members[from],
                    &plan.members[to],
                    birth,
                )?;
                plan.seams.insert(key.clone(), seam);
            }
            ScopeStep::Child(segment, steps) => {
                let mut child = scope.sub_scope(segment).expect("a valid segment");
                declare(&mut child, steps, plan)?
            }
            ScopeStep::Index(name, index, steps) => {
                let mut child = scope.index(name, *index).expect("a valid segment");
                declare(&mut child, steps, plan)?
            }
            ScopeStep::MergeLeast(parents, onto) => {
                let parents: Vec<_> = parents.iter().map(plan.names).collect();
                let keeper = match onto {
                    Some(key) => WorthQueryLeastMergeKeeper::Onto(&plan.members[key]),
                    None => WorthQueryLeastMergeKeeper::InPlace,
                };
                scope.merge_least_entity(&parents, keeper)?
            }
            ScopeStep::SeamMergeLeast(parents, onto) => {
                let parents: Vec<_> = parents.iter().map(plan.seam_names).collect();
                let keeper = match onto {
                    Some(key) => WorthQueryLeastMergeKeeper::Onto(&plan.seams[key]),
                    None => WorthQueryLeastMergeKeeper::InPlace,
                };
                scope.merge_least_relation(&parents, keeper)?
            }
        }
    }
    Ok(())
}
