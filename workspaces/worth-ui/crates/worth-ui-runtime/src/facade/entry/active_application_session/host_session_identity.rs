use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub fn host_session_identity(&self) -> crate::facade::WorthUiHostSessionIdentity {
        self.host_session.identity()
    }
}
