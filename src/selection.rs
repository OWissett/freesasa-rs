// TODO: Remove this later once implemented. Just here to keep compiler happy
use freesasa_sys::{
    freesasa_selection, freesasa_selection_free, freesasa_selection_new,
};

use crate::{
    free_raw_c_strings, result::SasaResult, structure::Structure,
    utils::str_to_c_string,
};

#[derive(Debug)]
pub struct Selection {
    ptr: *mut freesasa_selection,
}

impl Selection {
    /// Creates a new selection from a command string, structure, and result.
    ///
    /// # Arguments
    /// * `command` - The command string to use for the selection, uses a subset of
    ///               the PyMOL selection language.
    /// * `structure_` - The structure to use for the selection.
    /// * `result_` - The result to use for the selection.
    ///
    /// # Returns
    /// * `Ok(Selection)` - The selection was created successfully.
    /// * `Err(&'static str)` - The selection could not be created or invalid command (non-ascii).
    ///
    pub fn new(
        command: &str,
        structure_: &Structure,
        result_: &SasaResult,
    ) -> Result<Self, &'static str> {
        let command = str_to_c_string(command)?.into_raw();
        let ptr = unsafe {
            freesasa_selection_new(
                command,
                structure_.as_const_ptr(),
                result_.as_const_ptr(),
            )
        };

        if ptr.is_null() {
            return Err("Failed to create freesasa selection");
        }

        free_raw_c_strings!(command);

        Ok(Self { ptr })
    }
}

impl Drop for Selection {
    fn drop(&mut self) {
        unsafe {
            freesasa_selection_free(self.ptr);
        }
    }
}

// TODO: Implement freesasa selection functions
