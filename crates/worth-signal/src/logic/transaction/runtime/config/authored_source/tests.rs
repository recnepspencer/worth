use super::*;
use crate::data::aspect::{Aspect, AspectVersion};
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::node::NodeContract;
use crate::data::output::{NodeEvaluationResult, StructuralMemoKey};
use crate::data::tier::{DependencyMode, DirtyPropagation, EvaluationTrigger};
use crate::durability::authored_image::{read_runtime_rows, write_runtime_rows, Write};
use worth_foundational::facade::*;

#[test]
fn runtime_declarations_rebind_stable_tier_names_and_keep_memo_values_cold() {
    let branch = DurableSignalBranchIdentity::from_recorded(
        DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([3; 16]).unwrap(),
            SignalOwnerSlotIdentity::from_recorded(1),
        ),
        0,
    );
    let vocabulary = SignalDurableTierVocabulary::declare([(2_u8, "interactive")]).unwrap();
    let mut config = SignalRuntimeConfig::new();
    let policy = TierPolicy::new(
        2,
        DependencyMode::AutoDiscovered,
        DirtyPropagation::Immediate,
        EvaluationTrigger::OnDemand,
    )
    .with_default_comparator(VersionComparatorPolicy::Tolerance { epsilon: 19 });
    config.set_tier_policy(policy);
    config.set_fallback_comparator(VersionComparatorPolicy::Custom {
        key: "fresh-program-provider".into(),
    });
    config.set_resource_runtime_deadline(
        crate::data::temporal::TemporalDuration::new("deadline", 71).unwrap(),
    );
    let family = config
        .define_computation(
            "totals",
            NodeContract::default(),
            2,
            VersionComparatorPolicy::Exact,
        )
        .unwrap();
    let key = ComputationKey::new("customer-17");
    // Descriptive rows exercise the private registration codec without creating
    // a runtime root or claiming operational branch admission.
    let node = crate::data::handle::NodeId::new(0, 1);
    config.node_meta.ensure_capacity(1);
    config.node_meta.set_tier(node, 2);
    let family_id = config.key_registry.intern_family(&family);
    let key_id = config.key_registry.intern_key(&key);
    config.keyed_nodes.insert((family_id, key_id), node);
    let before = encoded_declarations(&config, node, &vocabulary, branch);
    config.store_memoized_result(
        &family,
        &key,
        &StructuralMemoKey::new("memo-only-key"),
        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
            Aspect::new(0),
            0xDEAD_BEEF,
        )])),
    );
    let image = encoded_declarations(&config, node, &vocabulary, branch);
    assert_eq!(before, image);
    assert!(!image.windows(13).any(|bytes| bytes == b"memo-only-key"));
    let rebound_vocabulary =
        SignalDurableTierVocabulary::declare([(99_u16, "interactive")]).unwrap();
    let recovered = SignalRuntimeConfig::readmit_authored_rows(
        read_runtime_rows(&image, true, 97).unwrap(),
        &rebound_vocabulary,
        |candidate| candidate == node,
    )
    .unwrap();
    assert_eq!(recovered.test_registry_counts(), (1, 1, 0, 1, 0));
    assert_eq!(recovered.node_meta().tier_for_node(node), Some(99));
    let recovered_family = *recovered.key_registry.family_lookup.get(&family).unwrap();
    let recovered_key = *recovered.key_registry.key_lookup.get(&key).unwrap();
    assert_eq!(
        recovered
            .keyed_nodes
            .get(&(recovered_family, recovered_key)),
        Some(&node)
    );
    assert_eq!(
        encoded_declarations(&recovered, node, &rebound_vocabulary, branch),
        image
    );
    assert_eq!(
        recovered.resource_runtime_deadline(),
        config.resource_runtime_deadline()
    );
    let missing = SignalDurableTierVocabulary::<u16>::declare([]).unwrap();
    assert!(matches!(
        SignalRuntimeConfig::readmit_authored_rows(
            read_runtime_rows(&image, true, 97).unwrap(),
            &missing,
            |_| true
        ),
        Err(Denial::TierVocabulary(
            crate::durability::SignalDurableTierVocabularyDenial::UnknownTierName(_)
        ))
    ));
}

fn encoded_declarations<T: Copy + Ord>(
    config: &SignalRuntimeConfig<T>,
    node: crate::data::handle::NodeId,
    vocabulary: &SignalDurableTierVocabulary<T>,
    branch: DurableSignalBranchIdentity,
) -> Vec<u8> {
    let view = config.authored_definitions([node], vocabulary).unwrap();
    let mut bytes = Vec::new();
    let mut emit = |part: &[u8]| {
        bytes.extend_from_slice(part);
        Ok(())
    };
    let mut output = Write::new(branch, &mut emit);
    output.prefix().unwrap();
    write_runtime_rows(&mut output, &view).unwrap();
    bytes
}
