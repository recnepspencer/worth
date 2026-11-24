//! The one sculpt plan a single naming commit runs, and the keys it creates
//! with the model names they will hold: shared by direct commits and the
//! first stage of a candidate envelope.
use super::{authored, place, Driver};
use crate::oracle::persistent_naming_dst::ops::Op;
use crate::persistent_naming::outputs::sculpt::{Keeper, Plan, ScopeStep, Step};
use crate::persistent_naming::outputs::{birth, piece, read, MemberCut};
use worth_query_host::facade::application_naming::OccurrencePath;

impl Driver {
    pub(super) fn plan(
        &mut self,
        op: &Op,
        step: usize,
    ) -> Result<(Plan, Vec<(String, usize)>), String> {
        let mut plan = Plan::default();
        let mut created = Vec::new();
        match op {
            Op::Create { name, at } | Op::Restore { name, at } => {
                let key = self.fresh_key();
                place(&mut plan, &[(key.clone(), *at)]);
                plan.steps.push(Step::Birth(
                    key.clone(),
                    birth(&authored(*name), "authored"),
                ));
                created.push((key, *name));
            }
            Op::Preserve { name, at } => {
                let old = self.key_of(*name);
                let key = self.fresh_key();
                place(&mut plan, &[(key.clone(), *at)]);
                plan.reads.push(old.clone());
                plan.deletes.push(old.clone());
                plan.steps.push(Step::Retire(read(&old)));
                plan.steps.push(Step::Birth(
                    key.clone(),
                    birth(&authored(*name), "authored"),
                ));
                created.push((key, *name));
            }
            Op::Split { parent, pieces, at } => {
                let old = self.key_of(*parent);
                let keys = [self.fresh_key(), self.fresh_key()];
                place(
                    &mut plan,
                    &[(keys[0].clone(), at[0]), (keys[1].clone(), at[1])],
                );
                plan.reads.push(old.clone());
                plan.deletes.push(old.clone());
                let occurrence = format!("dst-split-{step}");
                let children = (0..2)
                    .map(|i| (keys[i].clone(), piece(&occurrence, i as u64 + 1, None)))
                    .collect();
                plan.steps.push(Step::Split(read(&old), children));
                created.extend([(keys[0].clone(), pieces[0]), (keys[1].clone(), pieces[1])]);
            }
            Op::MergeKeep {
                keeper,
                other,
                listed,
            } => {
                // The keeper is held by name only: this commit never stages it.
                let other_key = self.key_of(*other);
                plan.reads.push(other_key.clone());
                plan.deletes.push(other_key.clone());
                let held = |name: usize| {
                    if name == *keeper {
                        self.held(*keeper)
                    } else {
                        read(&other_key)
                    }
                };
                plan.steps.push(match listed {
                    None => Step::Merge(
                        vec![held(*keeper), held(*other)],
                        Keeper::InPlace(self.held(*keeper)),
                    ),
                    Some(listed) => Step::Scoped(
                        OccurrencePath::root(&format!("dst-least-{step}")).unwrap(),
                        vec![ScopeStep::MergeLeast(listed.map(held).to_vec(), None)],
                    ),
                });
            }
            Op::MergeMint {
                parents,
                minted,
                at,
            } => {
                let old = parents.map(|name| self.key_of(name));
                let key = self.fresh_key();
                place(&mut plan, &[(key.clone(), *at)]);
                plan.reads.extend(old.iter().cloned());
                plan.deletes.extend(old.iter().cloned());
                let cut = MemberCut::Side {
                    side: 0,
                    cutter: None,
                };
                let minting =
                    birth(&format!("dst-merge-{step}"), "merged").with_discriminator(&cut);
                plan.steps.push(Step::Merge(
                    old.iter().map(|key| read(key)).collect(),
                    Keeper::MintNew(key.clone(), minting),
                ));
                created.push((key, *minted));
            }
            Op::Retire { name } => {
                let old = self.key_of(*name);
                plan.reads.push(old.clone());
                plan.deletes.push(old.clone());
                plan.steps.push(Step::Retire(read(&old)));
            }
            other => return Err(format!("not a single naming commit: {other:?}")),
        }
        Ok((plan, created))
    }
}
