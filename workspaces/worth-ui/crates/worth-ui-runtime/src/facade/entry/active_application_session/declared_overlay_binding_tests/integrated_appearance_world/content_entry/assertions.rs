use super::TextWorld;
use crate::mounting::*;
use worth_ui_host_contract::*;

impl TextWorld {
    pub fn assert_text(&self, index: usize, expected: &str) {
        let (basis, accepted_attempt, projection) = self.accepted[index]
            .as_ref()
            .expect("surface accepted a presentation");
        let (attempt, commands) = self
            .host
            .accepted_text_commands(self.surfaces[index])
            .expect("host accepted text work on this surface");
        assert_eq!(
            attempt, *accepted_attempt,
            "host text belongs to the exact accepted attempt"
        );
        let values: Vec<_> = commands
            .iter()
            .filter_map(|command| match command {
                UiMountedPaintCommand::SemanticText { mechanic, .. }
                    if mechanic.slot() == UiSemanticTextSlot::Value =>
                {
                    Some(mechanic)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            values.len(),
            self.occurrences
                .iter()
                .filter(|(_, surface, _)| *surface == index)
                .count()
        );
        for (instance, _, _) in self
            .occurrences
            .iter()
            .filter(|(_, surface, _)| *surface == index)
        {
            assert_eq!(
                values
                    .iter()
                    .find(|value| value.mounted_instance() == *instance)
                    .expect("host consumed this occurrence's text")
                    .text(),
                expected
            );
        }
        let view = projection.view_for(basis.binding()).unwrap();
        assert_eq!(
            view.semantic_text()
                .rows()
                .iter()
                .filter(|row| row.slot() == UiSemanticTextSlot::Value)
                .count(),
            self.occurrences
                .iter()
                .filter(|(_, surface, _)| *surface == index)
                .count(),
            "accepted text contains exactly the selected surface's live occurrences"
        );
        for (instance, _, bounds) in self
            .occurrences
            .iter()
            .filter(|(_, member, _)| *member == index)
        {
            let row = view
                .semantic_text()
                .rows()
                .iter()
                .find(|row| {
                    row.mounted_instance() == *instance && row.slot() == UiSemanticTextSlot::Value
                })
                .expect("each accepted occurrence has its actual text");
            assert_eq!(row.text(), expected);
            let mechanic = view
                .authored_paint_commands()
                .iter()
                .find_map(|command| match command {
                    UiMountedPaintCommand::SemanticText { mechanic, .. }
                        if mechanic.mounted_instance() == *instance
                            && mechanic.slot() == UiSemanticTextSlot::Value =>
                    {
                        Some(mechanic)
                    }
                    _ => None,
                })
                .expect("accepted presentation contains the text paint command");
            assert_eq!(mechanic.text(), expected);
            let actual = mechanic.bounds();
            assert_eq!(
                [actual.x(), actual.y(), actual.width(), actual.height()],
                *bounds
            );
        }
    }

    pub fn assert_pending(&self, expected: Option<(u64, &str)>) {
        let projection = self.session.presentation.project().unwrap();
        let expected_revisions: Vec<_> = expected
            .map(|(revision, _)| {
                (
                    format!("component:{}", super::COMPONENT).into_boxed_str(),
                    self.graph,
                    revision,
                )
            })
            .into_iter()
            .collect();
        assert_eq!(projection.text_revisions_for_test(), expected_revisions);
        let content = projection.content();
        let actual = content.get(self.graph).map(|row| match row {
            UiMountedSemanticTextContent::Scalar(row) => match row.value() {
                UiMountedSemanticTextValueDirective::Replace(text) => text.as_ref(),
                _ => panic!("pending product content must carry its exact value"),
            },
            _ => panic!("product label is scalar text"),
        });
        assert_eq!(actual, expected.map(|(_, text)| text));
    }
}
