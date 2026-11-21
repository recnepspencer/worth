/// Whether governed disclosure withheld any value from a query result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryOmissionPosture {
    /// Nothing was withheld.
    NoOmission,
    /// At least one value was withheld; see the receipt's disclosure record.
    GovernedOmission,
}
