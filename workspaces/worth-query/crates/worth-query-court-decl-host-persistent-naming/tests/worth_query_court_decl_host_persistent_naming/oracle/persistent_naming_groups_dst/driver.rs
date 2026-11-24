//! Drives one seed's group actions through real installed commits: every
//! action is one `sculpt` operation whose candidate writer declares loop
//! partitions and deletes members, and every observation is one decision
//! making tracked `group_name_of` / `resolve_group` reads.
use std::collections::{BTreeMap, BTreeSet};

use super::candidate::{First, Read};
use super::model::{Answer, Model, Op};
use crate::persistent_naming::coupled::candidate::{edited, sculpted};
use crate::persistent_naming::groups::{group, loops, seeded};
use crate::persistent_naming::outputs::sculpt::{
    GroupAnswer, GroupName, GroupRead, Held, Plan, Step,
};
use crate::persistent_naming::outputs::{keys, naming_refusal, Outcome, World};
use crate::persistent_naming::{GroupRole, Loop, Member};
use worth_query_host::facade::application_naming::{NameSelection, WorthQueryName};

/// What one observation answered, in model labels.
pub struct Observed {
    pub owners: BTreeMap<usize, Option<usize>>,
    pub answers: BTreeMap<(usize, bool), Answer>,
}

pub struct Driver {
    world: World,
    seed: u64,
    step: usize,
    pub members: Vec<WorthQueryName<Member>>,
    keys: Vec<String>,
    labels: BTreeMap<WorthQueryName<Loop>, usize>,
    names: BTreeMap<usize, WorthQueryName<Loop>>,
}

impl Driver {
    pub fn new(seed: u64, members: usize) -> Self {
        let keys: Vec<String> = (0..members).map(|m| format!("m{m}")).collect();
        let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
        let (world, members) = seeded(&format!("groups-dst-{seed}"), &refs);
        Self {
            world,
            seed,
            step: 0,
            members,
            keys,
            labels: BTreeMap::new(),
            names: BTreeMap::new(),
        }
    }

    fn token(&mut self, what: &str) -> String {
        self.step += 1;
        format!("groups-dst-{}-{}-{what}", self.seed, self.step)
    }

    fn roles(
        &self,
        members: &BTreeSet<usize>,
        host: usize,
    ) -> Vec<(GroupRole, WorthQueryName<Member>)> {
        let role = |m: usize| {
            if m == host {
                GroupRole::Host
            } else {
                GroupRole::Member
            }
        };
        members
            .iter()
            .map(|m| (role(*m), self.members[*m]))
            .collect()
    }

    fn deleting(&self, m: usize, steps: Vec<Step>) -> Plan {
        Plan {
            reads: keys(&[&self.keys[m]]),
            deletes: keys(&[&self.keys[m]]),
            steps,
            ..Plan::default()
        }
    }

    /// The plan `op` commits (or must refuse), given the model before it.
    fn plan(&self, op: &Op, model: &Model) -> Plan {
        let declare = |decl| Plan {
            steps: vec![Step::Groups(decl)],
            ..Plan::default()
        };
        match op {
            Op::Born {
                members, anchor, ..
            } => declare(loops(vec![group(&self.roles(members, *anchor))], &[])),
            Op::Revive { g, members } => {
                let anchor = model.groups[g].anchor;
                declare(loops(vec![group(&self.roles(members, anchor))], &[]))
            }
            Op::Split { piece, anchor, .. } => {
                declare(loops(vec![group(&self.roles(piece, *anchor))], &[]))
            }
            Op::SplitAway {
                g, parts, anchors, ..
            } => {
                let decl = loops(
                    vec![
                        group(&self.roles(&parts[0], anchors[0])),
                        group(&self.roles(&parts[1], anchors[1])),
                    ],
                    &[],
                );
                self.deleting(model.groups[g].anchor, vec![Step::Groups(decl)])
            }
            Op::Merge { keep, absorb } => {
                let mut all = model.groups[keep].members.clone();
                all.extend(model.groups[absorb].members.iter().copied());
                declare(loops(
                    vec![group(&self.roles(&all, model.groups[keep].anchor))],
                    &[],
                ))
            }
            Op::Dissolve { g } => declare(loops(Vec::new(), &[self.names[g]])),
            Op::DeleteFree { m } | Op::DeleteMember { m, .. } => self.deleting(*m, Vec::new()),
            Op::DeleteAnchor { g, .. } => self.deleting(model.groups[g].anchor, Vec::new()),
            Op::RefuseOverlap { m } => {
                let one = BTreeSet::from([*m]);
                declare(loops(
                    vec![group(&self.roles(&one, *m)), group(&self.roles(&one, *m))],
                    &[],
                ))
            }
        }
    }

