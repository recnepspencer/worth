use std::sync::Arc;

use super::super::UiMountedProjectionDenial;
use super::UiMountedSemanticTextFormattingSeed;

mod collection_source;

pub(in crate::mounting::projection) use collection_source::{
    UiMountedCollectionTextKey, UiMountedCollectionTextSource,
};

#[derive(Clone)]
pub(in crate::mounting::projection) struct UiMountedSemanticTextSeed {
    content: UiMountedSemanticTextSeedContent,
    posture: Arc<str>,
    formatting: UiMountedSemanticTextFormattingSeed,
    layer_semantic_order: u32,
    transition: UiMountedSemanticTextSeedTransition,
}

#[derive(Clone)]
pub(in crate::mounting::projection) enum UiMountedSemanticTextSeedContent {
    Scalar(Option<Arc<str>>),
    Collection(UiMountedCollectionTextSource),
}

#[derive(Clone)]
pub(in crate::mounting::projection) enum UiMountedSemanticTextSeedTransition {
    Retained,
    PaintOnly,
    Complete,
    CollectionPatch(Arc<[crate::mounting::UiMountedCollectionTextChange]>),
}

pub(in crate::mounting::projection) fn lower_semantic_text_seed(
    input: Option<&crate::mounting::UiMountedSemanticTextContent>,
    predecessor: Option<&UiMountedSemanticTextSeed>,
    formatting: Option<UiMountedSemanticTextFormattingSeed>,
) -> Result<Option<UiMountedSemanticTextSeed>, UiMountedProjectionDenial> {
    let Some((content, posture, transition)) = resolved_content(input, predecessor)? else {
        return Ok(None);
    };
    let formatting = formatting.ok_or(UiMountedProjectionDenial::MissingSemanticTextColor)?;
    let layer_semantic_order = formatting
        .layer_semantic_order()
        .checked_add(1)
        .ok_or(UiMountedProjectionDenial::SemanticTextLayerOrderExceeded)?;
    let formatting_changed =
        predecessor.is_some_and(|predecessor| predecessor.formatting != formatting);
    let paint_only = formatting_changed
        && matches!(&transition, UiMountedSemanticTextSeedTransition::Retained)
        && predecessor.is_some_and(|predecessor| {
            predecessor.formatting.same_layout_as(&formatting)
                && predecessor.layer_semantic_order == layer_semantic_order
        });
    Ok(Some(UiMountedSemanticTextSeed {
        content,
        posture,
        formatting: formatting.clone(),
        layer_semantic_order,
        transition: if paint_only {
            UiMountedSemanticTextSeedTransition::PaintOnly
        } else if formatting_changed
            || predecessor
                .is_some_and(|predecessor| predecessor.layer_semantic_order != layer_semantic_order)
        {
            UiMountedSemanticTextSeedTransition::Complete
        } else {
            transition
        },
    }))
}

fn resolved_content(
    input: Option<&crate::mounting::UiMountedSemanticTextContent>,
    predecessor: Option<&UiMountedSemanticTextSeed>,
) -> Result<
    Option<(
        UiMountedSemanticTextSeedContent,
        Arc<str>,
        UiMountedSemanticTextSeedTransition,
    )>,
    UiMountedProjectionDenial,
> {
    let Some(input) = input else {
        return Ok(predecessor.map(|seed| {
            (
                seed.content.clone(),
                Arc::clone(&seed.posture),
                UiMountedSemanticTextSeedTransition::Retained,
            )
        }));
    };
    match input {
        crate::mounting::UiMountedSemanticTextContent::Scalar(input) => {
            resolve_scalar(input, predecessor).map(Some)
        }
        crate::mounting::UiMountedSemanticTextContent::Collection(input) => {
            resolve_collection(input, predecessor).map(Some)
        }
    }
}

fn resolve_scalar(
    input: &crate::mounting::UiMountedScalarSemanticTextContent,
    predecessor: Option<&UiMountedSemanticTextSeed>,
) -> Result<
    (
        UiMountedSemanticTextSeedContent,
        Arc<str>,
        UiMountedSemanticTextSeedTransition,
    ),
    UiMountedProjectionDenial,
