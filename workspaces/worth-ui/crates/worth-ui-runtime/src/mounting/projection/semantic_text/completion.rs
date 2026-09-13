use std::sync::Arc;

use worth_ui_host_contract::{
    UiMountedCollectionRowCorrelation, UiMountedSemanticTextCompletionInput,
    UiMountedSemanticTextMechanic, UiSemanticTextProfile, UiSemanticTextSlot,
};

use super::super::frame_storage::{UiMountedProjectionNodeRecord, UiMountedSemanticProjection};
use super::super::UiMountedProjectionDenial;

pub(in crate::mounting::projection) fn complete_node_semantic_text(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    node: &UiMountedProjectionNodeRecord,
) -> Result<Vec<super::UiMountedQualifiedSemanticText>, UiMountedProjectionDenial> {
    let mut rows = Vec::new();
    if let Some(seed) = node.semantic_text.as_ref() {
        push_node_rows(context, node, seed, &mut rows)?;
    }
    Ok(rows)
}

pub(in crate::mounting::projection) struct UiMountedSemanticTextCompletionContext<'a> {
    pub frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub content_generation: worth_ui_host_contract::UiMountedContentGeneration,
    pub receipt_basis: &'a super::super::super::UiMountedNodeReceiptBasis,
    pub semantic: &'a UiMountedSemanticProjection,
    pub capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub capability_profile_digest: u64,
    pub font_collection: &'a Arc<worth_ui_text::UiGlobalFontCollection>,
    pub qualification_cache: &'a super::UiMountedTextQualificationCache,
}

fn push_node_rows(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    node: &UiMountedProjectionNodeRecord,
    seed: &super::UiMountedSemanticTextSeed,
    rows: &mut Vec<super::UiMountedQualifiedSemanticText>,
) -> Result<(), UiMountedProjectionDenial> {
    let surface = context
        .semantic
        .surface_for(node.receipt.semantic_surface())
        .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
    let (bounds, allocation_basis) = super::require_allocation(node, surface)?;
    let mounted_instance = node.receipt.mounted_instance();
    let node_receipt = context
        .receipt_basis
        .receipt_for(mounted_instance)
        .ok_or(UiMountedProjectionDenial::SemanticTextNodeReceiptMismatch)?;
    let value_row_count = value_row_count(seed)?;
    push_value_rows(
        context,
        seed,
        UiMountedNodeTextBasis {
            surface,
            mounted_instance,
            portal_group: node.completed_appearance_geometry().portal_group(),
            node_receipt,
            allocation_basis,
            bounds,
        },
        rows,
    )?;
    push_row(
        context,
        UiMountedSemanticTextRowBasis {
            surface,
            mounted_instance,
            portal_group: node.completed_appearance_geometry().portal_group(),
            node_receipt,
            allocation_basis,
            bounds,
            origin_x: bounds.x(),
            origin_y: super::geometry::row_origin(bounds, value_row_count, value_row_count + 1),
            text: Arc::clone(seed.posture()),
            slot: UiSemanticTextSlot::Posture,
            collection_row: None,
            layer_semantic_order: seed
                .layer_semantic_order()
                .checked_add(
                    u32::try_from(value_row_count)
                        .map_err(|_| UiMountedProjectionDenial::SemanticTextLayerOrderExceeded)?,
                )
                .ok_or(UiMountedProjectionDenial::SemanticTextLayerOrderExceeded)?,
            formatting: seed.formatting().default_row(),
        },
        rows,
    )
}

