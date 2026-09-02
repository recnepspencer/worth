use super::super::{sealing, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    for (role, provenance) in state.appearance_roles.values() {
        if role.partitions().iter().any(|(aspect, partition)| {
            partition
                .cells()
                .iter()
                .any(|cell| cell.result().value_kind() != aspect.value_kind())
        }) {
            state.diagnostics.push(sealing::appearance_diagnostic(
                crate::WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind,
                "appearance role contains a wrong-kind decision result",
                provenance,
            ));
        }
    }
}
