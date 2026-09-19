//! Typed upstream operation compiler for the eight locality axes.

use std::sync::Arc;

use worth_ui::facade::declaration::{
    ComponentSemanticTextSpanContract, ThemeTokenId, ThemeTokenValue, UiThemeColor,
};
use worth_ui_host_contract::UiTextOriginalRange;
use worth_ui_native_platform::{
    UiNativeApplicationFrame, UiNativeApplicationProgram, UiNativeComponentPresenceChange,
    UiNativeComponentSemanticTextChange, UiNativeQualificationPlan,
};
use worth_ui_text::{
    UiTextAlignment, UiTextBaseDirection, UiTextOverflow, UiTextParagraphConstraints,
    UiTextParagraphConstraintsInput, UiTextStyle, UiTextWrap,
};

pub(super) const BASE_TOKEN: &str = "theme.phase5.matrix.base";
pub(super) const TARGET_TOKEN: &str = "theme.phase5.matrix.target";
pub(super) const ROOT: &str = "phase5.matrix.root";
pub(super) const SURFACE: &str = "phase5.matrix.surface";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Phase5LocalityAxis {
    Content,
    Width,
    PaintBoundary,
    Dpi,
    AtlasMiss,
    UploadCompletion,
    PinRelease,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Phase5LocalityCase {
    retained_size: usize,
    axis: Phase5LocalityAxis,
}

impl Phase5LocalityAxis {
    pub(super) const ALL: [Self; 7] = [
        Self::Content,
        Self::Width,
        Self::PaintBoundary,
        Self::Dpi,
        Self::AtlasMiss,
        Self::UploadCompletion,
        Self::PinRelease,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Width => "width",
            Self::PaintBoundary => "paint-boundary",
            Self::Dpi => "dpi",
            Self::AtlasMiss => "atlas-miss",
            Self::UploadCompletion => "upload-completion",
            Self::PinRelease => "pin-release",
        }
    }
}

impl Phase5LocalityCase {
    pub(super) fn new(retained_size: usize, axis: Phase5LocalityAxis) -> Self {
        assert!([1, 32, 2_048, 4_096].contains(&retained_size));
        Self {
            retained_size,
            axis,
        }
    }

    pub(super) const fn retained_size(self) -> usize {
        self.retained_size
    }

    pub(super) const fn retained_mechanics(self) -> usize {
        if self.retained_size == 1 {
            2
        } else {
            self.retained_size
        }
    }

    pub(super) const fn retained_paragraphs(self) -> usize {
        if self.retained_size == 1 {
            1
        } else {
            self.retained_size / 2
        }
    }

    pub(super) const fn axis(self) -> Phase5LocalityAxis {
        self.axis
    }

    pub(super) fn target_index(self) -> usize {
        match self.axis {
            Phase5LocalityAxis::Content => 0,
            Phase5LocalityAxis::AtlasMiss => self.retained_paragraphs() / 2,
            _ => self.retained_paragraphs().saturating_sub(1),
        }
    }

    pub(super) fn component_identity(self, index: usize) -> String {
        format!("phase5.matrix.text.n{index}")
    }

    pub(super) fn authored_identity(self, index: usize) -> String {
        format!("component:{}", self.component_identity(index))
    }

    pub(super) fn program(self) -> UiNativeApplicationProgram {
        UiNativeApplicationProgram::new([
            self.initial_frame(),
            self.successor_frame(),
            UiNativeApplicationFrame::present_current(),
        ])
        .expect("matrix program is within declared frame and change limits")
    }

    pub(super) fn qualification(self) -> UiNativeQualificationPlan {
        let qualification = match self.axis {
            Phase5LocalityAxis::Width => UiNativeQualificationPlan::ordinary()
                .with_client_width_delta_after_presentation(1, 24)
                .expect("bounded width successor"),
            Phase5LocalityAxis::Dpi => UiNativeQualificationPlan::ordinary()
                .with_dpi_scale_multiplier_after_presentation(1, 1_250)
                .expect("bounded DPI successor"),
            _ => UiNativeQualificationPlan::ordinary(),
        };
        qualification.with_certification_worker_event_loop()
    }