pub(in crate::mounting::projection) fn complete_semantic_text_replacement(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    node: &UiMountedProjectionNodeRecord,
    predecessor: &UiMountedSemanticTextMechanic,
    text: &Arc<str>,
    formatting: super::formatting::UiMountedSemanticTextRowFormatting<'_>,
) -> Result<super::UiMountedQualifiedSemanticText, UiMountedProjectionDenial> {
    let surface = context
        .semantic
        .surface_for(node.receipt.semantic_surface())
        .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
    let receipt = context
        .receipt_basis
        .receipt_for(node.receipt.mounted_instance())
        .ok_or(UiMountedProjectionDenial::SemanticTextNodeReceiptMismatch)?;
    let qualified =
        super::qualification::qualify_layout(context, text, predecessor.bounds(), formatting)?;
    let mechanic = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: context.content_generation,
            frame: context.frame,
            surface: surface.surface,
            binding: surface.binding,
            mounted_instance: node.receipt.mounted_instance(),
            portal_group: predecessor.portal_group(),
            node_receipt: receipt,
            allocation_basis: predecessor.allocation_basis(),
            bounds: predecessor.bounds(),
            clip_bounds: predecessor.clip_bounds(),
            origin_x: predecessor.origin_x(),
            origin_y: predecessor.origin_y(),
            text: Arc::clone(text),
            layout: qualified.layout().view(),
            slot: predecessor.slot(),
            collection_row: predecessor.collection_row().cloned(),
            foregrounds: Arc::clone(qualified.foregrounds()),
            profile: predecessor.profile(),
            layer_semantic_order: predecessor.layer_semantic_order(),
            capability_generation: context.capability_generation,
            capability_profile_digest: context.capability_profile_digest,
        },
    )
    .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?;
    Ok(super::UiMountedQualifiedSemanticText::new(
        mechanic,
        Arc::clone(qualified.layout()),
    ))
}

#[derive(Clone, Copy)]
struct UiMountedNodeTextBasis {
    surface: super::super::frame_storage::UiMountedProjectionSurface,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    portal_group: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
}

struct UiMountedSemanticTextRowBasis<'formatting> {
    surface: super::super::frame_storage::UiMountedProjectionSurface,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    portal_group: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    origin_x: f32,
    origin_y: f32,
    text: Arc<str>,
    slot: UiSemanticTextSlot,
    collection_row: Option<UiMountedCollectionRowCorrelation>,
    layer_semantic_order: u32,
    formatting: super::formatting::UiMountedSemanticTextRowFormatting<'formatting>,
}

enum UiMountedSemanticTextValueMeaning {
    Scalar(Arc<str>),
    Collection {
        text: Arc<str>,
        row: UiMountedCollectionRowCorrelation,
        selected_field_ordinal: u16,
    },
}

struct UiMountedSemanticTextValueRowInput {
    basis: UiMountedNodeTextBasis,
    meaning: UiMountedSemanticTextValueMeaning,
    index: usize,
    total: usize,
}

fn push_row(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    row: UiMountedSemanticTextRowBasis<'_>,
    rows: &mut Vec<super::UiMountedQualifiedSemanticText>,
) -> Result<(), UiMountedProjectionDenial> {
    if rows.len() >= worth_ui_host_contract::UiMountedSemanticTextTable::MAX_ROWS {
        return Err(UiMountedProjectionDenial::SemanticTextCapacityExceeded);
    }
    let qualified =
        super::qualification::qualify_layout(context, &row.text, row.bounds, row.formatting)?;
    let mechanic = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: context.content_generation,
            frame: context.frame,
            surface: row.surface.surface,
            binding: row.surface.binding,
            mounted_instance: row.mounted_instance,
            portal_group: row.portal_group,
            node_receipt: row.node_receipt,
            allocation_basis: row.allocation_basis,
            bounds: row.bounds,
            clip_bounds: row.bounds,
            origin_x: row.origin_x,
            origin_y: row.origin_y,
            text: row.text,
            layout: qualified.layout().view(),
            slot: row.slot,
            collection_row: row.collection_row,
            foregrounds: Arc::clone(qualified.foregrounds()),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: row.layer_semantic_order,
            capability_generation: context.capability_generation,
            capability_profile_digest: context.capability_profile_digest,
        },
    )
    .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?;
    rows.push(super::UiMountedQualifiedSemanticText::new(
        mechanic,
        Arc::clone(qualified.layout()),
    ));
    Ok(())
}

