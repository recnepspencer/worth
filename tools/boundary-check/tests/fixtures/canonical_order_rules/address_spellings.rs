mod sort_key {
    fn order(xs: &mut Vec<Arc<Token>>) {
        xs.sort_by_key(|x| Arc::as_ptr(x) as usize);
    }
}
mod max_key {
    fn order(xs: &[Arc<Token>]) {
        xs.iter().max_by_key(|x| Arc::as_ptr(x) as usize);
    }
}
mod ordered_map {
    fn order(x: &Token) {
        let mut index: BTreeMap<usize, ()> = BTreeMap::new();
        index.insert(std::ptr::from_ref(x) as usize, ());
    }
}
mod helper {
    impl Token {
        fn address(&self) -> usize {
            Arc::as_ptr(&self.0) as usize
        }
    }
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            self.address().cmp(&other.address())
        }
    }
}
mod cast_only {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            (self as *const Self as usize).cmp(&(other as *const Self as usize))
        }
    }
}
mod bare_from_ref {
    use std::ptr::from_ref;
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            from_ref(self).cmp(&from_ref(other))
        }
    }
}
mod qualified_from_ref {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            ptr::from_ref(self).cmp(&ptr::from_ref(other))
        }
    }
}
mod addr_of {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            std::ptr::addr_of!(self.0).cmp(&std::ptr::addr_of!(other.0))
        }
    }
}
mod raw_const {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            (&raw const self.0).cmp(&(&raw const other.0))
        }
    }
}
mod into_raw {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            Arc::into_raw(self.0.clone()).cmp(&Arc::into_raw(other.0.clone()))
        }
    }
}
mod non_null {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            NonNull::from(self).cmp(&NonNull::from(other))
        }
    }
}
mod expose_provenance {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.expose_provenance().cmp(&other.0.expose_provenance())
        }
    }
}
mod as_mut_ptr {
    impl Ord for Token {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.as_mut_ptr().cmp(&other.0.as_mut_ptr())
        }
    }
}
mod derive_pointer {
    #[derive(Ord, PartialOrd)]
    struct Token {
        address: *const u8,
    }
}
mod local_address {
    fn order(x: &Token, y: &Token) {
        let a = std::ptr::from_ref(x) as usize;
        let b = std::ptr::from_ref(y) as usize;
        a.cmp(&b);
    }
}
