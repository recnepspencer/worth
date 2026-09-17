#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiStatusActionIdentity {
    session: u64,
    lineage: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiStatusActionRequest {
    source_revision: u64,
    status: String,
    identity: WorthUiStatusActionIdentity,
}

impl WorthUiStatusActionIdentity {
    pub const fn new(session: u64, lineage: u64) -> Self {
        Self { session, lineage }
    }

    pub const fn session(self) -> u64 {
        self.session
    }

    pub const fn lineage(self) -> u64 {
        self.lineage
    }
}

impl WorthUiStatusActionRequest {
    pub fn new(
        source_revision: u64,
        status: impl Into<String>,
        session: u64,
        lineage: u64,
    ) -> Result<Self, &'static str> {
        let status = status.into();
        if status.is_empty() || status.len() > 65_536 {
            return Err("action status must be 1 to 65,536 bytes");
        }
        Ok(Self {
            source_revision,
            status,
            identity: WorthUiStatusActionIdentity::new(session, lineage),
        })
    }

    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub const fn identity(&self) -> WorthUiStatusActionIdentity {
        self.identity
    }
}
