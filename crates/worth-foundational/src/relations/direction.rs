/// Direction relative to a stored relation's source and target.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum RelationDirection {
    /// Follow source to target.
    Forward,
    /// Follow target to source.
    Reverse,
}

#[cfg(test)]
mod tests {
    use super::RelationDirection;

    #[test]
    fn shared_direction_preserves_existing_path_spelling_and_order() {
        for (direction, encoded) in [
            (RelationDirection::Forward, "\"Forward\""),
            (RelationDirection::Reverse, "\"Reverse\""),
        ] {
            assert_eq!(serde_json::to_string(&direction).unwrap(), encoded);
            assert_eq!(
                serde_json::from_str::<RelationDirection>(encoded).unwrap(),
                direction
            );
        }
        assert!(RelationDirection::Forward < RelationDirection::Reverse);
    }
}
