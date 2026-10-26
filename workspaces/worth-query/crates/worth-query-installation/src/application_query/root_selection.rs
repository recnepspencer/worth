use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, ScalarAspectType};
use worth_query_declaration::facade::application_query::RelationDirection;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledRootPathGuard {
    after_step: usize,
    entity: String,
    aspect: AspectKey,
    field: FieldKey,
    scalar_family: ScalarAspectType,
    value_type: String,
    expected: AspectValue,
}

impl WorthQueryInstalledRootPathGuard {
    pub(crate) fn new(
        guard: &worth_query_declaration::facade::application_query::ApplicationQueryRootPathGuard,
    ) -> Self {
        Self {
            after_step: guard.after_step(),
            entity: guard.entity().to_string(),
            aspect: AspectKey::new(guard.aspect())
                .expect("declared root-path guard aspect is schema validated"),
            field: FieldKey::new(guard.field())
                .expect("declared root-path guard field is schema validated"),
            scalar_family: guard.scalar_family(),
            value_type: guard.value_type().to_string(),
            expected: guard.expected().clone(),
        }
    }

    pub const fn after_step(&self) -> usize {
        self.after_step
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field(&self) -> &FieldKey {
        &self.field
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub fn value_type(&self) -> &str {
        &self.value_type
    }

    pub const fn expected(&self) -> &AspectValue {
        &self.expected
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledRootPathStep {
    relation: String,
    from: String,
    to: String,
    direction: RelationDirection,
    depth: usize,
}

impl WorthQueryInstalledRootPathStep {
    pub(crate) fn new(
        relation: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        direction: RelationDirection,
        depth: usize,
    ) -> Self {
        Self {
            relation: relation.into(),
            from: from.into(),
            to: to.into(),
            direction,
            depth,
        }
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> RelationDirection {
        self.direction
    }

    pub const fn depth(&self) -> usize {
        self.depth
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledRootPath {
    steps: Vec<WorthQueryInstalledRootPathStep>,
    guards: Vec<WorthQueryInstalledRootPathGuard>,
}

impl WorthQueryInstalledRootPath {
    pub(crate) fn new(
        steps: Vec<WorthQueryInstalledRootPathStep>,
        guards: Vec<WorthQueryInstalledRootPathGuard>,
    ) -> Self {
        Self { steps, guards }
    }

    pub fn steps(&self) -> &[WorthQueryInstalledRootPathStep] {
        &self.steps
    }

    pub fn guards(&self) -> &[WorthQueryInstalledRootPathGuard] {
        &self.guards
    }
}
