//! Canonical full recipe meaning, shared by the actual schema snapshot digest.
use super::DerivedRelationNode as Node;
use super::{DerivedRelationDefinitionParts as Parts, DerivedRelationIdentity as Identity};
use sha2::{Digest, Sha256};

pub(crate) fn update_digest(hasher: &mut Sha256, definitions: &[Parts]) {
    if definitions.is_empty() {
        return;
    }
    hasher.update(b"\0WORTH_DERIVED_RELATION_DEFINITIONS_V1\0");
    count(hasher, definitions.len());
    for definition in definitions {
        identity(hasher, &definition.identity);
        hasher.update(definition.from.0.to_le_bytes());
        hasher.update(definition.to.0.to_le_bytes());
        count(hasher, definition.nodes.len());
        for node in &definition.nodes {
            match node {
                Node::Stored { relation, from, to } => {
                    hasher.update([1]);
                    for kind in [relation, from, to] {
                        hasher.update(kind.0.to_le_bytes());
                    }
                }
                Node::Derived(source) => {
                    hasher.update([2]);
                    identity(hasher, source);
                }
                Node::Compose { left, right } => {
                    hasher.update([3]);
                    count(hasher, *left);
                    count(hasher, *right);
                }
                Node::Reverse { input } => {
                    hasher.update([4]);
                    count(hasher, *input);
                }
                Node::WithoutSelfEdges { input } => {
                    hasher.update([5]);
                    count(hasher, *input);
                }
            }
        }
    }
}

fn identity(hasher: &mut Sha256, identity: &Identity) {
    for text in [&identity.schema.0, &identity.name] {
        count(hasher, text.len());
        hasher.update(text.as_bytes());
    }
    hasher.update(identity.version.0.to_le_bytes());
}

fn count(hasher: &mut Sha256, value: usize) {
    hasher.update((value as u64).to_le_bytes());
}