> {
    let predecessor_value = predecessor
        .map(UiMountedSemanticTextSeed::scalar_value)
        .transpose()?
        .flatten();
    let value = match input.value() {
        crate::mounting::UiMountedSemanticTextValueDirective::Replace(value) => {
            Some(Arc::clone(value))
        }
        crate::mounting::UiMountedSemanticTextValueDirective::Preserve => predecessor_value.clone(),
        crate::mounting::UiMountedSemanticTextValueDirective::Clear => None,
    };
    let transition = if predecessor.is_some_and(|predecessor| {
        predecessor_value.as_deref() == value.as_deref()
            && predecessor.posture.as_ref() == input.posture().as_ref()
    }) {
        UiMountedSemanticTextSeedTransition::Retained
    } else {
        UiMountedSemanticTextSeedTransition::Complete
    };
    Ok((
        UiMountedSemanticTextSeedContent::Scalar(value),
        Arc::clone(input.posture()),
        transition,
    ))
}

fn resolve_collection(
    input: &crate::mounting::UiMountedCollectionSemanticTextContent,
    predecessor: Option<&UiMountedSemanticTextSeed>,
) -> Result<
    (
        UiMountedSemanticTextSeedContent,
        Arc<str>,
        UiMountedSemanticTextSeedTransition,
    ),
    UiMountedProjectionDenial,
> {
    let predecessor = predecessor
        .map(UiMountedSemanticTextSeed::collection_rows)
        .transpose()?;
    let (rows, transition) = match input.value() {
        crate::mounting::UiMountedCollectionTextDirective::Replace(rows) => (
            UiMountedCollectionTextSource::replace(rows)?,
            UiMountedSemanticTextSeedTransition::Complete,
        ),
        crate::mounting::UiMountedCollectionTextDirective::Patch(changes) => (
            predecessor
                .ok_or(UiMountedProjectionDenial::MissingSemanticCollectionPredecessor)?
                .apply(changes)?,
            UiMountedSemanticTextSeedTransition::CollectionPatch(changes.to_vec().into()),
        ),
        crate::mounting::UiMountedCollectionTextDirective::Preserve => (
            predecessor.cloned().unwrap_or_default(),
            UiMountedSemanticTextSeedTransition::Complete,
        ),
        crate::mounting::UiMountedCollectionTextDirective::Clear => (
            UiMountedCollectionTextSource::default(),
            UiMountedSemanticTextSeedTransition::Complete,
        ),
    };
    Ok((
        UiMountedSemanticTextSeedContent::Collection(rows),
        Arc::clone(input.posture()),
        transition,
    ))
}

impl UiMountedSemanticTextSeed {
    pub(in crate::mounting::projection) fn require_complete_mechanics(&mut self) {
        self.transition = UiMountedSemanticTextSeedTransition::Complete;
    }

    fn scalar_value(&self) -> Result<Option<Arc<str>>, UiMountedProjectionDenial> {
        match &self.content {
            UiMountedSemanticTextSeedContent::Scalar(value) => Ok(value.clone()),
            UiMountedSemanticTextSeedContent::Collection(_) => {
                Err(UiMountedProjectionDenial::SemanticTextShapeMismatch)
            }
        }
    }

    fn collection_rows(&self) -> Result<&UiMountedCollectionTextSource, UiMountedProjectionDenial> {
        match &self.content {
            UiMountedSemanticTextSeedContent::Collection(rows) => Ok(rows),
            UiMountedSemanticTextSeedContent::Scalar(_) => {
                Err(UiMountedProjectionDenial::SemanticTextShapeMismatch)
            }
        }
    }

    pub(in crate::mounting::projection) const fn content(
        &self,
    ) -> &UiMountedSemanticTextSeedContent {
        &self.content
    }

    pub(in crate::mounting::projection) fn posture(&self) -> &Arc<str> {
        &self.posture
    }

    pub(in crate::mounting::projection) const fn layer_semantic_order(&self) -> u32 {
        self.layer_semantic_order
    }

    pub(in crate::mounting::projection) const fn transition(
        &self,
    ) -> &UiMountedSemanticTextSeedTransition {
        &self.transition
    }

