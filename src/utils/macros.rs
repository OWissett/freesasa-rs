#[macro_export]
macro_rules! free_raw_c_strings {
    ( $( $x:expr ),* ) => {
        {unsafe {
            $(
                if $x.is_null() {
                    error!("Fatal error: tried to free a null pointer!");
                    panic!("Tried to free a null pointer!");
                }
                let _ = std::ffi::CString::from_raw($x);
            )*
        }}
    };
}