    /// Commits `op`, or checks it refuses typed with zero effects.
    pub fn execute(&mut self, op: &Op, model: &Model) -> Result<(), String> {
        let plan = self.plan(op, model);
        let token = self.token("op");
        let outcome = self.world.try_sculpt(&token, plan)?;
        match (op, &outcome) {
            (Op::RefuseOverlap { .. }, outcome) if naming_refusal(outcome).is_some() => Ok(()),
            (Op::RefuseOverlap { .. }, outcome) => Err(format!("overlap must refuse: {outcome:?}")),
            (_, Outcome::Committed { .. }) => Ok(()),
            (_, outcome) => Err(format!("{op:?} must commit: {outcome:?}")),
        }
    }

    /// Learns the names of the groups `op` newly labelled, after the model
    /// applied it: each is read through the group its anchor belongs to.
    pub fn learn(&mut self, op: &Op, model: &Model) -> Result<(), String> {
        let labelled: Vec<usize> = match op {
            Op::Born { g, .. } | Op::Revive { g, .. } => vec![*g],
            Op::Split { n, .. } => vec![*n],
            Op::SplitAway { news, .. } => news.to_vec(),
            Op::DeleteAnchor {
                successor: Some((n, _)),
                ..
            } => vec![*n],
            _ => Vec::new(),
        };
        for label in labelled {
            let anchor = model.groups[&label].anchor;
            let token = self.token("learn");
            let name = crate::persistent_naming::groups::loop_of(
                &self.world,
                &token,
                self.members[anchor],
            )
            .ok_or(format!("g{} is not live", label + 1))?;
            match (self.labels.get(&name), self.names.get(&label)) {
                (Some(known), _) if *known != label => {
                    return Err(format!("g{} came back as g{}", label + 1, known + 1));
                }
                (None, Some(_)) => return Err(format!("g{} revived under a new name", label + 1)),
                _ => {}
            }
            self.labels.insert(name, label);
            self.names.insert(label, name);
        }
        Ok(())
    }

    /// One decision reading every live member's loop and resolving every
    /// labelled loop under `Exact` and `AllDescendants`.
    pub fn observe(&mut self, model: &Model) -> Result<Observed, String> {
        let live: Vec<usize> = (0..self.members.len())
            .filter(|m| model.live_member[*m])
            .collect();
        let mut reads: Vec<GroupRead> = live
            .iter()
            .map(|m| GroupRead::LoopOf(Held::Name(self.members[*m])))
            .collect();
        let labels: Vec<usize> = self.names.keys().copied().collect();
        for label in &labels {
            for selection in [NameSelection::Exact, NameSelection::AllDescendants] {
                reads.push(GroupRead::Loop(self.names[label], selection));
            }
        }
        let token = self.token("observe");
        let answers = crate::persistent_naming::groups::read(&self.world, &token, reads);
        let label = |name: &GroupName| {
            self.labels
                .get(&name.loop_name())
                .copied()
                .ok_or(format!("unlabelled {name:?}"))
        };
        let mut observed = Observed {
            owners: BTreeMap::new(),
            answers: BTreeMap::new(),
        };
        let mut answers = answers.into_iter();
        for m in live {
            match answers.next() {
                Some(GroupAnswer::Of(owner)) => {
                    observed
                        .owners
                        .insert(m, owner.as_ref().map(label).transpose()?);
                }
                other => return Err(format!("m{} read {other:?}", m + 1)),
            }
        }
        for g in labels {
            for all in [false, true] {
                let answer = match answers.next() {
                    Some(GroupAnswer::Exact(name)) => ("exact", BTreeSet::from([label(&name)?])),
                    Some(GroupAnswer::Plural(names)) => {
                        ("plural", names.iter().map(label).collect::<Result<_, _>>()?)
                    }
                    Some(GroupAnswer::Ambiguous(names)) => (
                        "ambiguous",
                        names.iter().map(label).collect::<Result<_, _>>()?,
                    ),
                    Some(GroupAnswer::Retired) => ("retired", BTreeSet::new()),
                    other => return Err(format!("g{} resolved {other:?}", g + 1)),
                };
                observed.answers.insert((g, all), answer);
            }
        }
        Ok(observed)
    }

