use super::*;

pub(super) fn seed_pseudorealistic_rocketship_world(
    runtime: &RelationalRuntime,
    node_count: usize,
    query_target_count: usize,
) -> RocketshipPseudoRealisticSeedOutcome {
    let total_weight: usize = ROCKETSHIP_SUBSYSTEM_LAYOUTS
        .iter()
        .map(|layout| layout.weight)
        .sum();
    let mut assigned = 0usize;
    let mut subsystem_ranges = Vec::with_capacity(ROCKETSHIP_SUBSYSTEM_LAYOUTS.len());

    let entity_commit_started_at = Instant::now();
    let entity_outcome = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
        let mut batch = WorkerIntentBatch::new("rocketship-pseudorealistic-entities");
        let mut entity_specs = Vec::with_capacity(node_count);
        for (layout_index, layout) in ROCKETSHIP_SUBSYSTEM_LAYOUTS.iter().enumerate() {
            let remaining_layouts = ROCKETSHIP_SUBSYSTEM_LAYOUTS.len() - layout_index;
            let remaining_nodes = node_count.saturating_sub(assigned);
            let subsystem_count = if remaining_layouts == 1 {
                remaining_nodes
            } else {
                ((node_count * layout.weight) / total_weight).max(512)
            }
            .min(remaining_nodes.saturating_sub(remaining_layouts - 1));
            let start = assigned;
            let end = start + subsystem_count;
            assigned = end;
            subsystem_ranges.push((start, end, *layout));

            for local_index in 0..subsystem_count {
                let aspect = match local_index % 4 {
                    0 => "structure",
                    1 => "thermal",
                    2 => "fluid",
                    _ => "control",
                };
                let partition_id = PartitionId(
                    layout.partition_base
                        + (local_index % ROCKETSHIP_SUBSYSTEM_ENTITY_PARTITION_FANOUT) as u32,
                );
                entity_specs.push(crate::transactions::data::EntitySpec {
                    partition_id,
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(format!(
                        "rocket.{}.{}.{}",
                        layout.section, layout.subsystem, local_index
                    )),
                    fields: crate::tests::support::aspect_field_patch_from_values([
                        (
                            crate::tests::support::aspect_key("section"),
                            crate::tests::support::field_key("section"),
                            crate::tests::support::string_aspect_value(layout.section),
                        ),
                        (
                            crate::tests::support::aspect_key("subsystem"),
                            crate::tests::support::field_key("subsystem"),
                            crate::tests::support::string_aspect_value(layout.subsystem),
                        ),
                        (
                            crate::tests::support::aspect_key("aspect"),
                            crate::tests::support::field_key("aspect"),
                            crate::tests::support::string_aspect_value(aspect),
                        ),
                        (
                            crate::tests::support::aspect_key("ordinal"),
                            crate::tests::support::field_key("ordinal"),
                            crate::tests::support::usize_aspect_value(local_index),
                        ),
                    ]),
                });
            }
        }
        for intent in bulk_entity_create_intents(&entity_specs) {
            batch = batch.push(intent);
        }
        txn.push_batch(batch)
            .expect("test staging stays within configured resource budgets");
        txn.commit(runtime)
            .expect("pseudorealistic rocketship entity seed commit")
    };
    let entity_commit_micros = entity_commit_started_at.elapsed().as_micros();
    assert_eq!(
        changed_entities(&entity_outcome).len(),
        node_count,
        "pseudorealistic rocketship should seed all entities"
    );
    let entities = rebuild_pseudorealistic_entity_order(runtime, &subsystem_ranges, node_count);

    let mut relation_specs = Vec::new();
    let mut mixed_query_targets = Vec::new();
    let mut traversal_seeds = Vec::new();

    for (range_index, (start, end, layout)) in subsystem_ranges.iter().enumerate() {
        let subsystem_entities = &entities[*start..*end];
        for local_index in 0..subsystem_entities.len().saturating_sub(1) {
            relation_specs.push(crate::transactions::data::RelationSpec {
                partition_id: PartitionId(201 + ((range_index + local_index) % 32) as u32),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(format!(
                    "rocket.local.{}.{}.{}",
                    layout.section, layout.subsystem, local_index
                )),
                source: crate::transactions::data::EntityReference::Existing(
                    subsystem_entities[local_index],
                ),
                target: crate::transactions::data::EntityReference::Existing(
                    subsystem_entities[local_index + 1],
                ),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            });
            if local_index + 8 < subsystem_entities.len() && local_index % 16 == 0 {
                relation_specs.push(crate::transactions::data::RelationSpec {
                    partition_id: PartitionId(301 + ((range_index + local_index) % 32) as u32),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw(format!(
                        "rocket.aspect.{}.{}.{}",
                        layout.section, layout.subsystem, local_index
                    )),
                    source: crate::transactions::data::EntityReference::Existing(
                        subsystem_entities[local_index],
                    ),
                    target: crate::transactions::data::EntityReference::Existing(
                        subsystem_entities[local_index + 8],
                    ),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                });
            }
        }

        let midpoint = subsystem_entities.len() / 2;
        mixed_query_targets.push(RecordRef::Entity(subsystem_entities[midpoint]));
        traversal_seeds.push(subsystem_entities[midpoint]);
        if subsystem_entities.len() > 64 {
            mixed_query_targets.push(RecordRef::Entity(
                subsystem_entities[subsystem_entities.len() / 4],
            ));
            mixed_query_targets.push(RecordRef::Entity(
                subsystem_entities[(subsystem_entities.len() * 3) / 4],
            ));
        }
    }

    for pair in subsystem_ranges.windows(2) {
        let (left_start, left_end, left_layout) = pair[0];
        let (right_start, right_end, right_layout) = pair[1];
        let left_entities = &entities[left_start..left_end];
        let right_entities = &entities[right_start..right_end];
        let interface_stride = (left_entities.len().min(right_entities.len()) / 96).max(1);
        for interface_index in
            (0..left_entities.len().min(right_entities.len())).step_by(interface_stride)
        {
            relation_specs.push(crate::transactions::data::RelationSpec {
                partition_id: PartitionId(401 + (interface_index % 32) as u32),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(format!(
                    "rocket.interface.{}.{}.{}.{}",
                    left_layout.section,
                    left_layout.subsystem,
                    right_layout.section,
                    interface_index
                )),
                source: crate::transactions::data::EntityReference::Existing(
                    left_entities[interface_index],
                ),
                target: crate::transactions::data::EntityReference::Existing(
                    right_entities[interface_index],
                ),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            });
        }
    }

    let guidance_anchor = traversal_seeds[0];
    let avionics_anchor = traversal_seeds[1];
    let engine_anchor = traversal_seeds[9];
    let plumbing_anchor = traversal_seeds[10];
    let fin_anchor = traversal_seeds[11];
    relation_specs.push(crate::transactions::data::RelationSpec {
        partition_id: PartitionId(501),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("rocket.control.guidance-avionics"),
        source: crate::transactions::data::EntityReference::Existing(guidance_anchor),
        target: crate::transactions::data::EntityReference::Existing(avionics_anchor),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    });
    relation_specs.push(crate::transactions::data::RelationSpec {
        partition_id: PartitionId(502),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("rocket.control.avionics-engine"),
        source: crate::transactions::data::EntityReference::Existing(avionics_anchor),
        target: crate::transactions::data::EntityReference::Existing(engine_anchor),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    });
    relation_specs.push(crate::transactions::data::RelationSpec {
        partition_id: PartitionId(503),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("rocket.feed.plumbing-engine"),
        source: crate::transactions::data::EntityReference::Existing(plumbing_anchor),
        target: crate::transactions::data::EntityReference::Existing(engine_anchor),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    });
    relation_specs.push(crate::transactions::data::RelationSpec {
        partition_id: PartitionId(504),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("rocket.control.avionics-fin"),
        source: crate::transactions::data::EntityReference::Existing(avionics_anchor),
        target: crate::transactions::data::EntityReference::Existing(fin_anchor),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    });

    let relation_count = relation_specs.len();
    let mut relation_commit_micros = 0u128;
    let mut relation_commit_phase_timing = crate::transactions::data::CommitPhaseTiming::default();
    for (chunk_index, relation_chunk) in relation_specs
        .chunks(ROCKETSHIP_RELATION_SEED_BATCH_SIZE)
        .enumerate()
    {
        let relation_commit_started_at = Instant::now();
        let outcome = {
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
            let mut batch = WorkerIntentBatch::new(format!(
                "rocketship-pseudorealistic-relations-bulk-{chunk_index}"
            ));
            for intent in bulk_relation_create_intents(relation_chunk) {
                batch = batch.push(intent);
            }
            txn.push_batch(batch)
                .expect("test staging stays within configured resource budgets");
            txn.commit(runtime)
                .expect("pseudorealistic rocketship relation seed commit chunk")
        };
        relation_commit_micros += relation_commit_started_at.elapsed().as_micros();
        relation_commit_phase_timing.draft_preparation_micros +=
            outcome.execution().phase_timing.draft_preparation_micros;
        relation_commit_phase_timing.draft_bulk_admission_micros +=
            outcome.execution().phase_timing.draft_bulk_admission_micros;
        relation_commit_phase_timing.draft_merge_plan_micros +=
            outcome.execution().phase_timing.draft_merge_plan_micros;
        relation_commit_phase_timing.draft_structural_summary_micros += outcome
            .execution()
            .phase_timing
            .draft_structural_summary_micros;
        relation_commit_phase_timing.draft_working_state_clone_micros += outcome
            .execution()
            .phase_timing
            .draft_working_state_clone_micros;
        relation_commit_phase_timing.working_state_preparation_micros += outcome
            .execution()
            .phase_timing
            .working_state_preparation_micros;
        relation_commit_phase_timing.invariant_pre_check_micros +=
            outcome.execution().phase_timing.invariant_pre_check_micros;
        relation_commit_phase_timing.authoritative_mutation_micros += outcome
            .execution()
            .phase_timing
            .authoritative_mutation_micros;
        relation_commit_phase_timing.history_resolution_micros +=
            outcome.execution().phase_timing.history_resolution_micros;
        relation_commit_phase_timing.invariant_post_check_micros +=
            outcome.execution().phase_timing.invariant_post_check_micros;
        relation_commit_phase_timing.artifact_assembly_micros +=
            outcome.execution().phase_timing.artifact_assembly_micros;
        relation_commit_phase_timing.durable_append_micros +=
            outcome.execution().phase_timing.durable_append_micros;
        relation_commit_phase_timing.publication_micros +=
            outcome.execution().phase_timing.publication_micros;
        relation_commit_phase_timing.publication_storage_commit_micros += outcome
            .execution()
            .phase_timing
            .publication_storage_commit_micros;
        assert_eq!(changed_relations(&outcome).len(), relation_chunk.len());
    }

    mixed_query_targets.truncate(query_target_count.max(ROCKETSHIP_SUBSYSTEM_LAYOUTS.len()));
    let hot_update_target =
        entities[subsystem_ranges[9].0 + ((subsystem_ranges[9].1 - subsystem_ranges[9].0) / 2)];

    RocketshipPseudoRealisticSeedOutcome {
        entities,
        mixed_query_targets,
        traversal_seeds,
        hot_update_target,
        relation_count,
        subsystem_count: ROCKETSHIP_SUBSYSTEM_LAYOUTS.len(),
        entity_commit_micros,
        relation_commit_micros,
        relation_commit_phase_timing,
    }
}

