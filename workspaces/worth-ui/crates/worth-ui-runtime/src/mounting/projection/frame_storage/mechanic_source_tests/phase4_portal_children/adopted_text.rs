use super::*;
use crate::mounting::projection::semantic_text::{
    lower_semantic_text_formatting, lower_semantic_text_seed,
};
use std::collections::BTreeMap;

pub(super) fn adopt_value(world: &mut GeometryWorld, child: UiMountedInstanceIdentity) {
    let mut node = world.semantic.node(child).unwrap().clone();
    let graph_node = node.receipt().graph_node();
    let token = ThemeTokenId::new("theme.test.foreground").unwrap();
    let constraints = worth_ui_text::UiTextParagraphConstraints::new(
        worth_ui_text::UiTextParagraphConstraintsInput {
            language: Arc::from("und"),
            base_direction: worth_ui_text::UiTextBaseDirection::Auto,
            wrap: worth_ui_text::UiTextWrap::UnicodeWord,
            alignment: worth_ui_text::UiTextAlignment::Start,
            overflow: worth_ui_text::UiTextOverflow::Clip,
            font_size_millipoints: 14_000,
            width_millipoints: 220_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 1,
        },
    )
    .unwrap();
    let span = ComponentSemanticTextSpanContract::new(
        worth_ui_host_contract::UiTextOriginalRange::new(0, 5).unwrap(),
        token.clone(),
        worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
    )
    .unwrap()
    .with_appearance_foreground();
    let contract = ComponentSemanticTextContract::spanned(token.clone(), 1, [span]).unwrap();
    let formatting = crate::mounting::UiMountedSemanticTextFormattingDirective::new(
        contract,
        BTreeMap::from([(
            token,
            crate::runtime::tests::appearance_component_session_test_support::appearance_theme_value(
                "#112233",
            ),
        )]),
    );
    let mut content = crate::mounting::UiMountedSemanticContentInput::empty();
    content
        .insert_scalar_with_formatting(
            graph_node,
            crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::from("value")),
            Arc::from("CURRENT"),
            Some(formatting),
        )
        .unwrap();
    let input = content.get(graph_node);
    let formatting = lower_semantic_text_formatting(
        crate::mounting::UiMountedPlanProjectionSource::PreviewOnly,
        &crate::mounting::UiMountedThemeValueSource::from_admitted(Default::default()),
        graph_node,
        None,
        input,
        None,
        false,
    )
    .unwrap();
    node.semantic_text = lower_semantic_text_seed(input, None, formatting).unwrap();
    world.semantic.insert_node(node);
}
