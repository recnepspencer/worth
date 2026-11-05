mod alpha;
mod beta;
mod gamma;

pub use alpha::Alpha;

#[cfg(test)]
mod tests {
    use super::beta::Beta;

    #[test]
    fn beta_is_reachable_from_tests() {
        let _ = Beta(None);
    }
}
