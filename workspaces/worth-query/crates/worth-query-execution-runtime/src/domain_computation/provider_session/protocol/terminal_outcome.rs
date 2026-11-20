use super::WorthQueryProviderSessionProtocolCounters;

/// Provider-authored text describing a completed physical transition.
///
/// This value is deliberately descriptive only. It is never parsed and cannot
/// register, locate, commit, abort, clean up, or readmit a provider session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderTerminalDescription(std::sync::Arc<str>);

impl WorthQueryProviderTerminalDescription {
    pub fn new(description: impl Into<std::sync::Arc<str>>) -> Result<Self, &'static str> {
        let description = description.into();
        if description.trim().is_empty() || description.trim() != description.as_ref() {
            return Err("provider terminal description must be non-empty and trimmed");
        }
        Ok(Self(description))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct WorthQueryClosedProviderSessionDisposition {
    provider_description: WorthQueryProviderTerminalDescription,
    counters: WorthQueryProviderSessionProtocolCounters,
    terminal_binding: super::WorthQueryProviderSessionTerminalBinding,
}

impl WorthQueryClosedProviderSessionDisposition {
    pub fn close(
        provider_description: WorthQueryProviderTerminalDescription,
        counters: WorthQueryProviderSessionProtocolCounters,
        terminal_binding: super::WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        Self {
            provider_description,
            counters,
            terminal_binding,
        }
    }

    pub fn provider_description(&self) -> &WorthQueryProviderTerminalDescription {
        &self.provider_description
    }

    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub const fn terminal_binding(
        &self,
    ) -> &super::WorthQueryProviderSessionTerminalBinding {
        &self.terminal_binding
    }
}

/// A provider closes the executed session with either a physical publication
/// description or an owner-issued, contact-bearing output adoption.
#[derive(Debug)]
pub enum WorthQueryProviderCommitCompletion {
    Committed(WorthQueryProviderTerminalDescription),
    OutputAdoptionPrepared(crate::ports::PreparedOutputAdoptionPayload),
}
