use sha2::{Digest, Sha256};

std::thread_local! {
    static TEST_HASH_PARTS_CALL_COUNT: std::cell::Cell<u64> = const {
        std::cell::Cell::new(0)
    };
    static TEST_HASH_PARTS_DOMAINS: std::cell::RefCell<Vec<String>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

pub(crate) fn hash_parts(parts: &[String]) -> String {
    TEST_HASH_PARTS_CALL_COUNT.with(|count| {
        count.set(
            count
                .get()
                .checked_add(1)
                .expect("test execution-digest counter must not wrap"),
        );
    });
    TEST_HASH_PARTS_DOMAINS.with(|domains| {
        domains
            .borrow_mut()
            .push(parts.first().cloned().unwrap_or_default());
    });
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn test_hash_parts_call_count() -> u64 {
    TEST_HASH_PARTS_CALL_COUNT.with(std::cell::Cell::get)
}

pub fn test_hash_parts_domains_since(start: u64) -> Vec<String> {
    let start = usize::try_from(start).expect("test digest count fits usize");
    TEST_HASH_PARTS_DOMAINS.with(|domains| domains.borrow()[start..].to_vec())
}
