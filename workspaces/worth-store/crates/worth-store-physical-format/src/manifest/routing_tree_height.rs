pub fn required_tree_level(entry_count: u64, capacity: u16) -> Option<u16> {
    if entry_count == 0 || capacity < 2 {
        return None;
    }
    let capacity = u64::from(capacity);
    let mut level = 0_u16;
    let mut nodes = entry_count.div_ceil(capacity);
    while nodes > 1 {
        nodes = nodes.div_ceil(capacity);
        level = level.checked_add(1)?;
    }
    Some(level)
}

#[cfg(test)]
mod tests {
    use super::required_tree_level;

    #[test]
    fn height_is_the_minimum_fixed_fanout_tree_cover() {
        assert_eq!(required_tree_level(0, 2), None);
        assert_eq!(required_tree_level(1, 2), Some(0));
        assert_eq!(required_tree_level(2, 2), Some(0));
        assert_eq!(required_tree_level(3, 2), Some(1));
        assert_eq!(required_tree_level(4, 2), Some(1));
        assert_eq!(required_tree_level(5, 2), Some(2));
        assert_eq!(required_tree_level(u64::MAX, 2), Some(63));
    }
}
