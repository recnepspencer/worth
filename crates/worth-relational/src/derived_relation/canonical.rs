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
        endpoint(hasher, &definition.from);
        endpoint(hasher, &definition.to);
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
                Node::CarrierLift { domain, case } => {
                    hasher.update([6]);
                    projected_identity(hasher, domain);
                    count(hasher, case.len());
                    hasher.update(case.as_bytes());
                }
                Node::Union { left, right } | Node::Intersection { left, right } => {
                    hasher.update([if matches!(node, Node::Union { .. }) {
                        7
                    } else {
                        8
                    }]);
                    count(hasher, *left);
                    count(hasher, *right);
                }
            }
        }
        if let Some(laws) = &definition.laws {
            super::laws::canonical::update_digest(hasher, laws);
        }
    }
}

pub(super) fn endpoint(hasher: &mut Sha256, domain: &crate::graph_member::NativeMemberDomain) {
    match domain {
        // This is exactly the old physical endpoint encoding.
        crate::graph_member::NativeMemberDomain::Entity(kind) => {
            hasher.update(kind.0.to_le_bytes())
        }
        crate::graph_member::NativeMemberDomain::Projected { projected } => {
            hasher.update(b"\0WORTH_PROJECTED_ENDPOINT_V1\0");
            projected_identity(hasher, projected);
        }
    }
}

fn projected_identity(
    hasher: &mut Sha256,
    identity: &crate::projected_member_domain::ProjectedMemberDomainIdentity,
) {
    for text in [&identity.schema.0, &identity.name] {
        count(hasher, text.len());
        hasher.update(text.as_bytes());
    }
    hasher.update(identity.version.0.to_le_bytes());
}

fn identity(hasher: &mut Sha256, identity: &Identity) {
    write_identity(identity, |bytes| hasher.update(bytes));
}

pub(crate) fn write_identity(identity: &Identity, mut write: impl FnMut(&[u8])) {
    for text in [&identity.schema.0, &identity.name] {
        write(&(text.len() as u64).to_le_bytes());
        write(text.as_bytes());
    }
    write(&identity.version.0.to_le_bytes());
}

fn count(hasher: &mut Sha256, value: usize) {
    hasher.update((value as u64).to_le_bytes());
}
