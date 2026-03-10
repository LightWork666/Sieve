// PyO3 bindings — exposes sieve_core to Python as a native module.

use pyo3::prelude::*;

use crate::{sift as core_sift, SiftLevel as CoreSiftLevel};

/// Python-visible sift level enum.
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiftLevel {
    Low = 0,
    Medium = 1,
    High = 2,
}

/// Python-visible sift result.
#[pyclass]
#[derive(Debug, Clone)]
pub struct SiftResult {
    #[pyo3(get)]
    pub original: String,
    #[pyo3(get)]
    pub sifted: String,
    #[pyo3(get)]
    pub tokens_removed: usize,
    #[pyo3(get)]
    pub compression_ratio: f64,
}

#[pymethods]
impl SiftResult {
    fn __repr__(&self) -> String {
        format!(
            "SiftResult(tokens_removed={}, compression_ratio={:.2}, sifted={:?})",
            self.tokens_removed, self.compression_ratio, self.sifted
        )
    }
}

/// Sift a prompt to reduce token noise.
///
/// Args:
///     prompt (str): The input prompt to optimize.
///     level (SiftLevel): Compression aggressiveness. Default: Medium.
///
/// Returns:
///     SiftResult: The optimization result.
#[pyfunction]
#[pyo3(signature = (prompt, level=SiftLevel::Medium))]
fn sift(prompt: &str, level: SiftLevel) -> SiftResult {
    let core_level = match level {
        SiftLevel::Low => CoreSiftLevel::Low,
        SiftLevel::Medium => CoreSiftLevel::Medium,
        SiftLevel::High => CoreSiftLevel::High,
    };

    let r = core_sift(prompt, core_level);

    SiftResult {
        original: r.original,
        sifted: r.sifted,
        tokens_removed: r.tokens_removed,
        compression_ratio: r.compression_ratio,
    }
}

/// The sieve_core Python module.
#[pymodule]
fn sieve_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SiftLevel>()?;
    m.add_class::<SiftResult>()?;
    m.add_function(wrap_pyfunction!(sift, m)?)?;
    Ok(())
}
