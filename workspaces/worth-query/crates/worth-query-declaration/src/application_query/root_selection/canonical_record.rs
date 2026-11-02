//! One ordered path record grammar, independent of its consumer frame.
use super::ApplicationQueryRootPathMeaning;

impl ApplicationQueryRootPathMeaning {
    pub(crate) fn visit_canonical_parts(&self, mut part: impl FnMut(&str)) {
        part(self.start_entity());
        part(self.terminal_entity());
        part(&self.steps().len().to_string());
        for step in self.steps() {
            part(step.relation());
            part(step.from());
            part(step.to());
            part(
                match (step.source_kind(), step.traversal(), step.direction()) {
                    (super::ApplicationQueryRootPathSourceKind::Derived, traversal, direction) => {
                        match (traversal, direction) {
                            (
                                super::ApplicationQueryRootPathTraversal::OneHop,
                                super::RelationDirection::Forward,
                            ) => "forward-derived",
                            (
                                super::ApplicationQueryRootPathTraversal::OneHop,
                                super::RelationDirection::Reverse,
                            ) => "reverse-derived",
                            (
                                super::ApplicationQueryRootPathTraversal::Closure,
                                super::RelationDirection::Forward,
                            ) => "forward-derived-closure",
                            (
                                super::ApplicationQueryRootPathTraversal::Closure,
                                super::RelationDirection::Reverse,
                            ) => "reverse-derived-closure",
                        }
                    }
                    (
                        super::ApplicationQueryRootPathSourceKind::Stored,
                        crate::application_query::ApplicationQueryRootPathTraversal::OneHop,
                        crate::application_query::RelationDirection::Forward,
                    ) => "forward",
                    (
                        super::ApplicationQueryRootPathSourceKind::Stored,
                        crate::application_query::ApplicationQueryRootPathTraversal::OneHop,
                        crate::application_query::RelationDirection::Reverse,
                    ) => "reverse",
                    (
                        super::ApplicationQueryRootPathSourceKind::Stored,
                        crate::application_query::ApplicationQueryRootPathTraversal::Closure,
                        crate::application_query::RelationDirection::Forward,
                    ) => "forward-closure",
                    (
                        super::ApplicationQueryRootPathSourceKind::Stored,
                        crate::application_query::ApplicationQueryRootPathTraversal::Closure,
                        crate::application_query::RelationDirection::Reverse,
                    ) => "reverse-closure",
                },
            );
        }
        part(&self.guards().len().to_string());
        for guard in self.guards() {
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
}
