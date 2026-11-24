//! Drives one seed's actions through real installed commits: every action
//! is one `sculpt` operation whose candidate writer creates, deletes and
//! declares persistent-name postures, exactly as an application would.
use std::collections::{BTreeMap, BTreeSet};

use super::ops::{Op, Placement};

mod coupled;
use crate::persistent_naming::outputs::sculpt::{Held, Keeper, Plan, ScopeStep, Step};
use crate::persistent_naming::outputs::{
    birth, naming_refusal, piece, read, MemberCut, Outcome, World,
};
use crate::persistent_naming::{world, Member, MemberRole, MEMBER_SCHEME};
use worth_query_host::facade::application_naming::{
    LineageNameDenial, LineageRecordTarget, OccurrencePath, WorthQueryName,
};

/// A record's native identity `(partition, slot, generation)`.
pub type RecordIdentity = (u32, u64, u32);

/// The live installed world one seed drives, and what the harness knows of it.
pub struct Driver {
    pub world: World,
    seed: u64,
    generation: usize,
    next_key: usize,
    /// The live record key each live model name is held by.
    pub keys: BTreeMap<usize, String>,
    /// The admitted name each model name stands for.
    pub names: BTreeMap<usize, WorthQueryName<Member>>,
    /// Every record key ever created, live or not.
    pub created: Vec<String>,
    /// Committed commits, in order, each its stages' plans (one for a
    /// direct operation): the canonical authored history.
    committed: Vec<Vec<Plan>>,
    /// Every native record identity this history ever held.
    seen: BTreeSet<RecordIdentity>,
    /// Model `a`'s context partition, where owned records are created.
    context_partition: Option<u32>,
    pub naming_commits: usize,
}

/// The authored occurrence of a model name's first birth.
fn authored(name: usize) -> String {
    format!("dst-n{}", name + 1)
}

fn place(plan: &mut Plan, at: &[(String, Placement)]) {
    for (key, placement) in at {
        match placement {
            Placement::Issued => plan.issued = true,
            Placement::Owned("b") => plan.owners.push((key.clone(), "b".to_owned())),
            Placement::Owned(_) => {}
        }
        plan.creates.push(key.clone());
    }
}

impl Driver {
    pub fn new(seed: u64) -> Self {
        Self {
            world: World::install(),
            seed,
            generation: 0,
            next_key: 0,
            keys: BTreeMap::new(),
            names: BTreeMap::new(),
            created: Vec::new(),
            committed: Vec::new(),
            seen: BTreeSet::new(),
            context_partition: None,
            naming_commits: 0,
        }
    }

    pub fn token(&self, step: usize, role: &str) -> String {
        format!("dst-{:016x}-{}-{role}-{step}", self.seed, self.generation)
    }

    fn fresh_key(&mut self) -> String {
        self.next_key += 1;
        let key = format!("r{}", self.next_key);
        self.created.push(key.clone());
        key
    }

    fn key_of(&self, name: usize) -> String {
        self.keys[&name].clone()
    }

    fn held(&self, name: usize) -> Held {
        Held::Name(self.names[&name])
    }

