//! Canonical semantic content of a persistent group fact: the read and its
//! exact answer.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::persistent_group::{
    PersistentGroupAnswer, PersistentGroupRead, PersistentGroupSubject,
};

pub fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    read: &PersistentGroupRead,
    answer: &PersistentGroupAnswer,
) {
    kind(entries, prefix, "persistent-group-v1");
    let (subject, digest) = match read.subject {
        PersistentGroupSubject::Member(member) => ("member", member),
        PersistentGroupSubject::Resolution(held) => ("resolution", held),
    };
    let value = |name: String| (name, CanonicalBasisEntryKind::Value);
    let (locus, entry) = value(format!("{prefix}.group-kind"));
    push(entries, locus, entry, text(read.group_kind.as_str()));
    let (locus, entry) = value(format!("{prefix}.member-kind"));
    push(entries, locus, entry, text(read.member_kind.as_str()));
    let (locus, entry) = value(format!("{prefix}.{subject}"));
    push(entries, locus, entry, digest_value(digest));
    match answer {
        PersistentGroupAnswer::Member(group) => {
            kind(entries, &format!("{prefix}.answer"), "member");
            if let Some(group) = group {
                let (locus, entry) = value(format!("{prefix}.answer.group"));
                push(entries, locus, entry, digest_value(*group));
            }
        }
        PersistentGroupAnswer::Live(groups) => {
            kind(entries, &format!("{prefix}.answer"), "live");
            number(entries, prefix, "answer.group-count", groups.len());
            for (index, (group, members)) in groups.iter().enumerate() {
                let (locus, entry) = value(format!("{prefix}.answer.{index}.group"));
                push(entries, locus, entry, digest_value(*group));
                number(
                    entries,
                    prefix,
                    &format!("answer.{index}.member-count"),
                    members.len(),
                );
                for (position, member) in members.iter().enumerate() {
                    let (locus, entry) = value(format!("{prefix}.answer.{index}.{position}"));
                    push(entries, locus, entry, digest_value(*member));
                }
            }
        }
        PersistentGroupAnswer::Retired(at_commit) => {
            kind(entries, &format!("{prefix}.answer"), "retired");
            number_u64(entries, prefix, "answer.at-commit", *at_commit);
        }
        PersistentGroupAnswer::Unknown => kind(entries, &format!("{prefix}.answer"), "unknown"),
    }
}

fn digest_value(digest: [u8; 32]) -> CanonicalBasisValue {
    CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(digest))
}
