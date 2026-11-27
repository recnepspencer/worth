//! Canonical semantic content of a persistent name-history fact: the read,
//! its exact Relational question and its exact answer.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::persistent_name_history::{
    PersistentNameHistoryAnswer, PersistentNameHistoryQuestion, PersistentNameHistoryRead,
};
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameResolution, PersistentNameResolutionDenial,
    PersistentNameSelection, PersistentNameSubjectKind, PersistentNameTarget,
};

pub fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    read: &PersistentNameHistoryRead,
    answer: &PersistentNameHistoryAnswer,
) {
    kind(entries, prefix, "persistent-name-history-v1");
    let subject = match read.subject {
        PersistentNameSubjectKind::Entity => "entity",
        PersistentNameSubjectKind::Relation => "relation",
        PersistentNameSubjectKind::Group => "group",
    };
    value(entries, format!("{prefix}.subject"), text(subject));
    value(entries, format!("{prefix}.kind"), text(read.kind.as_str()));
    match &read.question {
        PersistentNameHistoryQuestion::Resolution { held, selection } => {
            kind(entries, &format!("{prefix}.question"), "resolution");
            name_key(entries, &format!("{prefix}.question.held"), *held);
            append_selection(entries, &format!("{prefix}.question.selection"), selection);
        }
        PersistentNameHistoryQuestion::Birth(digest) => {
            kind(entries, &format!("{prefix}.question"), "birth");
            value(
                entries,
                format!("{prefix}.question.digest"),
                digest_value(*digest),
            );
        }
        PersistentNameHistoryQuestion::MemberGroups(member) => {
            kind(entries, &format!("{prefix}.question"), "member-groups");
            value(
                entries,
                format!("{prefix}.question.member"),
                digest_value(*member),
            );
        }
    }
    let answer_prefix = format!("{prefix}.answer");
    match answer {
        PersistentNameHistoryAnswer::Birth { recorded } => {
            kind(
                entries,
                &answer_prefix,
                if *recorded { "born" } else { "unborn" },
            );
        }
        PersistentNameHistoryAnswer::MemberGroups(groups) => {
            kind(entries, &answer_prefix, "member-groups");
            number(entries, &answer_prefix, "group-count", groups.len());
            for (index, (group_kind, group)) in groups.iter().enumerate() {
                let at = format!("{answer_prefix}.group.{index}");
                value(entries, format!("{at}.kind"), text(group_kind.as_str()));
                value(entries, format!("{at}.digest"), digest_value(*group));
            }
        }
        PersistentNameHistoryAnswer::Resolution { result, rests_on } => {
            append_result(entries, &answer_prefix, result);
            number(entries, &answer_prefix, "rests-on-count", rests_on.len());
            for (index, lineage) in rests_on.iter().enumerate() {
                number_u64(
                    entries,
                    &answer_prefix,
                    &format!("rests-on.{index}"),
                    lineage.0,
                );
            }
        }
    }
}

fn append_selection(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    selection: &PersistentNameSelection,
) {
    match selection {
        PersistentNameSelection::Exact => kind(entries, prefix, "exact"),
        PersistentNameSelection::AllDescendants => kind(entries, prefix, "all-descendants"),
        PersistentNameSelection::KeepSide { discriminator } => {
            kind(entries, prefix, "keep-side");
            let hex: String = discriminator
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            value(entries, format!("{prefix}.discriminator-hex"), text(hex));
        }
        PersistentNameSelection::BoundedByCarriers { carriers } => {
            kind(entries, prefix, "bounded-by-carriers");
            number(entries, prefix, "carrier-count", carriers.len());
            for (index, carrier) in carriers.iter().enumerate() {
                let digest = digest_value(*carrier.as_bytes());
                value(entries, format!("{prefix}.carrier.{index}"), digest);
            }
        }
    }
}

fn append_result(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    result: &Result<PersistentNameResolution, PersistentNameResolutionDenial>,
) {
    let keys = |entries: &mut Vec<CanonicalBasisEntry>, keys: &[PersistentNameKey]| {
        number(entries, prefix, "key-count", keys.len());
        for (index, key) in keys.iter().enumerate() {
            name_key(entries, &format!("{prefix}.key.{index}"), *key);
        }
    };
    match result {
        Ok(PersistentNameResolution::Exact(target)) => {
            kind(entries, prefix, "exact");
            name_target(entries, &format!("{prefix}.target"), target);
        }
        Ok(PersistentNameResolution::Plural(targets)) => {
            kind(entries, prefix, "plural");
            number(entries, prefix, "target-count", targets.len());
            for (index, target) in targets.iter().enumerate() {
                name_target(entries, &format!("{prefix}.target.{index}"), target);
            }
        }
        Ok(PersistentNameResolution::Retired { at_commit }) => {
            kind(entries, prefix, "retired");
            number_u64(entries, prefix, "at-commit", at_commit.0);
        }
        Err(PersistentNameResolutionDenial::Ambiguous { candidates }) => {
            kind(entries, prefix, "ambiguous");
            keys(entries, candidates);
        }
        Err(PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates }) => {
            kind(entries, prefix, "no-matching-candidate");
            keys(entries, live_candidates);
        }
        Err(PersistentNameResolutionDenial::BasisPruned {
            basis,
            retention_floor,
        }) => {
            kind(entries, prefix, "basis-pruned");
            number_u64(entries, prefix, "basis", basis.0);
            number_u64(entries, prefix, "retention-floor", retention_floor.0);
        }
        Err(PersistentNameResolutionDenial::UnknownName { name }) => {
            kind(entries, prefix, "unknown-name");
            name_key(entries, &format!("{prefix}.name"), *name);
        }
        Err(PersistentNameResolutionDenial::EmptySelectionFilter) => {
            kind(entries, prefix, "empty-selection-filter");
        }
    }
}

fn name_target(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    target: &PersistentNameTarget,
) {
    name_key(entries, &format!("{prefix}.name"), target.key());
    match target {
        PersistentNameTarget::Entity {
            entity: id,
            lineage,
            ..
        } => {
            entity(entries, prefix, "entity", *id);
            number_u64(entries, prefix, "lineage", lineage.0);
        }
        PersistentNameTarget::Relation {
            relation, lineage, ..
        } => {
            record(entries, prefix, "relation", *relation);
            number_u64(entries, prefix, "lineage", lineage.0);
        }
        PersistentNameTarget::Group { .. } => {}
    }
}

fn name_key(entries: &mut Vec<CanonicalBasisEntry>, prefix: &str, key: PersistentNameKey) {
    let (subject, digest) = match key {
        PersistentNameKey::Entity(identity) => ("entity", *identity.as_bytes()),
        PersistentNameKey::Relation(identity) => ("relation", *identity.as_bytes()),
        PersistentNameKey::Group(group) => ("group", *group.bytes()),
    };
    kind(entries, prefix, subject);
    value(entries, format!("{prefix}.digest"), digest_value(digest));
}

fn value(entries: &mut Vec<CanonicalBasisEntry>, locus: String, value: CanonicalBasisValue) {
    push(entries, locus, CanonicalBasisEntryKind::Value, value);
}

fn digest_value(digest: [u8; 32]) -> CanonicalBasisValue {
    CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(digest))
}
