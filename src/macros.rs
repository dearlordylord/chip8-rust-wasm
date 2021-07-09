macro_rules! newtype_copy {
    ($t:ident) => {
        impl Clone for $t {
            fn clone(&self) -> Self {
                $t(self.0)
            }
        }
        impl Copy for $t {

        }
    }
}
pub(crate) use newtype_copy;