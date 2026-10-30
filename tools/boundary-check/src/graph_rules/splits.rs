//! Registered court areas must be disjoint or explicitly delegated.
use crate::config::*;
use std::collections::BTreeSet;
pub fn check(config: &toml::Value) -> Vec<Diagnostic> {
    let splits = table(config, "splits");
    let mut names = BTreeSet::new();
    let mut out = Vec::new();
    for split in &splits {
        let fail = |message| {
            diagnostic(
                Code::Topology,
                &format!("{CONFIG}court-frontier-splits.toml"),
                message,
            )
        };
        let owners = strings(split, "owners");
        let delegates = strings(split, "delegates");
        if !names.insert(string(split, "court"))
            || owners.is_empty()
            || string(split, "area").is_empty()
            || !string(split, "area")
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || owners.iter().chain(&delegates).any(|s| {
                s.split('/').any(|s| {
                    s.is_empty() || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                })
            })
        {
            out.push(fail("invalid or duplicate court split"));
        }
        let peers: Vec<_> = splits
            .iter()
            .filter(|s| {
                string(s, "court") != string(split, "court")
                    && strings(s, "frontier") == strings(split, "frontier")
            })
            .collect();
        if peers.is_empty()
            || peers
                .iter()
                .any(|s| string(s, "area") == string(split, "area"))
        {
            out.push(fail("split requires a sibling with a distinct area"));
        }
        for delegate in &delegates {
            if !owners.iter().any(|owner| {
                peers.iter().any(|other| {
                    strings(other, "owners")
                        .iter()
                        .any(|child| under(child, &format!("{owner}/{delegate}")))
                })
            }) {
                out.push(fail("delegate has no registered owning area"));
            }
        }
        for peer in peers {
            for child in strings(peer, "owners") {
                for owner in &owners {
                    if child == *owner
                        || under(&child, owner)
                            && !delegates
                                .iter()
                                .any(|d| under(&child, &format!("{owner}/{d}")))
                    {
                        out.push(fail("overlapping court area owners without delegation"));
                    }
                }
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn area_splits_require_explicit_delegation_of_nested_owners() {
        let text = "[[splits]]\ncourt='a'\nfrontier=['product']\narea='root'\nowners=['root']\ndelegates=['child']\n[[splits]]\ncourt='b'\nfrontier=['product']\narea='child'\nowners=['root/child']\n";
        assert!(check(&toml::from_str(text).unwrap()).is_empty());
        assert!(!check(
            &toml::from_str(&text.replace("delegates=['child']", "delegates=[]")).unwrap()
        )
        .is_empty());
    }
}