fn value_row_count(
    seed: &super::UiMountedSemanticTextSeed,
) -> Result<usize, UiMountedProjectionDenial> {
    match seed.content() {
        super::UiMountedSemanticTextSeedContent::Scalar(value) => Ok(usize::from(value.is_some())),
        super::UiMountedSemanticTextSeedContent::Collection(rows) => {
            Ok(rows.selected_value_count())
        }
    }
}

fn push_value_rows(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    seed: &super::UiMountedSemanticTextSeed,
    basis: UiMountedNodeTextBasis,
    rows: &mut Vec<super::UiMountedQualifiedSemanticText>,
) -> Result<(), UiMountedProjectionDenial> {
    match seed.content() {
        super::UiMountedSemanticTextSeedContent::Scalar(Some(value)) => push_value_row(
            context,
            seed,
            UiMountedSemanticTextValueRowInput {
                basis,
                meaning: UiMountedSemanticTextValueMeaning::Scalar(Arc::clone(value)),
                index: 0,
                total: 2,
            },
            rows,
        ),
        super::UiMountedSemanticTextSeedContent::Scalar(None) => Ok(()),
        super::UiMountedSemanticTextSeedContent::Collection(collection) => {
            let total = value_row_count(seed)? + 1;
            let mut index = 0usize;
            for row in collection.rows() {
                for (field_ordinal, value) in row.selected_values().iter().enumerate() {
                    push_value_row(
                        context,
                        seed,
                        UiMountedSemanticTextValueRowInput {
                            basis,
                            meaning: UiMountedSemanticTextValueMeaning::Collection {
                                text: Arc::clone(value),
                                row: UiMountedCollectionRowCorrelation::from_runtime_mounting(
                                    row.identity()
                                        .query_reference()
                                        .query_identity()
                                        .operational_key()
                                        .correlation_digest(),
                                ),
                                selected_field_ordinal: u16::try_from(field_ordinal).map_err(
                                    |_| UiMountedProjectionDenial::SemanticTextCapacityExceeded,
                                )?,
                            },
                            index,
                            total,
                        },
                        rows,
                    )?;
                    index += 1;
                }
            }
            Ok(())
        }
    }
}

fn push_value_row(
    context: &UiMountedSemanticTextCompletionContext<'_>,
    seed: &super::UiMountedSemanticTextSeed,
    input: UiMountedSemanticTextValueRowInput,
    rows: &mut Vec<super::UiMountedQualifiedSemanticText>,
) -> Result<(), UiMountedProjectionDenial> {
    let (text, slot, collection_row) = match input.meaning {
        UiMountedSemanticTextValueMeaning::Scalar(text) => (text, UiSemanticTextSlot::Value, None),
        UiMountedSemanticTextValueMeaning::Collection {
            text,
            row,
            selected_field_ordinal,
        } => (
            text,
            UiSemanticTextSlot::CollectionValue {
                selected_field_ordinal,
            },
            Some(row),
        ),
    };
    push_row(
        context,
        UiMountedSemanticTextRowBasis {
            surface: input.basis.surface,
            mounted_instance: input.basis.mounted_instance,
            portal_group: input.basis.portal_group,
            node_receipt: input.basis.node_receipt,
            allocation_basis: input.basis.allocation_basis,
            bounds: input.basis.bounds,
            origin_x: input.basis.bounds.x(),
            origin_y: super::geometry::row_origin(input.basis.bounds, input.index, input.total),
            text,
            slot,
            collection_row,
            layer_semantic_order: seed
                .layer_semantic_order()
                .checked_add(
                    u32::try_from(input.index)
                        .map_err(|_| UiMountedProjectionDenial::SemanticTextLayerOrderExceeded)?,
                )
                .ok_or(UiMountedProjectionDenial::SemanticTextLayerOrderExceeded)?,
            formatting: match slot {
                UiSemanticTextSlot::Value => seed.formatting().scalar_value_row(),
                UiSemanticTextSlot::CollectionValue { .. } | UiSemanticTextSlot::Posture => {
                    seed.formatting().default_row()
                }
            },
        },
        rows,
    )
}
