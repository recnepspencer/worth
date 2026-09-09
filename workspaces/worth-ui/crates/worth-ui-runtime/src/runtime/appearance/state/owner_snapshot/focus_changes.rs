impl super::UiAppearanceOwnerSnapshot {
    pub(crate) fn focus_changed_instances(
        &self,
        predecessor: &Self,
    ) -> Box<[worth_ui_host_contract::UiMountedInstanceIdentity]> {
        let previous = predecessor.focus().copied();
        let current = self.focus().copied();
        let unchanged = match (previous, current) {
            (Some(previous), Some(current)) => previous.appearance_dependency_eq(current),
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return Box::default();
        }
        let mut instances = previous
            .into_iter()
            .chain(current)
            .filter_map(|posture| posture.target())
            .map(|target| target.mounted_instance())
            .collect::<Vec<_>>();
        instances.sort_unstable();
        instances.dedup();
        instances.into()
    }
}
