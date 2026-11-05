impl<S> super::Runtime<S> {
    pub fn read(&self, key: u32) -> Option<u32> {
        <Self as crate::query::QueryOps<S>>::read(self, key)
    }

    pub fn open(&self, key: u32) -> Result<u32, ()> {
        self.core().session().open(key).map_err(|e| ())
    }

    pub fn wrap(&self, key: u32) -> Option<u32> {
        Some(crate::store::lookup(key)?)
    }

    pub fn arithmetic(&self, key: u32) -> u32 {
        self.core.size(key) + 1
    }

    pub fn negated(&self, key: bool) -> bool {
        !key && true
    }

    pub fn with_macro(&self) -> String {
        format!("{}", self.0)
    }
}
