use crate::ProtocolFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCounterexampleState {
    ordinal: u64,
    action: String,
    valuations: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCounterexample {
    protocol: ProtocolFamily,
    states: Vec<ProtocolCounterexampleState>,
}

impl ProtocolCounterexample {
    pub fn diagnostic(protocol: ProtocolFamily, state_edges: Vec<String>) -> Self {
        Self {
            protocol,
            states: state_edges
                .into_iter()
                .enumerate()
                .map(|(index, action)| ProtocolCounterexampleState {
                    ordinal: index as u64 + 1,
                    action,
                    valuations: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn from_tlc_states(
        protocol: ProtocolFamily,
        states: Vec<ProtocolCounterexampleState>,
    ) -> Self {
        Self { protocol, states }
    }

    pub const fn protocol(&self) -> ProtocolFamily {
        self.protocol
    }

    pub fn state_edges(&self) -> impl Iterator<Item = &str> {
        self.states.iter().map(ProtocolCounterexampleState::action)
    }

    pub fn states(&self) -> &[ProtocolCounterexampleState] {
        &self.states
    }
}

impl ProtocolCounterexampleState {
    pub fn observed(
        ordinal: u64,
        action: impl Into<String>,
        valuations: Vec<(String, String)>,
    ) -> Self {
        Self {
            ordinal,
            action: action.into(),
            valuations,
        }
    }

    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn valuations(&self) -> impl Iterator<Item = (&str, &str)> {
        self.valuations
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    pub fn valuation(&self, variable: &str) -> Option<&str> {
        self.valuations
            .iter()
            .find_map(|(name, value)| (name == variable).then_some(value.as_str()))
    }
}
