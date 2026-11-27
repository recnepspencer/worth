//! Installed role order used by the naming unit courts.
use super::*;

pub trait InstalledPersistentNameSchemeOwnerPort: Sized {
    fn name_scheme_from_ordered_roles_for_test(
        scheme_id: impl Into<String>,
        subject_kind: PersistentNameSubjectKind,
        ordered_roles: impl IntoIterator<Item = impl Into<String>>,
        split_capable: bool,
        require_authored_occurrence: bool,
    ) -> Self;
}

impl InstalledPersistentNameSchemeOwnerPort for InstalledPersistentNameScheme {
    fn name_scheme_from_ordered_roles_for_test(
        scheme_id: impl Into<String>,
        subject_kind: PersistentNameSubjectKind,
        ordered_roles: impl IntoIterator<Item = impl Into<String>>,
        split_capable: bool,
        require_authored_occurrence: bool,
    ) -> Self {
        let scheme_id = scheme_id.into();
        let mut role_ranks = BTreeMap::new();
        for (rank, role) in ordered_roles.into_iter().enumerate() {
            role_ranks.insert(
                role.into(),
                u32::try_from(rank).expect("role count fits u32"),
            );
        }
        Self {
            subject_name: scheme_id.clone(),
            scheme_id,
            subject_kind,
            role_ranks,
            split_capable,
            require_authored_occurrence,
            structural_identity: false,
        }
    }
}
