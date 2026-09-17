fn mutate_query_from_ui_admission<I: worth_ui::facade::intent::UiIntent>(
    owner: &worth_ui::facade::query_binding::WorthUiStatusSourceOwner,
    admission: worth_ui::facade::intent::UiAdmittedIntent<I>,
) {
    let _ = owner.execute_action(admission);
}
