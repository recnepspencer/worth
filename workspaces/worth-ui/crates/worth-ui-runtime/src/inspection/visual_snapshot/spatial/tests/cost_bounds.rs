use super::*;

#[test]
fn production_maximum_overlap_is_exactly_bounded_and_fully_accounted() {
    let maximum = usize::from(
        worth_ui_inspection::UiVisualInspectionPolicy::production_default(
            worth_ui_inspection::UiVisualInspectionDisclosure::redacted(
                worth_ui_inspection::UiVisualInspectionAudience::DiagnosticAgent,
            ),
        )
        .unwrap()
        .maximum_query_candidates(),
    );
    assert_eq!(maximum, 4_096);
    let world = SpatialWorld::new();
    let paint = (0..maximum)
        .map(|order| {
            paint(
                &world,
                bounds(0.0, 0.0, 8.0, 8.0),
                u32::try_from(order).unwrap(),
                u8::MAX,
            )
        })
        .collect::<Vec<_>>();
    let observed = paint
        .iter()
        .map(|row| observed_paint(*row, row.semantic_order()))
        .collect::<Vec<_>>();
    let basis =
        crate::mounting::UiMountedVisualRegionBasis::new(paint.into_boxed_slice(), Box::new([]));
    let indexed = validate_and_index(19, &basis, &observed, transform([16, 16]))
        .expect("the production maximum is an admitted finite snapshot");
    let (visible, _, cost) = indexed.into_parts();
    assert_eq!(cost.region_records_examined(), maximum);
    assert!(cost.retained_structural_bytes() > maximum);

    let (complete, complete_probes, complete_exhausted) =
        visible.point_candidates(point(4, 4), maximum).into_parts();
    assert_eq!(complete.len(), maximum);
    assert!(!complete_exhausted);
    assert!(complete_probes <= maximum.saturating_mul(2).saturating_add(1));

    let (truncated, truncated_probes, truncated_exhausted) = visible
        .point_candidates(point(4, 4), maximum - 1)
        .into_parts();
    assert_eq!(truncated.len(), maximum - 1);
    assert!(truncated_exhausted);
    assert!(truncated_probes <= complete_probes);
}

#[test]
fn generated_record_limits_have_exact_representation_costs() {
    for record_count in [1_usize, 1_024, 65_536] {
        let world = SpatialWorld::new();
        let paint = (0..record_count)
            .map(|index| {
                paint(
                    &world,
                    bounds(index as f32, 0.0, 1.0, 1.0),
                    u32::try_from(index).unwrap(),
                    u8::MAX,
                )
            })
            .collect::<Vec<_>>();
        let observed = paint
            .iter()
            .map(|row| observed_paint(*row, row.semantic_order()))
            .collect::<Vec<_>>();
        let basis = crate::mounting::UiMountedVisualRegionBasis::new(
            paint.into_boxed_slice(),
            Box::new([]),
        );
        let indexed = validate_and_index(
            23,
            &basis,
            &observed,
            transform([u32::try_from(record_count).unwrap(), 2]),
        )
        .expect("the declared finite record limit validates and indexes");
        let (visible, hit_test, cost) = indexed.into_parts();
        let estimated =
            super::super::UiVisibleRegionIndex::estimated_retained_structural_bytes(record_count)
                .and_then(|bytes| {
                    bytes.checked_add(
                        super::super::UiHitTestRegionIndex::estimated_retained_structural_bytes(0)?,
                    )
                })
                .expect("the declared record limit has a representable cost");

        assert_eq!(visible.len(), record_count);
        assert_eq!(hit_test.len(), 0);
        assert_eq!(cost.region_records_examined(), record_count);
        assert_eq!(cost.retained_structural_bytes(), estimated);
    }
}
