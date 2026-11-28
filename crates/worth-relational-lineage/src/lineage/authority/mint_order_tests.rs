//! Minting order follows in-commit dependencies, not scheme identifiers.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::identity::data::primitives::{LineageId, PartitionId};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::{
    DiscriminatorBasisDenialReason, InstalledPersistentNameScheme,
    InstalledPersistentNameSchemeRegistry, LineageBasisView, LineageNameDenial,
    LineageRecordTarget, LineageSubject, PersistentNameBirthRecord, PersistentNameBirthRequest,
    PersistentNameDiscriminatorReference, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameReferenceBasis, PersistentNameSubjectKind,
};

use super::name_minting::mint_target_births;

/// `EDGE` sorts before `FACE`, so the canonical order mints edges first.
const EDGE: &str = "worth.a.edge.v1";
const FACE: &str = "worth.z.face.v1";

struct EmptyBasis;
impl LineageBasisView for EmptyBasis {
    fn live_holder(&self, _: &PersistentNameIdentity) -> Option<PersistentNameHolder> {
        None
    }
    fn live_lineage(&self, _: &PersistentNameHolder) -> Option<LineageId> {
        None
    }
    fn retired_lineage(&self, _: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        None
    }
    fn holder_name(&self, _: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        None
    }
}

fn registry(edge_roles: [&str; 2], face_roles: [&str; 3]) -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    let entity = PersistentNameSubjectKind::Entity;
    for scheme in [
        InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
            EDGE, entity, "edge", edge_roles, false, true, false,
        )
        .expect("valid scheme"),
        InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
            FACE, entity, "face", face_roles, false, true, false,
        )
        .expect("valid scheme"),
    ] {
        registry.register(scheme).expect("scheme registers");
    }
    registry
}

fn cad_registry() -> InstalledPersistentNameSchemeRegistry {
    registry(["authored", "edge_of"], ["authored", "edge_of", "patch"])
}

/// A canonical discriminator: a label, then a name reference when given.
fn discriminator(label: &str, reference: Option<PersistentNameIdentity>) -> Vec<u8> {
    let mut bytes = b"worth.query.name-discriminator.v1\0".to_vec();
    let mut text = |tag: Option<u8>, text: &str| {
        bytes.extend(tag);
        bytes.extend_from_slice(&(text.len() as u64).to_be_bytes());
        bytes.extend_from_slice(text.as_bytes());
    };
    text(None, "worth.tests.mint-order.v1");
    text(Some(0x19), label);
    if let Some(identity) = reference {
        bytes.push(0x20);
        bytes.extend_from_slice(identity.as_bytes());
    }
    bytes
}

/// A birth of `role` under `scheme` in occurrence `occ`, referencing
/// `references` as born under the given roles of the same occurrence.
fn birth(
    scheme: &str,
    role: &str,
    embedded: Option<PersistentNameIdentity>,
    references: &[(PersistentNameIdentity, &str)],
) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: scheme.to_owned(),
        occurrence_key: "occ".to_owned(),
        role_path: role.to_owned(),
        parent_identities: Vec::new(),
        referenced_names: references
            .iter()
            .map(|(identity, role)| PersistentNameDiscriminatorReference {
                identity: *identity,
                basis: PersistentNameReferenceBasis::SameOccurrence {
                    role: (*role).to_owned(),
                },
            })
            .collect(),
        canonical_discriminator: discriminator(role, embedded),
        holder: PersistentNameHolder::Lineage(LineageId(0)),
    }
}

fn identity(request: &PersistentNameBirthRequest) -> PersistentNameIdentity {
    PersistentNameIdentity::of_pending_birth(
        &request.scheme_id,
        &request.occurrence_key,
        &request.role_path,
        &request.parent_identities,
        &request.canonical_discriminator,
        false,
    )
}

type Minted = Result<BTreeMap<u64, PersistentNameBirthRecord>, LineageNameDenial>;

