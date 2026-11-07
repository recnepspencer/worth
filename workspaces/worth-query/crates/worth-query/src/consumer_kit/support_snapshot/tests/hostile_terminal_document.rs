use crate::consumer_kit::{
    project_support_snapshot, WorthQueryExternalSupportSnapshotTerminalJsonDocument,
    WorthQuerySupportSnapshot,
};

use super::live_support_matrix;

pub(super) struct HostileSupportSnapshotTerminalDocument {
    text: String,
}

impl HostileSupportSnapshotTerminalDocument {
    pub(super) fn from_live_support_matrix() -> Self {
        let matrix = live_support_matrix();
        let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");
        Self::from_snapshot(&snapshot)
    }

    pub(super) fn from_snapshot(snapshot: &WorthQuerySupportSnapshot) -> Self {
        Self {
            text: snapshot
                .to_canonical_terminal_json_document()
                .expect("support snapshot terminal document should serialize")
                .as_str()
                .to_string(),
        }
    }

    pub(super) fn replace_top_level_string(
        &mut self,
        field: &'static str,
        value: impl Into<String>,
    ) {
        replace_string_field(&mut self.text, field, value, 0);
    }

    pub(super) fn replace_top_level_number(&mut self, field: &'static str, value: u16) {
        replace_number_field(&mut self.text, field, value, 0);
    }

    pub(super) fn replace_first_row_string(
        &mut self,
        field: &'static str,
        value: impl Into<String>,
    ) {
        replace_string_field(&mut self.text, field, value, 0);
    }

    pub(super) fn into_external_terminal_json_document(
        self,
    ) -> WorthQueryExternalSupportSnapshotTerminalJsonDocument {
        WorthQueryExternalSupportSnapshotTerminalJsonDocument::from_external_terminal_json_document(
            self.text,
        )
    }
}

fn replace_string_field(
    document: &mut String,
    field: &'static str,
    value: impl Into<String>,
    occurrence: usize,
) {
    let field_marker = format!("\"{field}\": \"");
    let start = nth_match(document, &field_marker, occurrence) + field_marker.len();
    let end = document[start..]
        .find('"')
        .map(|offset| start + offset)
        .expect("hostile terminal string field should have a closing quote");
    document.replace_range(start..end, &value.into());
}

fn replace_number_field(document: &mut String, field: &'static str, value: u16, occurrence: usize) {
    let field_marker = format!("\"{field}\": ");
    let start = nth_match(document, &field_marker, occurrence) + field_marker.len();
    let end = document[start..]
        .find([',', '\n'])
        .map(|offset| start + offset)
        .expect("hostile terminal number field should have a delimiter");
    document.replace_range(start..end, &value.to_string());
}

fn nth_match(document: &str, pattern: &str, occurrence: usize) -> usize {
    document
        .match_indices(pattern)
        .nth(occurrence)
        .map(|(index, _)| index)
        .expect("hostile terminal mutation field should exist")
}
