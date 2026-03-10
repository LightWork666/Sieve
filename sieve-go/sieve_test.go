package sieve

import (
	"testing"
)

func TestSift(t *testing.T) {
	prompt := "Hello, could you please summarize this document for me?"

	// Test Low level
	resLow := Sift(prompt, Low)
	if resLow.Original != prompt {
		t.Errorf("Expected original prompt: %s, got: %s", prompt, resLow.Original)
	}
	if resLow.Sifted == prompt {
		t.Error("Low level should have modified the prompt")
	}
	if resLow.TokensRemoved <= 0 {
		t.Error("Tokens should have been removed")
	}

	// Test High level
	resHigh := Sift(prompt, High)
	if len(resHigh.Sifted) >= len(resLow.Sifted) {
		t.Errorf("High level should compress more or equal to Low level.\nLow: %s\nHigh: %s", resLow.Sifted, resHigh.Sifted)
	}

	// Test empty prompt
	resEmpty := Sift("", High)
	if resEmpty.Original != "" || resEmpty.Sifted != "" {
		t.Error("Empty prompt should return empty results")
	}
	if resEmpty.TokensRemoved != 0 {
		t.Errorf("Empty prompt should have 0 tokens removed, got: %v", resEmpty.TokensRemoved)
	}

	// Test no-noise prompt
	clean := "Summarize the report"
	resClean := Sift(clean, High)
	if resClean.Sifted != clean {
		t.Errorf("Clean prompt should not be modified, got: %s", resClean.Sifted)
	}
}