/// Mints `births`, each created at the paired record slot.
fn mint(
    registry: &InstalledPersistentNameSchemeRegistry,
    births: &[(u64, &PersistentNameBirthRequest)],
) -> Minted {
    let target = |slot| LineageRecordTarget::Entity(EntityId::new(PartitionId::main(), slot, 1));
    let named: BTreeMap<_, _> = births
        .iter()
        .map(|(slot, request)| {
            let mut request = (*request).clone();
            request.holder = target(*slot).to_holder();
            (target(*slot), request)
        })
        .collect();
    let created: BTreeSet<_> = named.keys().copied().collect();
    let minted = mint_target_births(registry, &EmptyBasis, &BTreeSet::new(), &created, &named)?;
    Ok(births
        .iter()
        .map(|(slot, _)| (*slot, minted[&target(*slot)].clone()))
        .collect())
}

fn unborn(denial: &LineageNameDenial) -> bool {
    matches!(
        denial,
        LineageNameDenial::InvalidDiscriminatorBasis {
            reason: DiscriminatorBasisDenialReason::UnbornSameOccurrenceName { .. },
            ..
        }
    )
}

#[test]
fn a_birth_mints_after_the_name_it_references_in_a_later_sorting_scheme() {
    let registry = cad_registry();
    let face = birth(FACE, "authored", None, &[]);
    let face_id = identity(&face);
    let edge = birth(EDGE, "edge_of", Some(face_id), &[(face_id, "authored")]);
    let minted = mint(&registry, &[(1, &edge), (2, &face)]).expect("the edge follows its face");
    assert_eq!(minted[&2].identity(), face_id);
    assert_eq!(minted[&1].identity(), identity(&edge));
    // Record allocation never changes a name.
    let swapped = mint(&registry, &[(2, &edge), (1, &face)]).expect("allocation-independent");
    assert_eq!(swapped[&2].identity(), minted[&1].identity());
    assert_eq!(swapped[&1].identity(), minted[&2].identity());
}

#[test]
fn a_chain_through_two_schemes_mints_in_dependency_order() {
    let registry = cad_registry();
    let face = birth(FACE, "authored", None, &[]);
    let face_id = identity(&face);
    let edge = birth(EDGE, "edge_of", Some(face_id), &[(face_id, "authored")]);
    let edge_id = identity(&edge);
    // A second face told apart by the edge: FACE -> EDGE -> FACE.
    let patch = birth(FACE, "patch", Some(edge_id), &[(edge_id, "edge_of")]);
    let minted = mint(&registry, &[(1, &patch), (2, &edge), (3, &face)])
        .expect("each birth follows the one it references");
    assert_eq!(minted[&3].identity(), face_id);
    assert_eq!(minted[&2].identity(), edge_id);
    assert_eq!(minted[&1].identity(), identity(&patch));
}

#[test]
fn a_dependency_cycle_and_an_unborn_reference_still_refuse() {
    // Each scheme ranks the other birth's role first, so both references
    // pass the role law; the declared references alone form the cycle.
    let registry = registry(["x", "y"], ["y", "x", "z"]);
    let (mut a, mut b) = (birth(EDGE, "y", None, &[]), birth(FACE, "x", None, &[]));
    let (a_id, b_id) = (identity(&a), identity(&b));
    a.referenced_names = birth(EDGE, "y", None, &[(b_id, "x")]).referenced_names;
    b.referenced_names = birth(FACE, "x", None, &[(a_id, "y")]).referenced_names;
    let cycle = mint(&registry, &[(1, &a), (2, &b)]).expect_err("a cycle never mints");
    assert!(unborn(&cycle), "{cycle:?}");

    let registry = cad_registry();
    let absent = identity(&birth(FACE, "authored", None, &[]));
    let edge = birth(EDGE, "edge_of", Some(absent), &[(absent, "authored")]);
    let alone = mint(&registry, &[(1, &edge)]).expect_err("the face is not in the commit");
    assert!(unborn(&alone), "{alone:?}");
}