pub(super) fn rebuild_pseudorealistic_entity_order(
    runtime: &RelationalRuntime,
    subsystem_ranges: &[(usize, usize, RocketshipSubsystemLayout)],
    node_count: usize,
) -> Vec<crate::facade::identity::EntityId> {
    let expected_ranges = subsystem_ranges
        .iter()
        .map(|(start, end, layout)| ((layout.section, layout.subsystem), (*start, *end)))
        .collect::<BTreeMap<_, _>>();
    let snapshot = runtime.visibility_authority().snapshot();
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("pseudorealistic entity snapshot");
    let mut ordered = vec![None; node_count];

    for record in read.entities() {
        if record.kind.kind_id != KindId(1) {
            continue;
        }
        let Some(section) = read_entity_field(record, field_key("section")) else {
            continue;
        };
        let Some(subsystem) = read_entity_field(record, field_key("subsystem")) else {
            continue;
        };
        let Some(ordinal) = read_entity_field(record, field_key("ordinal"))
            .and_then(|value| value.parse::<usize>().ok())
        else {
            continue;
        };
        let Some((start, end)) = expected_ranges
            .get(&(section.as_str(), subsystem.as_str()))
            .copied()
        else {
            continue;
        };
        assert!(
            ordinal < end - start,
            "pseudorealistic entity ordinal must fit subsystem range"
        );
        ordered[start + ordinal] = Some(record.entity_id);
    }

    let released = runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok();
    assert!(
        released,
        "pseudorealistic entity reorder snapshot should release"
    );

    ordered
        .into_iter()
        .enumerate()
        .map(|(index, entity)| {
            entity.unwrap_or_else(|| panic!("missing pseudorealistic entity ordering slot {index}"))
        })
        .collect()
}
