// In this crate, macros are used to define several constants in order to allow them to be reused
// in the 'shells' module's init code via concat!
// Since the concat! macro is incapable of handling consts, but can resolve macros, this ends up
// being a simple pragmatic solution.

// PAZI_EXENDED_EXIT_CODES_ENV is the environment variable which indicates exit codes should be
// used to convey expected behavior, i.e. whether output should be printed or cd'd to
macro_rules! PAZI_EXTENDED_EXIT_CODES_ENV {
    () => {
        "__PAZI_EXTENDED_EXITCODES"
    }
}

// Arbitrarily chosen exit codes
macro_rules! EXIT_CODE {
    (SUCCESS) => { 90 };
    (SUCCESS_DIR) => { 91 };
    (ERROR) => { 92 };
    (ERROR_NO_INPUT) => { 93 };
}

pub enum PaziResult {
    Success,
    SuccessDirectory,
    Error,
    ErrorNoInput,
}

impl PaziResult {
    pub fn exit_code(self) -> i32 {
        match self {
            PaziResult::Success | PaziResult::SuccessDirectory => 0,
            PaziResult::Error | PaziResult::ErrorNoInput => 1,
        }
    }

    pub fn extended_exit_code(self) -> i32 {
        match self {
            PaziResult::Success => EXIT_CODE!(SUCCESS),
            PaziResult::SuccessDirectory => EXIT_CODE!(SUCCESS_DIR),
            PaziResult::Error => EXIT_CODE!(ERROR),
            PaziResult::ErrorNoInput => EXIT_CODE!(ERROR_NO_INPUT),
        }
    }
}
