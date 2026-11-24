//! Reads what the installed world says: the names live records carry and
//! every ever-minted name resolved unscoped and within each owner model,
//! `Exact` and `AllDescendants`, through the probe operation's tracked reader.
use std::collections::{BTreeMap, BTreeSet};

use crate::persistent_naming::probe::{self, Observation};
use super::driver::Driver;
use super::reference::{Answer, SCOPES, SELECTIONS};
use worth_query_decl::facade::application_naming::{NameSelection, WorthQueryName};

use crate::persistent_naming::Member;

/// What one observation of the world found.
pub struct Observed {
    /// Model names some live record carries.
    pub live: BTreeSet<usize>,
    /// `(name, scope, all_descendants)` to what resolving it answered.
    pub answers: BTreeMap<(usize, Option<&'static str>, bool), Answer>,
}

fn label(name: usize, scope: Option<&str>, all: bool) -> String {
    format!("n{}|{}|{all}", name + 1, scope.unwrap_or("*"))
}

fn selection(all: bool) -> NameSelection {
    if all {
        NameSelection::AllDescendants
    } else {
        NameSelection::Exact
    }
}

impl Driver {
    /// Learns the admitted names of `created` records: a new model name must
    /// carry a name no other model name has; a regenerated or restored one
    /// must carry its earlier name exactly.
    pub fn learn(&mut self, step: usize, created: &[(String, usize)]) -> Result<(), String> {
        if created.is_empty() {
            return Ok(());
        }
        let token = self.token(step, "learn");
        let keys: Vec<&str> = created.iter().map(|(key, _)| key.as_str()).collect();
        let observed = self.world.probe(&token, &keys, 0);
        for (key, model_name) in created {
            let name = observed
                .iter()
                .find_map(|observation| match observation {
                    Observation::Named { key: k, name } if k == key => Some(*name),
                    _ => None,
                })
                .ok_or_else(|| format!("created `{key}` carries no name: {observed:?}"))?;
            match self.names.get(model_name) {
                Some(known) if *known != name => {
                    return Err(format!("n{} came back as another name", model_name + 1));
                }
                Some(_) => {}
                None if self.names.values().any(|known| *known == name) => {
                    return Err(format!("n{} minted an existing name", model_name + 1));
                }
                None => {
                    self.names.insert(*model_name, name);
                }
            }
            self.keys.insert(*model_name, key.clone());
        }
        Ok(())
    }

    /// Resolves every ever-minted name in every scope and selection.
    pub fn observe(&mut self, step: usize) -> Result<Observed, String> {
        let token = self.token(step, "observe");
        for (&name, &held) in &self.names {
            for scope in SCOPES {
                for all in SELECTIONS {
                    let label = label(name, scope, all);
                    match scope {
                        None => probe::hold_selecting(&token, &label, held, selection(all)),
                        Some(owner) => probe::hold_in(&token, &label, held, selection(all), owner),
                    }
                }
            }
        }
        let keys: Vec<&str> = self.created.iter().map(String::as_str).collect();
        let observed = self.world.probe(&token, &keys, 0);
        let by_name: BTreeMap<WorthQueryName<Member>, usize> = self
            .names
            .iter()
            .map(|(model, name)| (*name, *model))
            .collect();
        let model = |name: &WorthQueryName<Member>| {
            by_name
                .get(name)
                .copied()
                .ok_or_else(|| format!("a record carries an unknown name: {observed:?}"))
        };
        let mut live = BTreeSet::new();
        let mut key_names = BTreeMap::new();
        for observation in &observed {
            if let Observation::Named { key, name } = observation {
                let name = model(name)?;
                if !live.insert(name) {
                    return Err(format!("two live records carry n{}", name + 1));
                }
                key_names.insert(key.clone(), name);
            }
        }
        for (name, key) in &self.keys {
            if live.contains(name) && key_names.get(key) != Some(name) {
                return Err(format!("n{} is not carried by `{key}`", name + 1));
            }
        }
        self.keys.retain(|name, _| live.contains(name));
        let keyed = |keys: &[String]| -> Result<BTreeSet<usize>, String> {
            keys.iter()
                .map(|key| {
                    key_names
                        .get(key)
                        .copied()
                        .ok_or(format!("unnamed `{key}`"))
                })
                .collect()
        };
        let named = |names: &mut dyn Iterator<Item = &WorthQueryName<Member>>| -> Result<BTreeSet<usize>, String> {
            names.map(|name| model(name)).collect()
        };
        let mut answers = BTreeMap::new();
        for &name in self.names.keys() {
            for scope in SCOPES {
                for all in SELECTIONS {
                    let label = label(name, scope, all);
                    let found = observed.iter().find(|observation| match observation {
                        Observation::ResolvedTo { label: l, .. }
                        | Observation::ResolvedElsewhere { label: l }
                        | Observation::Retired { label: l }
                        | Observation::ResolveRefused { label: l, .. }
                        | Observation::Plural { label: l, .. }
                        | Observation::Ambiguous { label: l, .. }
                        | Observation::AmbiguousOwned { label: l, .. }
                        | Observation::NotInOwner { label: l, .. } => *l == label,
                        _ => false,
                    });
                    let answer = match found {
                        Some(Observation::ResolvedTo { key, .. }) => Answer {
                            status: "Exact",
                            targets: keyed(std::slice::from_ref(key))?,
                        },
                        Some(Observation::Plural { keys, .. }) => Answer {
                            status: "Plural",
                            targets: keyed(keys)?,
                        },
                        Some(Observation::Retired { .. }) => Answer {
                            status: "Retired",
                            targets: BTreeSet::new(),
                        },
                        Some(Observation::Ambiguous { candidates, .. }) => Answer {
                            status: "Ambiguous",
                            targets: named(&mut candidates.iter())?,
                        },
                        Some(Observation::AmbiguousOwned { candidates, .. }) => Answer {
                            status: "Ambiguous",
                            targets: named(&mut candidates.iter().map(|(name, _)| name))?,
                        },
                        Some(Observation::NotInOwner { candidates, .. }) => Answer {
                            status: "NoCandidateInOwner",
                            targets: named(&mut candidates.iter().map(|(name, _)| name))?,
                        },
                        other => return Err(format!("`{label}` answered {other:?}")),
                    };
                    answers.insert((name, scope, all), answer);
                }
            }
        }
        Ok(Observed { live, answers })
    }
}
