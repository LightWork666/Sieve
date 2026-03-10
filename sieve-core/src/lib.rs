// Sieve-Core: High-performance LLM token optimization engine
//
// This is the main library entry point. It exposes the core `sift()` function
// which removes noise from prompts to reduce token consumption.

pub mod noise;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "python")]
pub mod python;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// Controls how aggressively Sieve compresses a prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SiftLevel {
    /// Remove only obvious filler phrases ("please", "could you kindly").
    Low,
    /// Remove filler + hedge words ("maybe", "I think", "sort of").
    Medium,
    /// Aggressive compression — strip everything non-essential.
    High,
}

impl SiftLevel {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" | "l" => SiftLevel::Low,
            "high" | "h" => SiftLevel::High,
            _ => SiftLevel::Medium,
        }
    }
}

/// The result of a sift operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiftResult {
    /// The original prompt, unchanged.
    pub original: String,
    /// The sifted (compressed) prompt.
    pub sifted: String,
    /// Estimated number of whitespace-delimited tokens removed.
    pub tokens_removed: usize,
    /// Compression ratio (0.0–1.0). Lower is better.
    pub compression_ratio: f64,
}

// ---------------------------------------------------------------------------
// Core public function
// ---------------------------------------------------------------------------

/// Sift a prompt: remove noise according to the given level.
///
/// # Examples
/// ```
/// use sieve_core::{sift, SiftLevel};
///
/// let result = sift("Hey, could you please summarize this document for me?", SiftLevel::High);
/// assert!(result.sifted.len() < result.original.len());
/// ```
pub fn sift(prompt: &str, level: SiftLevel) -> SiftResult {
    let sifted = noise::reduce(prompt, level);

    let original_tokens = count_tokens(prompt);
    let sifted_tokens = count_tokens(&sifted);
    let tokens_removed = original_tokens.saturating_sub(sifted_tokens);

    let compression_ratio = if original_tokens == 0 {
        1.0
    } else {
        sifted_tokens as f64 / original_tokens as f64
    };

    SiftResult {
        original: prompt.to_string(),
        sifted,
        tokens_removed,
        compression_ratio,
    }
}

/// Naive whitespace-delimited token counter (placeholder for a real tokenizer).
fn count_tokens(s: &str) -> usize {
    s.split_whitespace().count()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sift_low() {
        let r = sift(
            "Hello, could you please summarize this document for me?",
            SiftLevel::Low,
        );
        assert!(!r.sifted.contains("please"));
        assert!(r.sifted.contains("summarize"));
    }

    #[test]
    fn test_sift_high_compresses_more() {
        let prompt = "Hey there, I was just wondering if you could maybe possibly help me summarize this document";
        let low = sift(prompt, SiftLevel::Low);
        let high = sift(prompt, SiftLevel::High);
        assert!(
            high.sifted.len() <= low.sifted.len(),
            "High should compress at least as much as Low"
        );
    }

    #[test]
    fn test_empty_prompt() {
        let r = sift("", SiftLevel::High);
        assert_eq!(r.sifted, "");
        assert_eq!(r.tokens_removed, 0);
    }

    #[test]
    fn test_compression_ratio_range() {
        let r = sift("Please kindly help me with this task", SiftLevel::Medium);
        assert!(r.compression_ratio >= 0.0 && r.compression_ratio <= 1.0);
    }
}
