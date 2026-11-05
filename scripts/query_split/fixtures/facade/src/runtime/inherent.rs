impl<S: Clone> Runtime<S> {
    pub fn thin_field(&self) -> &S {
        &self.0
    }

    pub fn branchy(&self, flag: bool) -> u32 {
        if flag { 1 } else { 2 }
    }

    fn private_logic(&self) -> u32 {
        loop {}
    }

    #[cfg(test)]
    pub fn test_only(&self) -> u32 {
        match 1 { _ => 2 }
    }
}

impl<S> std::fmt::Debug for Runtime<S> {
    pub fn fmt(&self) -> u32 {
        if true { 1 } else { 2 }
    }
}

impl<S> Other<S> {
    pub fn not_ours(&self) -> u32 {
        1
    }
}