    pub(in crate::mounting::projection) const fn formatting(
        &self,
    ) -> &UiMountedSemanticTextFormattingSeed {
        &self.formatting
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn scalar_for_test() -> Self {
        Self {
            content: UiMountedSemanticTextSeedContent::Scalar(Some(Arc::from("value"))),
            posture: Arc::from("CURRENT"),
            formatting: UiMountedSemanticTextFormattingSeed::body_default_for_test(),
            layer_semantic_order: 1,
            transition: UiMountedSemanticTextSeedTransition::Complete,
        }
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn posture_only_for_test(posture: &'static str) -> Self {
        Self {
            content: UiMountedSemanticTextSeedContent::Scalar(None),
            posture: Arc::from(posture),
            formatting: UiMountedSemanticTextFormattingSeed::body_default_for_test(),
            layer_semantic_order: 1,
            transition: UiMountedSemanticTextSeedTransition::Complete,
        }
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn collection_for_test(
        rows: &[crate::mounting::UiMountedCollectionTextRow],
    ) -> Self {
        Self {
            content: UiMountedSemanticTextSeedContent::Collection(
                UiMountedCollectionTextSource::replace(rows).expect("test collection is valid"),
            ),
            posture: Arc::from("CURRENT"),
            formatting: UiMountedSemanticTextFormattingSeed::body_default_for_test(),
            layer_semantic_order: 1,
            transition: UiMountedSemanticTextSeedTransition::Complete,
        }
    }

    #[cfg(test)]
    pub(in crate::mounting::projection) fn collection_patch_for_test(
        predecessor: &Self,
        changes: &[crate::mounting::UiMountedCollectionTextChange],
    ) -> Self {
        let predecessor = predecessor
            .collection_rows()
            .expect("test predecessor is a collection");
        Self {
            content: UiMountedSemanticTextSeedContent::Collection(
                predecessor.apply(changes).expect("test patch is valid"),
            ),
            posture: Arc::from("CURRENT"),
            formatting: UiMountedSemanticTextFormattingSeed::body_default_for_test(),
            layer_semantic_order: 1,
            transition: UiMountedSemanticTextSeedTransition::CollectionPatch(
                changes.to_vec().into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_value_only_change_is_a_paint_only_transition() {
        let predecessor = UiMountedSemanticTextSeed::scalar_for_test();
        let formatting = UiMountedSemanticTextFormattingSeed::body_default_with_color_for_test(
            worth_ui_host_contract::UiMountedRgba8::new(247, 129, 47, 255),
        );
        let successor = lower_semantic_text_seed(None, Some(&predecessor), Some(formatting))
            .unwrap()
            .unwrap();
        assert!(matches!(
            successor.transition(),
            UiMountedSemanticTextSeedTransition::PaintOnly
        ));
    }

    #[test]
    fn repeated_scalar_replace_keeps_paint_only_formatting_local() {
        let predecessor = UiMountedSemanticTextSeed::scalar_for_test();
        let node = crate::graph::UiGraphNodeIdentity::new(991);
        let mut content = crate::mounting::UiMountedSemanticContentInput::empty();
        content
            .insert_scalar(
                node,
                crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::from("value")),
                Arc::from("CURRENT"),
            )
            .unwrap();
        let input = content.get(node).unwrap();
        assert!(matches!(
            input,
            crate::mounting::UiMountedSemanticTextContent::Scalar(_)
        ));
        let formatting = UiMountedSemanticTextFormattingSeed::body_default_with_color_for_test(
            worth_ui_host_contract::UiMountedRgba8::new(247, 129, 47, 255),
        );
        let successor = lower_semantic_text_seed(Some(input), Some(&predecessor), Some(formatting))
            .unwrap()
            .unwrap();
        assert!(matches!(
            successor.transition(),
            UiMountedSemanticTextSeedTransition::PaintOnly
        ));
    }

    #[test]
    fn layer_order_change_requires_complete_requalification() {
        let predecessor = UiMountedSemanticTextSeed::scalar_for_test();
        let formatting =
            UiMountedSemanticTextFormattingSeed::body_default_with_color_and_layer_for_test(
                worth_ui_host_contract::UiMountedRgba8::new(247, 129, 47, 255),
                1,
            );
        let successor = lower_semantic_text_seed(None, Some(&predecessor), Some(formatting))
            .unwrap()
            .unwrap();
        assert!(matches!(
            successor.transition(),
            UiMountedSemanticTextSeedTransition::Complete
        ));
    }
}
