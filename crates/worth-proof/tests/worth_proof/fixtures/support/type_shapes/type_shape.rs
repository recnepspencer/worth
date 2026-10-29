#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeShapeCheck {
    lane: &'static str,
    actual_bytes: usize,
    expected_bytes: usize,
}

impl TypeShapeCheck {
    pub fn new(lane: &'static str, actual_bytes: usize, expected_bytes: usize) -> Self {
        Self {
            lane,
            actual_bytes,
            expected_bytes,
        }
    }

    pub fn lane(&self) -> &'static str {
        self.lane
    }

    pub fn actual_bytes(&self) -> usize {
        self.actual_bytes
    }

    pub fn expected_bytes(&self) -> usize {
        self.expected_bytes
    }

    pub fn matches(&self) -> bool {
        self.actual_bytes == self.expected_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeShapeReport {
    suite: &'static str,
    checks: Vec<TypeShapeCheck>,
}

impl TypeShapeReport {
    pub fn new(suite: &'static str, checks: Vec<TypeShapeCheck>) -> Self {
        Self { suite, checks }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn checks(&self) -> &[TypeShapeCheck] {
        &self.checks
    }
}
