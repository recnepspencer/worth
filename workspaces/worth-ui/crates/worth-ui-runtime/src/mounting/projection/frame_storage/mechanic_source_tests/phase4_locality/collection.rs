use super::*;

const LOCALITY_SIZES: [usize; 4] = [1, 32, 2_048, 4_096];

#[test]
fn collection_patch_content_locality_is_constant_at_every_qualified_size() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let mut observations = Vec::new();
    let mut expected_cost = None;
    let mut expected_collection_visits = None;

    for size in LOCALITY_SIZES {
        if size == 1 {
            observations.push(single_paragraph_observation(&fonts, &mut expected_cost));
            continue;
        }
        let mut source = UiMountedMechanicSource::default();
        let fixture = CollectionFixture::new(size - 1);
        fixture.install(&mut source, &fonts);
        assert_eq!(
            semantic_rows(&source, fixture.instance, fixture.surface, fixture.binding).len(),
            size,
        );
        let predecessor_layouts = source.collection_layouts_for_test(fixture.instance);
        source.begin_semantic_instance_index_observation();
        let mutation = fixture.patch_last(&mut source, &fonts);
        let index_work = crate::runtime::persistent_index::test_work();

        assert_eq!(mutation.semantic_text, 1);
        assert_eq!(mutation.command_changes.len(), 1);
        assert_constant_visits(
            &mut expected_collection_visits,
            index_work.iterated_entries(),
        );
        assert!(index_work.lookup_probes() <= 192);
        assert_one_collection_layout_changed(
            &predecessor_layouts,
            &source.collection_layouts_for_test(fixture.instance),
        );
        let changed = semantic_rows(&source, fixture.instance, fixture.surface, fixture.binding)
            .into_iter()
            .filter(|row| row.text() == "UPDATED")
            .collect::<Vec<_>>();
        assert_eq!(changed.len(), 1);
        let cost = source
            .qualified_layout(changed[0].qualified_layout_identity())
            .expect("changed row retains its qualified layout")
            .view()
            .cost();
        assert_constant_cost(&mut expected_cost, cost);
        observations.push(format!(
            "{{\"size\":{size},\"content\":{},\"lookup_probes\":{},\"local_row_visits\":{},\"sibling_visits\":0}}",
            cost_json(cost),
            index_work.lookup_probes(),
            index_work.iterated_entries(),
        ));
    }
    retained_scan_observation_detects_complete_collection_walk(&fonts);
    println!(
        "WORTH_UI_PHASE4_COLLECTION_LOCALITY={{\"observations\":[{}],\"changed_rows\":1}}",
        observations.join(",")
    );
}

fn assert_one_collection_layout_changed(
    predecessor: &std::collections::BTreeMap<[u8; 32], Arc<worth_ui_text::UiQualifiedTextLayout>>,
    successor: &std::collections::BTreeMap<[u8; 32], Arc<worth_ui_text::UiQualifiedTextLayout>>,
) {
    assert_eq!(predecessor.len(), successor.len());
    assert_eq!(
        predecessor
            .iter()
            .filter(|(identity, layout)| {
                !Arc::ptr_eq(
                    layout,
                    successor.get(*identity).expect("row identity retained"),
                )
            })
            .count(),
        1,
        "only the patched collection row may receive a new layout owner"
    );
}

fn single_paragraph_observation(
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    expected_cost: &mut Option<worth_ui_host_contract::UiQualifiedTextCostRecord>,
) -> String {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let mut source = UiMountedMechanicSource::default();
    let mut instances = Vec::new();
    append_paragraphs(&mut source, fonts, surface, binding, &mut instances, 1);
    let target = instances[0];
    source.begin_semantic_instance_index_observation();
    let mutation = replace_one_paragraph(
        &mut source,
        fonts,
        target,
        surface,
        binding,
        "UPDATED",
        160.0,
    );
    let index_work = crate::runtime::persistent_index::test_work();
    assert_eq!(mutation.semantic_text, 1);
    assert!(
        index_work.iterated_entries() <= 4,
        "one scalar paragraph may visit only its own retained rows"
    );
    let cost = layout(&source, target).view().cost();
    assert_constant_cost(expected_cost, cost);
    format!(
        "{{\"size\":1,\"content\":{},\"lookup_probes\":{},\"local_row_visits\":{},\"sibling_visits\":0}}",
        cost_json(cost),
        index_work.lookup_probes(),
        index_work.iterated_entries(),
    )
}

fn retained_scan_observation_detects_complete_collection_walk(
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
) {
    let mut source = UiMountedMechanicSource::default();
    let fixture = CollectionFixture::new(31);
    fixture.install(&mut source, fonts);
    source.begin_semantic_instance_index_observation();
    assert_eq!(source.retained_semantic_row_count_for_test(), 32);
    assert!(
        crate::runtime::persistent_index::test_work().iterated_entries() >= 32,
        "the locality oracle must observe a complete retained-row walk"
    );
}

struct CollectionFixture {
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    node: crate::graph::UiGraphNodeIdentity,
    initial: UiMountedSemanticTextSeed,
    last_identity: crate::mounting::UiMountedCollectionRowIdentity,
}

impl CollectionFixture {
    fn new(size: usize) -> Self {
        let rows = (0..size)
            .map(|index| row(index as u64, &format!("ROW-{index:04}")))
            .collect::<Vec<_>>();
        Self {
            instance: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            node: crate::graph::UiGraphNodeIdentity::new(140_000 + size as u64),
            initial: UiMountedSemanticTextSeed::collection_for_test(&rows),
            last_identity: identity((size - 1) as u64),
        }
    }

    fn install(
        &self,
        source: &mut UiMountedMechanicSource,
        fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    ) {
        self.apply(source, fonts, self.initial.clone(), 1, &[self.instance]);
    }

    fn patch_last(
        &self,
        source: &mut UiMountedMechanicSource,
        fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> crate::mounting::projection::frame_storage::mechanic_source::UiMountedMechanicMutation
    {
        let changes = [crate::mounting::UiMountedCollectionTextChange::Update(
            crate::mounting::UiMountedCollectionTextRow::new(
                self.last_identity.clone(),
                [Arc::from("UPDATED")],
            ),
        )];
        let successor =
            UiMountedSemanticTextSeed::collection_patch_for_test(&self.initial, &changes);
        self.apply(source, fonts, successor, 2, &[self.instance])
    }

    fn apply(
        &self,
        source: &mut UiMountedMechanicSource,
        fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
        seed: UiMountedSemanticTextSeed,
        generation: u64,
        changed: &[UiMountedInstanceIdentity],
    ) -> crate::mounting::projection::frame_storage::mechanic_source::UiMountedMechanicMutation
    {
        let semantic = semantic_projection_in(
            self.node,
            self.instance,
            self.surface,
            self.binding,
            seed,
            [0.0, 0.0, 160.0, 96.0],
        );
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let receipts = receipt_basis(frame, self.instance);
        source
            .apply(completion(
                frame,
                UiMountedContentGeneration::mint_unbound().unwrap(),
                &receipts,
                &semantic,
                fonts,
                changed,
                generation,
            ))
            .unwrap()
    }
}

fn row(local: u64, value: &str) -> crate::mounting::UiMountedCollectionTextRow {
    crate::mounting::UiMountedCollectionTextRow::new(identity(local), [Arc::from(value)])
}

fn identity(local: u64) -> crate::mounting::UiMountedCollectionRowIdentity {
    crate::mounting::UiMountedCollectionRowIdentity::from_query(
        &worth_ui_query_binding::certification::query_row_reference_fixture(local),
    )
}
