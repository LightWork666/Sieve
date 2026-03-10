package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"

	"github.com/sieve-ai/sieve-go"
)

// A mocked payload that a user might send to an LLM completion API
type CompletionRequest struct {
	Model  string `json:"model"`
	Prompt string `json:"prompt"`
}

// A mocked response from an LLM
type CompletionResponse struct {
	Response     string `json:"response"`
	TokensUsed   int    `json:"tokens_used"`
	TokensSifted int    `json:"tokens_sifted_away,omitempty"`
}

func main() {
	// 1. The Core Application Handler (the "LLM API")
	// This handler just reads the prompt and acts like an LLM.
	// IT DOES NOT KNOW ABOUT SIEVE. It simply processes whatever hits it.
	llmHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		body, err := io.ReadAll(r.Body)
		if err != nil {
			http.Error(w, "Bad request", http.StatusBadRequest)
			return
		}

		var req CompletionRequest
		if err := json.Unmarshal(body, &req); err != nil {
			http.Error(w, "Invalid JSON", http.StatusBadRequest)
			return
		}

		// Simulate the LLM receiving the prompt
		fmt.Printf("[LLM Handler] Received Prompt: %q\n", req.Prompt)

		// Mock a response based on the prompt
		resp := CompletionResponse{
			Response:   "Here is the code you requested: `def reverse(s): return s[::-1]`",
			TokensUsed: len(req.Prompt) / 4, // fake token count approximation
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	})

	// 2. Initialize the Sieve Middleware
	// We want to compress prompts aggressively ("High"), and we specify that the
	// payload key containing the text is called "prompt".
	sieveMid := sieve.NewMiddleware(
		sieve.WithLevel(sieve.High),
		sieve.WithTargetKey("prompt"),
	)

	// 3. Wrap the LLM handler
	// Sieve will intercept the request, parse the JSON, sift the "prompt" field,
	// and forward the compressed JSON to `llmHandler`.
	http.Handle("/v1/completions", sieveMid.Wrap(llmHandler))

	fmt.Println("=== Sieve Middleware Demo ===")
	fmt.Println("Server is running on http://localhost:8080")
	fmt.Println("Send a request to /v1/completions to see the middleware in action!")

	if err := http.ListenAndServe(":8080", nil); err != nil {
		log.Fatal(err)
	}
}
