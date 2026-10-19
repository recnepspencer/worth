//! Definition-level action dependencies are not synthetic graph nodes.

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryWorkflowGlobalActionDependency {
    Terminate {
        identifier: String,
        input_type: String,
        binding: String,
        requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
    },
}

impl WorthQueryWorkflowGlobalActionDependency {
    pub fn binding(&self) -> &str {
        match self {
            Self::Terminate { binding, .. } => binding,
        }
    }

    pub fn program_names(&self) -> impl Iterator<Item = &str> {
        match self {
            Self::Terminate {
                identifier,
                binding,
                ..
            } => [identifier.as_str(), binding.as_str()].into_iter(),
        }
    }

    pub fn retained_string_bytes(&self) -> usize {
        match self {
            Self::Terminate {
                identifier,
                input_type,
                binding,
                ..
            } => identifier
                .len()
                .saturating_add(input_type.len())
                .saturating_add(binding.len()),
        }
    }
}
