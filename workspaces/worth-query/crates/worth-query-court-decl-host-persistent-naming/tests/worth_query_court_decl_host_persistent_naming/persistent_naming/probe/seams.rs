//! The probe's tracked seam name reads: `relation_name_of` on the seam
//! between two probed members, and `resolve_relation` on held seam names.
use std::collections::BTreeMap;
use std::sync::Mutex;

use super::super::*;
use worth_query_decl::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_host::facade::application_naming::ExecutionNameResolveDenial;

type Live = [(String, WorthQueryInvariantEntityIdentity<Schema, Member>)];
type Handle = WorthQueryInvariantRelation<Schema, Seam, Member, Member>;

/// What one probe run observed of seams. Endpoints are probe member keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeamObservation {
    /// `relation_name_of` on the live seam labelled `label`.
    Named {
        label: String,
        name: WorthQueryName<Seam>,
    },
    NameRefused {
        label: String,
        refusal: String,
    },
    /// No seam runs between the labelled pair at the probe's basis.
    Unlinked {
        label: String,
    },
    /// A held name resolved to the live seam between `ends`.
    ResolvedTo {
        label: String,
        ends: (String, String),
    },
    /// A held name resolved to two or more live seams.
    Plural {
        label: String,
        ends: Vec<(String, String)>,
    },
    Retired {
        label: String,
    },
    /// `Exact` refused with every live candidate name.
    Ambiguous {
        label: String,
        candidates: Vec<WorthQueryName<Seam>>,
    },
    ResolveRefused {
        label: String,
        refusal: String,
    },
}

#[derive(Default)]
struct SeamProbe {
    named: Vec<(String, String, String)>,
    held: Vec<(String, WorthQueryName<Seam>, NameSelection)>,
    observed: Vec<SeamObservation>,
}

static PROBES: Mutex<BTreeMap<String, SeamProbe>> = Mutex::new(BTreeMap::new());

fn probes() -> std::sync::MutexGuard<'static, BTreeMap<String, SeamProbe>> {
    PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Names the seam from member `from` to member `to` under `label` in every
/// later probe run carrying `token`.
pub fn name(token: &str, label: &str, from: &str, to: &str) {
    let mut probes = probes();
    let probe = probes.entry(token.to_owned()).or_default();
    probe
        .named
        .push((label.to_owned(), from.to_owned(), to.to_owned()));
}

/// Holds `name` under `label`, resolved with `selection` by later probes.
pub fn hold(token: &str, label: &str, name: WorthQueryName<Seam>, selection: NameSelection) {
    let mut probes = probes();
    let probe = probes.entry(token.to_owned()).or_default();
    probe.held.push((label.to_owned(), name, selection));
}

/// Takes what probe runs carrying `token` observed of seams since last taken.
pub fn take(token: &str) -> Vec<SeamObservation> {
    probes()
        .get_mut(token)
        .map(|probe| std::mem::take(&mut probe.observed))
        .unwrap_or_default()
}

fn key_of(live: &Live, member: &WorthQueryInvariantEntityIdentity<Schema, Member>) -> String {
    live.iter()
        .find(|(_, identity)| identity == member)
        .map_or_else(|| "?".to_owned(), |(key, _)| key.clone())
}

fn ends(live: &Live, seam: &Handle) -> (String, String) {
    (key_of(live, seam.from()), key_of(live, seam.to()))
}

/// Runs the seam reads registered for `token` over the probe's live members.
pub(super) fn observe(
    reader: &mut DecisionReader<'_, '_, '_, Schema, super::Binding>,
    token: &str,
    live: &Live,
) -> Result<(), HandlerExecutionDenial> {
    let (named, held) = probes()
        .get(token)
        .map(|probe| (probe.named.clone(), probe.held.clone()))
        .unwrap_or_default();
    let mut observed = Vec::new();
    let member = |key: &str| live.iter().find(|(k, _)| k == key).map(|(_, m)| m);
    for (label, from, to) in named {
        let (Some(from), Some(to)) = (member(&from), member(&to)) else {
            observed.push(SeamObservation::Unlinked { label });
            continue;
        };
        let seam = reader
            .relations_from(Seam::reference(), from)
            .map_err(HandlerExecutionDenial::new)?
            .into_iter()
            .find(|seam| seam.to() == to);
        observed.push(match seam {
            None => SeamObservation::Unlinked { label },
            Some(seam) => match reader.reader().relation_name_of(&seam) {
                Ok(name) => SeamObservation::Named { label, name },
                Err(refusal) => SeamObservation::NameRefused {
                    label,
                    refusal: format!("{refusal:?}"),
                },
            },
        });
    }
    for (label, name, selection) in held {
        let resolved = reader
            .reader()
            .resolve_relation(Seam::reference(), &name, &selection)
            .map_err(HandlerExecutionDenial::new)?;
        observed.push(match resolved {
            Ok(ResolvedName::Exact(seam)) => SeamObservation::ResolvedTo {
                label,
                ends: ends(live, &seam),
            },
            Ok(ResolvedName::Plural(seams)) => SeamObservation::Plural {
                label,
                ends: seams.iter().map(|seam| ends(live, seam)).collect(),
            },
            Ok(ResolvedName::Retired { .. }) => SeamObservation::Retired { label },
            Err(ExecutionNameResolveDenial::Resolution(NameResolutionDenial::Ambiguous {
                candidates,
            })) => SeamObservation::Ambiguous { label, candidates },
            Err(refusal) => SeamObservation::ResolveRefused {
                label,
                refusal: format!("{refusal:?}"),
            },
        });
    }
    probes().entry(token.to_owned()).or_default().observed = observed;
    Ok(())
}
