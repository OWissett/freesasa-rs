use freesasa_sys::{freesasa_classifier, freesasa_protor_classifier};

/// Very similar to the macro definition for the default classifier found in the
/// freesasa.h file:
///
/// ```C++
/// #define freesasa_default_classifier freesasa_protor_classifier
/// ```
///
pub(crate) static DEFAULT_CLASSIFIER: &freesasa_classifier =
    unsafe { &freesasa_protor_classifier };

// https://freesasa.github.io/doxygen/group__classifier.html

#[allow(dead_code)]
pub(crate) static NACCESS_CLASSIFIER: &freesasa_classifier =
    unsafe { &freesasa_sys::freesasa_naccess_classifier };

#[allow(dead_code)]
pub(crate) static OONS_CLASSIFIER: &freesasa_classifier =
    unsafe { &freesasa_sys::freesasa_oons_classifier };

// We need some sort of way for people to set which classifier they want to use.