    fn initial_frame(self) -> UiNativeApplicationFrame {
        let changes = (0..self.retained_paragraphs()).map(|index| {
            let text = match (self.axis, index == self.target_index()) {
                (Phase5LocalityAxis::Content, true) => "A A",
                (Phase5LocalityAxis::PinRelease, true) => "AB",
                _ => "AAAA",
            };
            let change =
                UiNativeComponentSemanticTextChange::new(self.authored_identity(index), text)
                    .expect("matrix semantic text identity");
            if self.axis == Phase5LocalityAxis::PaintBoundary && index == self.target_index() {
                change
                    .with_spans(spans(2))
                    .expect("initial paint boundary is contiguous")
            } else {
                change
            }
        });
        UiNativeApplicationFrame::with_semantic_text(changes)
            .expect("retained-size initial frame is bounded")
    }

    fn successor_frame(self) -> UiNativeApplicationFrame {
        match self.axis {
            Phase5LocalityAxis::Content => self.one_text_successor("A\u{00a0}A"),
            Phase5LocalityAxis::Width | Phase5LocalityAxis::Dpi => {
                UiNativeApplicationFrame::present_current().after_host_surface_basis_successor()
            }
            Phase5LocalityAxis::PaintBoundary => {
                let change = UiNativeComponentSemanticTextChange::new(
                    self.authored_identity(self.target_index()),
                    "AAAA",
                )
                .expect("matrix paint boundary identity")
                .with_spans(spans(1))
                .expect("successor paint boundary is contiguous");
                UiNativeApplicationFrame::with_semantic_text([change])
                    .expect("one paint boundary change is bounded")
            }
            Phase5LocalityAxis::AtlasMiss => self.one_text_successor("AAAB"),
            Phase5LocalityAxis::UploadCompletion => {
                let changes = (0..self.retained_paragraphs()).map(|index| {
                    UiNativeComponentSemanticTextChange::new(self.authored_identity(index), "AAAB")
                        .expect("matrix upload fanout identity")
                });
                UiNativeApplicationFrame::with_semantic_text(changes)
                    .expect("upload fanout frame is bounded")
            }
            Phase5LocalityAxis::PinRelease => UiNativeApplicationFrame::with_component_presence([
                UiNativeComponentPresenceChange::new(
                    self.authored_identity(self.target_index()),
                    false,
                )
                .expect("matrix pin release identity"),
            ])
            .expect("one layout removal is bounded"),
        }
    }

    fn one_text_successor(self, text: &str) -> UiNativeApplicationFrame {
        UiNativeApplicationFrame::with_semantic_text([UiNativeComponentSemanticTextChange::new(
            self.authored_identity(self.target_index()),
            text,
        )
        .expect("matrix local text successor")])
        .expect("one semantic text change is bounded")
    }
}

pub(super) fn token(identity: &str) -> ThemeTokenId {
    ThemeTokenId::new(identity).expect("matrix token identity")
}

pub(super) fn color(value: &str) -> ThemeTokenValue {
    ThemeTokenValue::color(UiThemeColor::parse(value).expect("matrix token color"))
}

fn spans(boundary: u32) -> [ComponentSemanticTextSpanContract; 2] {
    [
        span(0, boundary, TARGET_TOKEN),
        span(boundary, 4, BASE_TOKEN),
    ]
}

fn span(start: u32, end: u32, token_identity: &str) -> ComponentSemanticTextSpanContract {
    ComponentSemanticTextSpanContract::new(
        UiTextOriginalRange::new(start, end).expect("matrix span range"),
        token(token_identity),
        matrix_style(),
    )
    .expect("matrix span is nonempty")
}

fn matrix_style() -> UiTextStyle {
    let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Clip,
        font_size_millipoints: 14_000,
        width_millipoints: 160_000,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: 1,
    })
    .expect("matrix text constraints");
    UiTextStyle::from_paragraph_constraints(&constraints)
}
