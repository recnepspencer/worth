pub const VALUE: u8 = 40;

#[cfg(test)]
mod tests {
    #[test]
    fn constant_is_positive() {
        assert!(super::VALUE > 0);
    }
}
