use crate::freesasa_ffi::{
    freesasa_classifier, freesasa_protor_classifier,
};

/// Very similar to the macro definition for the default classifier found in the
/// freesasa.h file:
///
/// ```C++
/// #define freesasa_default_classifier freesasa_protor_classifier
/// ```
///
pub(crate) static DEFAULT_CLASSIFIER: &freesasa_classifier =
    unsafe { &freesasa_protor_classifier };