    /// Runs `op` as one installed operation and returns the keys it created
    /// with the model names they will hold. A refusal the op requires is
    /// checked here; any other refusal fails the seed.
    pub fn execute(&mut self, op: &Op, step: usize) -> Result<Vec<(String, usize)>, String> {
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
            Op::Duplicate { name } => return self.duplicate(*name, step),
            Op::Unnamed { at, pair } => return self.unnamed(*at, *pair, step),
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
            Op::MergeLeast { .. } => return Err("a least merge must be settled".to_owned()),
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
            Op::Rebuild => {
                self.rebuild()?;
                return Ok(Vec::new());
            }
            Op::Coupled { names, at } => return self.coupled(*names, *at, step),
            Op::CoupledDuplicate { name, at } => return self.coupled_duplicate(*name, *at, step),
        }
        self.commit(step, plan)?;
        for (key, placement) in created.iter().zip(placements(op)) {
            self.admit_record(&key.0, placement)?;
        }
        Ok(created)
    }

    fn commit(&mut self, step: usize, plan: Plan) -> Result<(), String> {
        let token = self.token(step, "op");
        let outcome = self.world.try_sculpt(&token, plan.clone())?;
        if !matches!(outcome, Outcome::Committed { .. }) {
            return Err(format!("step {step} must commit: {outcome:?}"));
        }
        self.committed.push(vec![plan]);
        self.naming_commits += 1;
        Ok(())
    }

    /// Settles a least-keeper merge: the harness, not the world, decides
    /// which parent name is least (`WorthQueryName`'s order), so the world's
    /// `merge_least` must keep exactly that one.
    pub fn settle(&self, op: Op) -> Op {
        match op {
            Op::MergeLeast { parents } => {
                let [keeper, other] = if self.names[&parents[0]] < self.names[&parents[1]] {
                    parents
                } else {
                    [parents[1], parents[0]]
                };
                Op::MergeKeep {
                    keeper,
                    other,
                    listed: Some(parents),
                }
            }
            op => op,
        }
    }

    /// Re-minting a live name's exact birth refuses `DuplicateBirth` (and
    /// nothing else) with zero effects.
    fn duplicate(&mut self, name: usize, step: usize) -> Result<Vec<(String, usize)>, String> {
        let key = self.fresh_key();
        let mut plan = Plan::default();
        place(&mut plan, &[(key.clone(), Placement::Owned("a"))]);
        plan.steps
            .push(Step::Birth(key.clone(), birth(&authored(name), "authored")));
        let outcome = self.world.try_sculpt(&self.token(step, "dup"), plan)?;
        let typed = matches!(
            naming_refusal(&outcome),
            Some(LineageNameDenial::DuplicateBirth { .. })
        );
        if !typed {
            return Err(format!(
                "a live re-mint must refuse DuplicateBirth: {outcome:?}"
            ));
        }
        if self.world.live(&key) {
            return Err("a refused re-mint must leave zero effects".to_owned());
        }
        Ok(Vec::new())
    }

    /// A commit creating a record of the schemed kind it leaves unnamed
    /// (beside a named sibling when `pair`) refuses `MissingBirth` for the
    /// member scheme with zero effects, the named sibling included.
    fn unnamed(
        &mut self,
        at: Placement,
        pair: bool,
        step: usize,
    ) -> Result<Vec<(String, usize)>, String> {
        let keys: Vec<String> = (0..1 + usize::from(pair))
            .map(|_| self.fresh_key())
            .collect();
        let mut plan = Plan::default();
        place(
            &mut plan,
            &keys.iter().map(|key| (key.clone(), at)).collect::<Vec<_>>(),
        );
        if pair {
            plan.steps.push(Step::Scoped(
                OccurrencePath::root(&format!("dst-unnamed-{step}")).unwrap(),
                vec![ScopeStep::Name(keys[0].clone(), MemberRole::Authored)],
            ));
        }
        let outcome = self.world.try_sculpt(&self.token(step, "unnamed"), plan)?;
        match naming_refusal(&outcome) {
            Some(LineageNameDenial::MissingBirth {
                target: LineageRecordTarget::Entity(_),
                scheme_id,
            }) if scheme_id == MEMBER_SCHEME => {}
            _ => {
                return Err(format!(
                    "an unnamed birth must refuse MissingBirth: {outcome:?}"
                ))
            }
        }
        if keys.iter().any(|key| self.world.live(key)) {
            return Err("a refused unnamed birth must leave zero effects".to_owned());
        }
        Ok(Vec::new())
    }

    /// Checks a created record's partition (model `a`'s context, or one the
    /// commit issued outside it) and that its native identity is one this
    /// history never held before (a fresh generation).
    fn admit_record(&mut self, key: &str, at: Placement) -> Result<(), String> {
        let identity = self
            .identity(key)
            .ok_or_else(|| format!("created `{key}` must be live"))?;
        let context = match self.context_partition {
            Some(context) => context,
            None => *self
                .context_partition
                .insert(world_context_partition(self)?),
        };
        let placed = match at {
            Placement::Owned(_) => identity.0 == context,
            Placement::Issued => identity.0 != context,
        };
        if !placed {
            return Err(format!(
                "`{key}` ({at:?}) landed in partition {} (context {context})",
                identity.0
            ));
        }
        if !self.seen.insert(identity) {
            return Err(format!("`{key}` reuses native identity {identity:?}"));
        }
        Ok(())
    }

    fn identity(&self, key: &str) -> Option<RecordIdentity> {
        world::member_identity(&self.world.host, self.world.host.current_world(), key)
    }

    pub fn partitions_differ(&self, first: usize, second: usize) -> Option<bool> {
        let partition = |name: usize| self.identity(&self.keys[&name]).map(|identity| identity.0);
        Some(partition(first)? != partition(second)?)
    }

    /// Destroys the derived name index: a fresh installed world, whose index
    /// starts empty, receives the same canonical authored commits. Native
    /// slots need not repeat (a refused re-mint consumed some), so the seen
    /// identities are re-derived from the replayed commits.
    fn rebuild(&mut self) -> Result<(), String> {
        self.generation += 1;
        self.world = World::install();
        self.seen.clear();
        for (ordinal, plans) in self.committed.clone().into_iter().enumerate() {
            self.replay(ordinal, &plans)?;
            for key in plans.iter().flat_map(|plan| &plan.creates) {
                if let Some(identity) = self.identity(key) {
                    self.seen.insert(identity);
                }
            }
        }
        Ok(())
    }
}

/// Model `a`'s context partition: where the seeded member `direct` lives.
fn world_context_partition(driver: &Driver) -> Result<u32, String> {
    driver
        .identity("direct")
        .map(|identity| identity.0)
        .ok_or_else(|| "the seeded member `direct` must be live".to_owned())
}

fn placements(op: &Op) -> Vec<Placement> {
    match op {
        Op::Create { at, .. }
        | Op::Restore { at, .. }
        | Op::Preserve { at, .. }
        | Op::MergeMint { at, .. } => vec![*at],
        Op::Split { at, .. } | Op::Coupled { at, .. } => at.to_vec(),
        Op::Duplicate { .. }
        | Op::Unnamed { .. }
        | Op::MergeKeep { .. }
        | Op::MergeLeast { .. }
        | Op::Retire { .. }
        | Op::Rebuild
        | Op::CoupledDuplicate { .. } => Vec::new(),
    }
}
