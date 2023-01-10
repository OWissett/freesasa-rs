#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi;

    #[test]
    fn freesasa_calculation() {
        unsafe {
            let pdb_file_path = "../data/5XH3.pdb";
        }
    }
}