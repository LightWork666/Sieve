package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"

	"github.com/sieve-ai/sieve-go"
)

// SiftRequest is the expected JSON payload.
type SiftRequest struct {
	Prompt string `json:"prompt"`
	Level  string `json:"level"`
}

func main() {
	port := flag.Int("port", 4141, "Port to listen on")
	flag.Parse()

	http.HandleFunc("/sift", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req SiftRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "Invalid JSON payload", http.StatusBadRequest)
			return
		}

		level := sieve.Medium // default
		switch req.Level {
		case "low":
			level = sieve.Low
		case "high":
			level = sieve.High
		}

		result := sieve.Sift(req.Prompt, level)

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(result); err != nil {
			log.Printf("Error encoding response: %v", err)
			http.Error(w, "Internal server error", http.StatusInternalServerError)
		}
	})

	addr := fmt.Sprintf(":%d", *port)
	log.Printf("Starting Sieve local sidecar proxy on http://localhost%s", addr)
	log.Printf("  POST /sift -d '{\"prompt\": \"...\", \"level\": \"high\"}'")

	if err := http.ListenAndServe(addr, nil); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}
