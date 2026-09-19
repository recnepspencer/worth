use worth_ui_host_contract::{
    UiMountedDrawableReference, UiMountedInstanceIdentity, UiMountedSemanticTextMechanic,
    UiMountedSemanticTextReference,
};

use super::super::UiMountedProjectionDenial;

pub(super) type UiMountedDrawableReferenceIndex =
    std::collections::BTreeMap<UiMountedInstanceIdentity, Box<[UiMountedDrawableReference]>>;

pub(super) fn drawable_reference_index(
    portal_overlays: &[worth_ui_host_contract::UiMountedPortalOverlayMechanic],
    semantic_text: &[UiMountedSemanticTextMechanic],
) -> Result<UiMountedDrawableReferenceIndex, UiMountedProjectionDenial> {
    let mut sources = std::collections::BTreeMap::<_, Vec<_>>::new();
    for (index, row) in portal_overlays.iter().enumerate() {
        let reference =
            worth_ui_host_contract::UiMountedPortalOverlayReference::from_runtime_mounting(
                u16::try_from(index)
                    .map_err(|_| UiMountedProjectionDenial::PortalOverlayCapacityExceeded)?,
            );
        sources.entry(row.owner()).or_default().push((
            row.layer_semantic_order(),
            UiMountedDrawableReference::PortalOverlay(reference),
        ));
    }
    append_semantic_text_sources(&mut sources, semantic_text)?;
    sources
        .into_iter()
        .map(|(instance, mut sources)| ordered_sources(instance, &mut sources))
        .collect()
}

pub(super) fn validate_drawable_coverage(
    sources: &UiMountedDrawableReferenceIndex,
    nodes: &[worth_ui_host_contract::UiMountedNodeProjectionView],
    table_row_count: usize,
) -> Result<(), UiMountedProjectionDenial> {
    let attached = nodes
        .iter()
        .map(|node| match sources.get(&node.mounted_instance()) {
            Some(expected) if expected.as_ref() == node.drawables() => Some(node.drawables().len()),
            None if node.drawables().is_empty() => Some(0),
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    let Some(attached) = attached else {
        return Err(UiMountedProjectionDenial::DrawableSourceCoverageMismatch);
    };
    let attached_rows = attached.into_iter().sum::<usize>();
    let attached_instances = nodes
        .iter()
        .filter(|node| !node.drawables().is_empty())
        .count();
    if attached_rows != table_row_count || attached_instances != sources.len() {
        return Err(UiMountedProjectionDenial::DrawableSourceCoverageMismatch);
    }
    Ok(())
}

fn append_semantic_text_sources(
    sources: &mut std::collections::BTreeMap<
        UiMountedInstanceIdentity,
        Vec<(u32, UiMountedDrawableReference)>,
    >,
    semantic_text: &[UiMountedSemanticTextMechanic],
) -> Result<(), UiMountedProjectionDenial> {
    for (index, row) in semantic_text.iter().enumerate() {
        let reference = UiMountedSemanticTextReference::from_runtime_mounting(
            u16::try_from(index)
                .map_err(|_| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?,
        );
        sources.entry(row.mounted_instance()).or_default().push((
            row.layer_semantic_order(),
            UiMountedDrawableReference::SemanticText(reference),
        ));
    }
    Ok(())
}

fn ordered_sources(
    instance: UiMountedInstanceIdentity,
    sources: &mut Vec<(u32, UiMountedDrawableReference)>,
) -> Result<(UiMountedInstanceIdentity, Box<[UiMountedDrawableReference]>), UiMountedProjectionDenial>
{
    sources.sort_by_key(|(layer, _)| *layer);
    if let Some(layer) = duplicate_layer(sources) {
        return Err(UiMountedProjectionDenial::AmbiguousDrawableOrder { instance, layer });
    }
    Ok((
        instance,
        sources.drain(..).map(|(_, reference)| reference).collect(),
    ))
}

fn duplicate_layer(sources: &[(u32, UiMountedDrawableReference)]) -> Option<u32> {
    sources
        .windows(2)
        .find_map(|pair| (pair[0].0 == pair[1].0).then_some(pair[0].0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orphan_drawable_source_is_rejected_before_projection_issuance() {
        let instance = UiMountedInstanceIdentity::mint_unbound().expect("mounted instance");
        let sources = std::collections::BTreeMap::from([(
            instance,
            vec![UiMountedDrawableReference::SemanticText(
                UiMountedSemanticTextReference::from_runtime_mounting(0),
            )]
            .into_boxed_slice(),
        )]);
        assert_eq!(
            validate_drawable_coverage(&sources, &[], 1),
            Err(UiMountedProjectionDenial::DrawableSourceCoverageMismatch)
        );
    }
}
