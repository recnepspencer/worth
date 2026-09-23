#[derive(Clone, Debug)]
pub struct RequiredWorkflowEvidence {
    instance: worth_relational::facade::identity::EntityId,
    node_path: String,
    required_assessments: usize,
    completed_assessments: usize,
    passing_assessments: usize,
}

impl RequiredWorkflowEvidence {
    pub(in crate::domain_computation::primary_graph) fn new(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        required_assessments: usize,
        completed_assessments: usize,
        passing_assessments: usize,
    ) -> Self {
        Self {
            instance,
            node_path,
            required_assessments,
            completed_assessments,
            passing_assessments,
        }
    }

    pub const fn instance(&self) -> worth_relational::facade::identity::EntityId {
        self.instance
    }
    pub fn node_path(&self) -> &str {
        &self.node_path
    }
    pub const fn required_assessments(&self) -> usize {
        self.required_assessments
    }
    pub const fn completed_assessments(&self) -> usize {
        self.completed_assessments
    }
    pub const fn passing_assessments(&self) -> usize {
        self.passing_assessments
    }
}
