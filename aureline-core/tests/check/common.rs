#[allow(dead_code, unused_imports)]
pub use aureline_test_support::{
    ExpectedDiagnostic, assert_range, assert_single_diagnostic, diagnostics_for, only_diagnostic,
};

macro_rules! aureline_schema {
    ($($line:literal),* $(,)?) => {
        aureline_test_support::aureline_schema!($($line),*)
    };
}
