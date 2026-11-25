//! The probe's tracked birth reads: `birth_of` on held member names and
//! `name_for_birth` on birth parts, observed whether or not the run seals.
use std::collections::BTreeMap;
use std::sync::Mutex;

use super::super::*;
use worth_query_decl::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_host::facade::application_naming::WorthQueryOutputBirth;

/// What one probe run observed of births.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BirthObservation {
    /// `birth_of` on the name held under `label`.
    Born {
        label: String,
        birth: RecordedNameBirth<Member>,
    },
    BirthRefused {
        label: String,
        refusal: String,
    },
    /// `name_for_birth` on the birth parts held under `label`.
    Minted {
        label: String,
        name: Option<WorthQueryName<Member>>,
    },
    LookupRefused {
        label: String,
        refusal: String,
    },
    /// `relation_birth_of` on the seam name held under `label`.
    SeamBorn {
        label: String,
        birth: RecordedNameBirth<Seam>,
    },
    /// `relation_name_for_birth` on the seam birth parts held under `label`.
    SeamMinted {
        label: String,
        name: Option<WorthQueryName<Seam>>,
    },
}

#[derive(Clone)]
enum Read {
    BirthOf(WorthQueryName<Member>),
    ForBirth(WorthQueryOutputBirth, Vec<WorthQueryName<Member>>),
    SeamBirthOf(WorthQueryName<Seam>),
    SeamForBirth(WorthQueryOutputBirth),
}

#[derive(Default)]
struct BirthProbe {
    reads: Vec<(String, Read)>,
    observed: Vec<BirthObservation>,
}

static PROBES: Mutex<BTreeMap<String, BirthProbe>> = Mutex::new(BTreeMap::new());

fn probes() -> std::sync::MutexGuard<'static, BTreeMap<String, BirthProbe>> {
    PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn hold(token: &str, label: &str, read: Read) {
    let mut probes = probes();
    let probe = probes.entry(token.to_owned()).or_default();
    probe.reads.push((label.to_owned(), read));
}

/// Reads the birth of `name` under `label` in every later probe run.
pub fn birth_of(token: &str, label: &str, name: WorthQueryName<Member>) {
    hold(token, label, Read::BirthOf(name));
}

/// Looks up the name `birth` under `parents` was minted as, under `label`.
pub fn name_for_birth(
    token: &str,
    label: &str,
    birth: WorthQueryOutputBirth,
    parents: Vec<WorthQueryName<Member>>,
) {
    hold(token, label, Read::ForBirth(birth, parents));
}

/// Reads the birth of seam name `name` under `label` in later probe runs.
pub fn seam_birth_of(token: &str, label: &str, name: WorthQueryName<Seam>) {
    hold(token, label, Read::SeamBirthOf(name));
}

/// Looks up the seam name `birth` was minted as, under `label`.
pub fn seam_name_for_birth(token: &str, label: &str, birth: WorthQueryOutputBirth) {
    hold(token, label, Read::SeamForBirth(birth));
}

/// Takes what probe runs carrying `token` observed since the last take.
pub fn take(token: &str) -> Vec<BirthObservation> {
    probes()
        .get_mut(token)
        .map(|probe| std::mem::take(&mut probe.observed))
        .unwrap_or_default()
}

pub(super) fn observe(
    reader: &mut DecisionReader<'_, '_, '_, Schema, super::Binding>,
    token: &str,
) {
    let reads = probes()
        .get(token)
        .map(|probe| probe.reads.clone())
        .unwrap_or_default();
    let observed = reads
        .into_iter()
        .map(|(label, read)| match read {
            Read::BirthOf(name) => match reader.reader().birth_of(&name) {
                Ok(birth) => BirthObservation::Born { label, birth },
                Err(refusal) => BirthObservation::BirthRefused {
                    label,
                    refusal: format!("{refusal:?}"),
                },
            },
            Read::SeamBirthOf(name) => {
                match reader.reader().relation_birth_of(Seam::reference(), &name) {
                    Ok(birth) => BirthObservation::SeamBorn { label, birth },
                    Err(refusal) => BirthObservation::BirthRefused {
                        label,
                        refusal: format!("{refusal:?}"),
                    },
                }
            }
            Read::SeamForBirth(birth) => {
                let seam = Seam::reference();
                match reader.reader().relation_name_for_birth(seam, &birth, &[]) {
                    Ok(name) => BirthObservation::SeamMinted { label, name },
                    Err(refusal) => BirthObservation::LookupRefused {
                        label,
                        refusal: format!("{refusal:?}"),
                    },
                }
            }
            Read::ForBirth(birth, parents) => {
                match reader.reader().name_for_birth(&birth, &parents) {
                    Ok(name) => BirthObservation::Minted { label, name },
                    Err(refusal) => BirthObservation::LookupRefused {
                        label,
                        refusal: format!("{refusal:?}"),
                    },
                }
            }
        })
        .collect();
    probes().entry(token.to_owned()).or_default().observed = observed;
}