    /// Runs `first` and then a stage making `reads` as one edit envelope on
    /// its unpublished candidate: whether it committed, and each answer in
    /// model labels (`Refused*` for each typed candidate refusal).
    pub fn envelope(
        &mut self,
        first: &First,
        reads: &[Read],
        model: &Model,
    ) -> Result<(bool, Vec<Answer>), String> {
        let token = self.token("envelope");
        let stage = match first {
            First::Op(op) => sculpted(&format!("{token}-1"), self.plan(op, model)),
            First::Edit(m) => edited(&self.keys[*m], 900 + self.step as u64),
        };
        let group_reads = reads
            .iter()
            .map(|read| match *read {
                Read::MemberOf(m) => GroupRead::LoopOf(Held::Name(self.members[m])),
                Read::Resolve(g, all) => GroupRead::Loop(
                    self.names[&g],
                    if all {
                        NameSelection::AllDescendants
                    } else {
                        NameSelection::Exact
                    },
                ),
            })
            .collect();
        let reading = format!("{token}-read");
        let outcome = self.world.parts(
            &token,
            vec![
                stage,
                sculpted(
                    &reading,
                    Plan {
                        group_reads,
                        ..Plan::default()
                    },
                ),
            ],
        );
        let committed =
            matches!(
            outcome,
            worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome::Committed(_)
        );
        let answers = crate::persistent_naming::outputs::sculpt::group_answers(&reading);
        if answers.len() != reads.len() {
            return Err(format!("{reads:?} answered {answers:?}: {outcome:?}"));
        }
        let label = |name: &GroupName| {
            self.labels
                .get(&name.loop_name())
                .copied()
                .ok_or(format!("unlabelled {name:?}"))
        };
        let labels = |names: &[GroupName]| names.iter().map(label).collect::<Result<_, _>>();
        let answers = answers
            .iter()
            .map(|answer| {
                Ok(match answer {
                    GroupAnswer::Of(owner) => (
                        "member",
                        owner.as_ref().map(label).transpose()?.into_iter().collect(),
                    ),
                    GroupAnswer::Exact(name) => ("exact", BTreeSet::from([label(name)?])),
                    GroupAnswer::Plural(names) => ("plural", labels(names)?),
                    GroupAnswer::Ambiguous(names) => ("ambiguous", labels(names)?),
                    GroupAnswer::Retired => ("retired", BTreeSet::new()),
                    GroupAnswer::Refused(refusal) => {
                        let status = [
                            ("CandidateGroupState", "RefusedClaim"),
                            ("EndedInCandidate", "RefusedEnded"),
                            ("CandidateNameHistory", "RefusedHistory"),
                        ]
                        .into_iter()
                        .find(|(variant, _)| refusal.starts_with(variant))
                        .map(|(_, status)| status)
                        .ok_or(format!("untyped candidate refusal {refusal}"))?;
                        (status, BTreeSet::new())
                    }
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok((committed, answers))
    }
}
