use super::{
    canonical_identity::canonical_resource_contract_token, validation::validate_resource_contract,
    WorthQueryExecutionStrategyContract,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum WorthQueryExecutionResourceContract {
    #[default]
    Undeclared,
    Declared {
        strategies: Vec<WorthQueryExecutionStrategyContract>,
    },
}

impl WorthQueryExecutionResourceContract {
    pub fn declared(
        strategies: impl IntoIterator<Item = WorthQueryExecutionStrategyContract>,
    ) -> Result<Self, &'static str> {
        let mut strategies = strategies.into_iter().collect::<Vec<_>>();
        strategies.sort_by(|left, right| left.name().cmp(right.name()));
        let contract = Self::Declared { strategies };
        validate_resource_contract(&contract)?;
        Ok(contract)
    }

    pub fn strategies(&self) -> &[WorthQueryExecutionStrategyContract] {
        match self {
            Self::Undeclared => &[],
            Self::Declared { strategies } => strategies,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        validate_resource_contract(self)
    }

    pub fn canonical_identity(&self) -> String {
        canonical_resource_contract_token(self)
    }

    pub(crate) fn canonical_token(&self) -> String {
        self.canonical_identity()
    }
}
