//! One ordered path record grammar, independent of its consumer frame.
use super::ApplicationQueryRootPathMeaning;

impl ApplicationQueryRootPathMeaning {
    pub(crate) fn visit_canonical_parts(&self, mut part: impl FnMut(&str)) {
        drive_root_path_parts(self, &mut part);
    }
}

#[inline(never)]
fn drive_root_path_parts(path: &ApplicationQueryRootPathMeaning, part: &mut dyn FnMut(&str)) {
    part(path.start_entity());
    part(path.terminal_entity());
    part(&path.steps().len().to_string());
    for step in path.steps() {
        part(step.relation());
        part(step.from());
        part(step.to());
        part(match (step.traversal(), step.direction()) {
            (
                crate::application_query::ApplicationQueryRootPathTraversal::OneHop,
                crate::application_query::RelationDirection::Forward,
            ) => "forward",
            (
                crate::application_query::ApplicationQueryRootPathTraversal::OneHop,
                crate::application_query::RelationDirection::Reverse,
            ) => "reverse",
            (
                crate::application_query::ApplicationQueryRootPathTraversal::Closure,
                crate::application_query::RelationDirection::Forward,
            ) => "forward-closure",
            (
                crate::application_query::ApplicationQueryRootPathTraversal::Closure,
                crate::application_query::RelationDirection::Reverse,
            ) => "reverse-closure",
        });
    }
    part(&path.guards().len().to_string());
    for guard in path.guards() {
        part(&guard.after_step().to_string());
        part(guard.entity());
        part(guard.aspect());
        part(guard.field());
        part(guard.scalar_family().canonical_name());
        part(guard.value_type());
        part(
            worth_foundational::facade::prepare_aspect_value_identity_basis(guard.expected())
                .as_str(),
        );
    }
}
