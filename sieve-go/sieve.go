// Package sieve provides a high-performance LLM token optimization engine.
// The pure-Go implementation of the noise reducer.
package sieve

import (
	"regexp"
	"strings"
)

// SiftLevel controls how aggressively Sieve compresses a prompt.
type SiftLevel string

const (
	// Low removes only obvious filler phrases ("please", "could you kindly").
	Low SiftLevel = "low"
	// Medium removes filler + hedge words ("maybe", "I think", "sort of").
	Medium SiftLevel = "medium"
	// High aggressively strips everything non-essential.
	High SiftLevel = "high"
)

// SiftResult is the output of a Sift operation.
type SiftResult struct {
	Original         string  `json:"original"`
	Sifted           string  `json:"sifted"`
	TokensRemoved    int     `json:"tokens_removed"`
	CompressionRatio float64 `json:"compression_ratio"`
}

var (
	fillerPhrases = []string{
		"i was just wondering if", "i was wondering if", "would you be so kind as to",
		"would it be possible to", "could you please kindly", "if you don't mind",
		"if it's not too much trouble", "i would really appreciate it if you could",
		"i would appreciate it if you could", "would you be able to", "could you kindly",
		"could you please", "would you mind", "would you please", "can you please",
		"i'd like you to", "i want you to", "hey there", "hi there", "hello there",
		"thanks so much", "thanks a lot", "thank you so much", "thank you very much",
		"thanks in advance", "thank you in advance", "please kindly", "please",
		"kindly", "thanks", "thank you", "hello", "hey", "hi",
	}

	hedgeWords = []string{
		"sort of", "kind of", "more or less", "in a way", "to some extent",
		"at the end of the day", "as a matter of fact", "in my opinion",
		"i think that", "i believe that", "i feel like", "i guess", "i think",
		"i believe", "i feel", "i suppose", "maybe", "perhaps", "possibly",
		"probably", "basically", "essentially", "actually", "just", "really",
		"very", "quite", "literally",
	}

	aggressivePhrases = []string{
		"for me", "for us", "if you can", "if possible", "when you get a chance",
		"at your earliest convenience", "as soon as possible", "go ahead and",
		"i need you to", "i want to", "help me", "assist me", "can you",
		"could you", "would you",
	}

	wsRegex = regexp.MustCompile(`\s{2,}`)
)

// Sift processes a prompt to remove noise according to the given level.
func Sift(prompt string, level SiftLevel) *SiftResult {
	if prompt == "" {
		return &SiftResult{
			Original:         "",
			Sifted:           "",
			TokensRemoved:    0,
			CompressionRatio: 1.0,
		}
	}

	sifted := reduce(prompt, level)

	originalTokens := countTokens(prompt)
	siftedTokens := countTokens(sifted)
	tokensRemoved := originalTokens - siftedTokens
	if tokensRemoved < 0 {
		tokensRemoved = 0
	}

	compressionRatio := 1.0
	if originalTokens > 0 {
		compressionRatio = float64(siftedTokens) / float64(originalTokens)
	}

	return &SiftResult{
		Original:         prompt,
		Sifted:           sifted,
		TokensRemoved:    tokensRemoved,
		CompressionRatio: compressionRatio,
	}
}

func countTokens(s string) int {
	return len(strings.Fields(s))
}

func reduce(input string, level SiftLevel) string {
	text := input

	// Phase 1: Low
	text = removePhrases(text, fillerPhrases)

	// Phase 2: Medium/High
	if level == Medium || level == High {
		text = removePhrases(text, hedgeWords)
	}

	// Phase 3: High
	if level == High {
		text = removePhrases(text, aggressivePhrases)
	}

	// Normalize whitespace
	text = wsRegex.ReplaceAllString(text, " ")
	text = strings.TrimSpace(text)

	// Strip leading and trailing punctuation artifacts
	text = strings.TrimLeft(text, ",.!? ")
	text = strings.TrimRight(text, ", ")
	text = strings.TrimSpace(text)
	
	// Preserve the original trailing punctuation (if any)
	if len(input) > 0 {
		lastChar := input[len(input)-1]
		if lastChar == '?' || lastChar == '!' || lastChar == '.' {
			if len(text) > 0 && text[len(text)-1] != lastChar {
				// avoid double punctuation if the noise reducer didn't strip it
				text += string(lastChar)
			}
		}
	}

	return text
}

func removePhrases(input string, phrases []string) string {
	result := input
	for _, phrase := range phrases {
		// Use regex for case-insensitive matching with word boundaries
		// Note: compiling in a loop is slow, this is a naive v0.1 implementation.
		// For high performance, everything should be precompiled.
		pattern := `(?i)\b` + regexp.QuoteMeta(phrase) + `\b`
		re := regexp.MustCompile(pattern)
		result = re.ReplaceAllString(result, "")
	}
	return result
}
