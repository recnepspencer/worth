use crate::schema::data::{
    HistoricalInterpretationSensitivity, SchemaDiffAtom, SubscriberBoundaryVisibility,
};

pub fn strongest_boundary_visibility(
    diff_atoms: &[SchemaDiffAtom],
) -> SubscriberBoundaryVisibility {
    diff_atoms
        .iter()
        .map(|atom| atom.boundary_visibility)
        .max()
        .unwrap_or(SubscriberBoundaryVisibility::NotVisible)
}

pub fn strongest_historical_interpretation(
    current: HistoricalInterpretationSensitivity,
    candidate: HistoricalInterpretationSensitivity,
) -> HistoricalInterpretationSensitivity {
    if candidate.sensitivity_rank() > current.sensitivity_rank() {
        candidate
    } else {
        current
    }
}
