// Noise Reducer — the heart of Sieve.
//
// Strips conversational filler, hedge words, and pleasantries from prompts
// while preserving semantic intent.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::SiftLevel;

// ---------------------------------------------------------------------------
// Phrase lists (ordered longest-first so greedy removal works correctly)
// ---------------------------------------------------------------------------

/// Filler phrases removed at ALL levels (Low, Medium, High).
const FILLER_PHRASES: &[&str] = &[
    "i was just wondering if",
    "i was wondering if",
    "would you be so kind as to",
    "would it be possible to",
    "could you please kindly",
    "if you don't mind",
    "if it's not too much trouble",
    "i would really appreciate it if you could",
    "i would appreciate it if you could",
    "would you be able to",
    "could you kindly",
    "could you please",
    "would you mind",
    "would you please",
    "can you please",
    "i'd like you to",
    "i want you to",
    "hey there",
    "hi there",
    "hello there",
    "thanks so much",
    "thanks a lot",
    "thank you so much",
    "thank you very much",
    "thanks in advance",
    "thank you in advance",
    "please kindly",
    "please",
    "kindly",
    "thanks",
    "thank you",
    "hello",
    "hey",
    "hi",
];

/// Hedge words removed at Medium and High.
const HEDGE_WORDS: &[&str] = &[
    "sort of",
    "kind of",
    "more or less",
    "in a way",
    "to some extent",
    "at the end of the day",
    "as a matter of fact",
    "in my opinion",
    "i think that",
    "i believe that",
    "i feel like",
    "i guess",
    "i think",
    "i believe",
    "i feel",
    "i suppose",
    "maybe",
    "perhaps",
    "possibly",
    "probably",
    "basically",
    "essentially",
    "actually",
    "just",
    "really",
    "very",
    "quite",
    "literally",
];

/// Extra aggressive phrases removed only at High.
const AGGRESSIVE_PHRASES: &[&str] = &[
    "for me",
    "for us",
    "if you can",
    "if possible",
    "when you get a chance",
    "at your earliest convenience",
    "as soon as possible",
    "go ahead and",
    "i need you to",
    "i want to",
    "help me",
    "assist me",
    "can you",
    "could you",
    "would you",
];

// Precompiled regex for collapsing whitespace.
static WS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}").unwrap());

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Reduce noise in `input` according to the specified `level`.
pub fn reduce(input: &str, level: SiftLevel) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Work on a lowercased copy for matching, but rebuild from original casing
    // where possible. For v0.1 we lowercase everything for simplicity — a
    // future version will preserve original casing via span tracking.
    let mut text = input.to_string();

    // Phase 1: Remove filler phrases (all levels)
    text = remove_phrases(&text, FILLER_PHRASES);

    // Phase 2: Remove hedge words (Medium, High)
    if matches!(level, SiftLevel::Medium | SiftLevel::High) {
        text = remove_phrases(&text, HEDGE_WORDS);
    }

    // Phase 3: Aggressive removal (High only)
    if matches!(level, SiftLevel::High) {
        text = remove_phrases(&text, AGGRESSIVE_PHRASES);
    }

    // Normalize whitespace and trim.
    let text = WS.replace_all(&text, " ");
    let text = text.trim();

    // Strip leading and trailing punctuation artifacts.
    let text = text.trim_start_matches(|c: char| c == ',' || c == '.' || c == '!' || c == '?' || c == ' ');
    let text = text.trim_end_matches(|c: char| c == ',' || c == ' ');
    let mut text = text.trim().to_string();

    // Preserve the original trailing punctuation (if any)
    if let Some(last_char) = input.chars().last() {
        if last_char == '?' || last_char == '!' || last_char == '.' {
            if !text.ends_with(last_char) {
                text.push(last_char);
            }
        }
    }

    text
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Case-insensitive phrase removal.
fn remove_phrases(input: &str, phrases: &[&str]) -> String {
    let mut result = input.to_string();
    for phrase in phrases {
        // Build a case-insensitive pattern with word-ish boundaries.
        let pattern = format!(r"(?i)\b{}\b", regex::escape(phrase));
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, "").to_string();
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filler_removal() {
        let input = "Hello, could you please summarize this document for me?";
        let out = reduce(input, SiftLevel::Low);
        assert!(!out.to_lowercase().contains("please"));
        assert!(!out.to_lowercase().contains("hello"));
        assert!(out.to_lowercase().contains("summarize"));
    }

    #[test]
    fn test_hedge_removal_medium() {
        let input = "I think maybe you should just summarize this";
        let out = reduce(input, SiftLevel::Medium);
        assert!(!out.to_lowercase().contains("i think"));
        assert!(!out.to_lowercase().contains("maybe"));
        assert!(!out.to_lowercase().contains("just"));
        assert!(out.to_lowercase().contains("summarize"));
    }

    #[test]
    fn test_aggressive_high() {
        let input = "Could you help me summarize this document for me if possible?";
        let out = reduce(input, SiftLevel::High);
        assert!(!out.to_lowercase().contains("help me"));
        assert!(!out.to_lowercase().contains("for me"));
        assert!(out.to_lowercase().contains("summarize"));
    }

    #[test]
    fn test_empty() {
        assert_eq!(reduce("", SiftLevel::High), "");
    }

    #[test]
    fn test_no_noise() {
        let input = "Summarize the quarterly earnings report";
        let out = reduce(input, SiftLevel::High);
        assert_eq!(out, input);
    }

    #[test]
    fn test_whitespace_normalization() {
        let input = "Please   kindly   summarize   this";
        let out = reduce(input, SiftLevel::Low);
        assert!(!out.contains("  "), "Should not contain double spaces");
    }
}
