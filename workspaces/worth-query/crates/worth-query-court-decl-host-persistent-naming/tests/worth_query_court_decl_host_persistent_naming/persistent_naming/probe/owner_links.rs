//! The probe's tracked reads of owner links: `relation_name_of` on the
//! `ModelMember` link from model `a` to a probed member. `ModelMember` has
//! no name scheme, so a link carries a name only when a run names it.
use std::collections::BTreeMap;
use std::sync::Mutex;

use super::super::*;
use worth_query_decl::facade::application_naming::WorthQueryName;

/// What one probe run observed of owner links, by label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkObservation {
    Named {
        label: String,
        name: WorthQueryName<ModelMember>,
    },
    NameRefused {
        label: String,
        refusal: String,
    },
}

#[derive(Default)]
struct LinkProbe {
    reads: Vec<(String, String)>,
    observed: Vec<LinkObservation>,
}

static PROBES: Mutex<BTreeMap<String, LinkProbe>> = Mutex::new(BTreeMap::new());

fn probes() -> std::sync::MutexGuard<'static, BTreeMap<String, LinkProbe>> {
    PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Names the link from model `a` to probed member `key` under `label`.
pub fn name(token: &str, label: &str, key: &str) {
    let mut probes = probes();
    let probe = probes.entry(token.to_owned()).or_default();
    probe.reads.push((label.to_owned(), key.to_owned()));
}

/// Takes what probe runs carrying `token` observed since the last take.
pub fn take(token: &str) -> Vec<LinkObservation> {
    probes()
        .get_mut(token)
        .map(|probe| std::mem::take(&mut probe.observed))
        .unwrap_or_default()
}

pub(super) fn observe(
    reader: &mut DecisionReader<'_, '_, '_, Schema, super::Binding>,
    token: &str,
    model: &WorthQueryInvariantEntityIdentity<Schema, Model>,
    live: &[(String, WorthQueryInvariantEntityIdentity<Schema, Member>)],
) -> Result<(), HandlerExecutionDenial> {
    let reads = probes()
        .get(token)
        .map(|probe| probe.reads.clone())
        .unwrap_or_default();
    let mut observed = Vec::new();
    for (label, key) in reads {
        let member = live.iter().find(|(k, _)| *k == key).map(|(_, m)| m);
        let member = member.unwrap_or_else(|| panic!("`{key}` is probed"));
        let link = reader
            .relations_from(ModelMember::reference(), model)
            .map_err(HandlerExecutionDenial::new)?
            .into_iter()
            .find(|link| link.to() == member)
            .unwrap_or_else(|| panic!("model `a` owns `{key}`"));
        observed.push(match reader.reader().relation_name_of(&link) {
            Ok(name) => LinkObservation::Named { label, name },
            Err(refusal) => LinkObservation::NameRefused {
                label,
                refusal: format!("{refusal:?}"),
            },
        });
    }
    probes().entry(token.to_owned()).or_default().observed = observed;
    Ok(())
}
