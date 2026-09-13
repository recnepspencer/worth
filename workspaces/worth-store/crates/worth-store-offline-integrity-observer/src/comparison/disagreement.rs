use super::{
    protocol::{family, parse, Artifact},
    PhysicalIntegrityComparison, PhysicalIntegrityComparisonCounters,
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_foundational::{PhysicalArtifactIdentity, PhysicalIntegrityDisagreement};

static COMPARISON_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("store.physical.integrity-comparison");
const COMPARISON_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);

/// Compare two role-bound version-1 observations without selecting an outcome.
pub fn compare_integrity_observations(
    runtime: &str,
    offline: &str,
    limits: PhysicalIntegrityComparisonLimits,
) -> Result<PhysicalIntegrityComparison, Denial> {
    let runtime_report = parse(runtime, "runtime-integrity-observer", limits)?;
    let offline_report = parse(offline, crate::OFFLINE_OBSERVER_ROLE_IDENTITY, limits)?;
    if runtime_report.store != offline_report.store || runtime_report.store.is_none() {
        return Err(Denial::StoreMismatch);
    }
    if runtime_report.scenario != offline_report.scenario {
        return Err(Denial::ScenarioMismatch);
    }
    if runtime_report.process == offline_report.process
        || runtime_report.executable == offline_report.executable
    {
        return Err(Denial::SameObserver);
    }
    let runtime_artifacts = index(&runtime_report.artifacts)?;
    let offline_artifacts = index(&offline_report.artifacts)?;
    let scopes: BTreeSet<_> = runtime_artifacts
        .keys()
        .chain(offline_artifacts.keys())
        .cloned()
        .collect();
    let mut comparisons = Vec::with_capacity(scopes.len());
    let mut agreements = 0;
    let mut disagreements = 0;
    for scope in scopes {
        let runtime = runtime_artifacts.get(&scope).copied();
        let offline = offline_artifacts.get(&scope).copied();
        let fields = differing_fields(runtime, offline);
        let agreement = fields.is_empty();
        if agreement {
            agreements += 1;
        } else {
            disagreements += 1;
        }
        let posture_difference = match (runtime, offline) {
            (Some(runtime), Some(offline)) => family(&runtime.family).and_then(|family| {
                PhysicalIntegrityDisagreement::new(
                    family,
                    PhysicalArtifactIdentity::new(runtime.identity.clone()).ok()?,
                    runtime.outcome.posture(),
                    offline.outcome.posture(),
                )
            }),
            _ => None,
        };
        comparisons.push(json!({"path":scope.0,"family":scope.1,"offset":scope.2,"unbounded_identity":scope.3,"agreement":agreement,"different_fields":fields,"posture_disagreement":posture_difference}));
    }
    let document = json!({"protocol":COMPARISON_PROTOCOL.as_str(),"version":COMPARISON_VERSION.get(),"runtime":runtime_report,"offline":offline_report,"comparisons":comparisons,"consumed":{"agreements":agreements,"disagreements":disagreements,"input_bytes":runtime.len()+offline.len()}});
    let wire = encode_bounded(&document, limits.report_bytes)?;
    let counters = PhysicalIntegrityComparisonCounters {
        agreements,
        disagreements,
        input_bytes: (runtime.len() + offline.len()) as u64,
        report_bytes: wire.len() as u64,
    };
    Ok(PhysicalIntegrityComparison { wire, counters })
}

type Scope = (String, String, Option<u64>, Option<String>);
fn index(artifacts: &[Artifact]) -> Result<BTreeMap<Scope, &Artifact>, Denial> {
    let mut indexed = BTreeMap::new();
    for artifact in artifacts {
        let key = (
            artifact.path.clone(),
            artifact.family.clone(),
            artifact.range.as_ref().map(|range| range.offset),
            artifact.range.is_none().then(|| artifact.identity.clone()),
        );
        if indexed.insert(key, artifact).is_some() {
            return Err(Denial::DuplicateScope);
        }
    }
    Ok(indexed)
}
fn differing_fields(runtime: Option<&Artifact>, offline: Option<&Artifact>) -> Vec<&'static str> {
    let (Some(runtime), Some(offline)) = (runtime, offline) else {
        return vec!["presence"];
    };
    let mut fields = Vec::new();
    if runtime.identity != offline.identity {
        fields.push("identity");
    }
    if runtime.generation != offline.generation {
        fields.push("generation");
    }
    if runtime.range != offline.range {
        fields.push("range");
    }
    if runtime.outcome != offline.outcome {
        fields.push("outcome");
    }
    if runtime.duplicates != offline.duplicates {
        fields.push("duplicates");
    }
    fields
}
fn encode_bounded(value: &Value, maximum: u64) -> Result<String, Denial> {
    let mut writer = BoundedWriter {
        bytes: Vec::new(),
        maximum,
    };
    serde_json::to_writer(&mut writer, value).map_err(|_| Denial::ReportBoundExceeded)?;
    String::from_utf8(writer.bytes).map_err(|_| Denial::MalformedProtocol)
}
struct BoundedWriter {
    bytes: Vec<u8>,
    maximum: u64,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if (self.bytes.len() as u64).saturating_add(bytes.len() as u64) > self.maximum {
            return Err(io::ErrorKind::OutOfMemory.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
