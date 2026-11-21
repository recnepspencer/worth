use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use crate::lineage::authority::PersistentNameMintingAuthority;
use crate::lineage::data::{
    InstalledPersistentNameSchemeRegistry, PersistentNameBirthRequest, PersistentNameIdentity,
};

use super::{entity_holder, face_registry};

#[derive(Clone, Debug)]
struct DagNodeSpec {
    occurrence_key: String,
    role_index: usize,
    parents: Vec<usize>,
    disc: Vec<u8>,
}

fn arb_birth_dag() -> impl Strategy<Value = Vec<DagNodeSpec>> {
    (2usize..=10).prop_flat_map(|node_count| {
        let mut strategies = Vec::with_capacity(node_count);
        for index in 0..node_count {
            let parent_strat = if index == 0 {
                Just(Vec::new()).boxed()
            } else {
                prop::collection::btree_set(0..index, 0..=index.min(3))
                    .prop_map(|set| set.into_iter().collect::<Vec<_>>())
                    .boxed()
            };
            strategies.push(
                (
                    parent_strat,
                    0usize..3,
                    prop::collection::vec(any::<u8>(), 1..=8),
                )
                    .prop_map(move |(parents, role_index, disc)| DagNodeSpec {
                        occurrence_key: format!("occ.node.{index}"),
                        role_index,
                        parents,
                        disc,
                    }),
            );
        }
        strategies
    })
}

fn execute_dag(
    registry: &InstalledPersistentNameSchemeRegistry,
    nodes: &[DagNodeSpec],
    order: &[usize],
    slot_offset: u32,
    generation_offset: u32,
    reverse_parents: bool,
) -> BTreeMap<String, PersistentNameIdentity> {
    const ROLES: [&str; 3] = ["authored", "cut_piece", "seam_patch"];
    let mut minted_by_idx = BTreeMap::<usize, PersistentNameIdentity>::new();
    let mut minted_by_occ = BTreeMap::<String, PersistentNameIdentity>::new();
    let mut live_holders = BTreeMap::new();

    for (step, &node_idx) in order.iter().enumerate() {
        let spec = &nodes[node_idx];
        let mut parents: Vec<PersistentNameIdentity> = spec
            .parents
            .iter()
            .map(|parent_idx| minted_by_idx[parent_idx])
            .collect();
        if reverse_parents {
            parents.reverse();
        }
        let holder = entity_holder(
            slot_offset.saturating_add(step as u32),
            generation_offset.saturating_add(1),
        );
        let request = PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: spec.occurrence_key.clone(),
            role_path: ROLES[spec.role_index].to_owned(),
            parent_identities: parents,
            referenced_same_occurrence_roles: Vec::new(),
            canonical_discriminator: spec.disc.clone(),
            holder: holder.clone(),
        };
        let records = PersistentNameMintingAuthority::mint_commit_births(
            registry,
            &live_holders,
            &BTreeSet::new(),
            &[request],
        )
        .expect("acyclic distinct occurrences mint cleanly");
        let id = records[0].identity();
        live_holders.insert(id, holder);
        minted_by_idx.insert(node_idx, id);
        minted_by_occ.insert(spec.occurrence_key.clone(), id);
    }
    minted_by_occ
}

fn reverse_topological_order(nodes: &[DagNodeSpec]) -> Vec<usize> {
    let n = nodes.len();
    let mut indegree = vec![0usize; n];
    let mut children = vec![Vec::<usize>::new(); n];
    for (idx, spec) in nodes.iter().enumerate() {
        indegree[idx] = spec.parents.len();
        for &p in &spec.parents {
            children[p].push(idx);
        }
    }
    let mut ready: BTreeSet<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(next) = ready.pop_last() {
        order.push(next);
        for &child in &children[next] {
            indegree[child] -= 1;
            if indegree[child] == 0 {
                ready.insert(child);
            }
        }
    }
    order
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn allocation_and_topological_reorder_preserve_byte_identical_identities(
        dag in arb_birth_dag(),
        dummy_slot_offset in 1u32..50_000,
        dummy_generation_offset in 1u32..5_000,
    ) {
        let registry = face_registry();
        let natural_order: Vec<usize> = (0..dag.len()).collect();
        let permuted_order = reverse_topological_order(&dag);

        let baseline = execute_dag(&registry, &dag, &natural_order, 0, 1, false);
        let permuted = execute_dag(
            &registry,
            &dag,
            &permuted_order,
            dummy_slot_offset,
            dummy_generation_offset,
            true,
        );

        prop_assert_eq!(baseline, permuted);
    }
}
