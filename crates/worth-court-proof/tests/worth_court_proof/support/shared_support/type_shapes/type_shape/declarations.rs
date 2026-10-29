#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeShapeCheck {
    pub(crate) lane: &'static str,
    pub(crate) actual_bytes: usize,
    pub(crate) expected_bytes: usize,
}

impl TypeShapeCheck {
    pub(crate) fn new(lane: &'static str, actual_bytes: usize, expected_bytes: usize) -> Self {
        Self {
            lane,
            actual_bytes,
            expected_bytes,
        }
    }

    pub(crate) fn lane(&self) -> &'static str {
        self.lane
    }

    pub(crate) fn actual_bytes(&self) -> usize {
        self.actual_bytes
    }

    pub(crate) fn expected_bytes(&self) -> usize {
        self.expected_bytes
    }

    pub(crate) fn matches(&self) -> bool {
        self.actual_bytes == self.expected_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeShapeReport {
    pub(crate) suite: &'static str,
    pub(crate) checks: Vec<TypeShapeCheck>,
}

impl TypeShapeReport {
    pub(crate) fn new(suite: &'static str, checks: Vec<TypeShapeCheck>) -> Self {
        Self { suite, checks }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn checks(&self) -> &[TypeShapeCheck] {
        &self.checks
    }
}