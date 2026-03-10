package sieve

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
)

// Middleware handles HTTP request interception and prompt Sifting.
type Middleware struct {
	level     SiftLevel
	targetKey string // The JSON key indicating the prompt, e.g. "prompt" or "messages"
}

// Option configures the Middleware.
type Option func(*Middleware)

// WithLevel sets the compression level.
func WithLevel(level SiftLevel) Option {
	return func(m *Middleware) {
		m.level = level
	}
}

// WithTargetKey sets the JSON key to sift. Defaults to "prompt".
func WithTargetKey(key string) Option {
	return func(m *Middleware) {
		m.targetKey = key
	}
}

// NewMiddleware creates a new Sieve proxy middleware.
func NewMiddleware(opts ...Option) *Middleware {
	m := &Middleware{
		level:     Medium,
		targetKey: "prompt",
	}
	for _, opt := range opts {
		opt(m)
	}
	return m
}

// Wrap wraps an existing http.Handler.
func (m *Middleware) Wrap(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Only intercept POST/PUT requests
		if r.Method != http.MethodPost && r.Method != http.MethodPut {
			next.ServeHTTP(w, r)
			return
		}

		// Read the body
		body, err := io.ReadAll(r.Body)
		if err != nil {
			http.Error(w, "Error reading request body", http.StatusBadRequest)
			return
		}
		defer r.Body.Close()

		// Attempt to parse JSON and sift the target key
		var payload map[string]interface{}
		if err := json.Unmarshal(body, &payload); err == nil {
			if promptVal, ok := payload[m.targetKey]; ok {
				if promptStr, ok := promptVal.(string); ok {
					// Sift!
					result := Sift(promptStr, m.level)
					// Update the payload
					payload[m.targetKey] = result.Sifted

					// Re-marshal to inject back into the request
					if newBody, err := json.Marshal(payload); err == nil {
						r.Body = io.NopCloser(bytes.NewBuffer(newBody))
						r.ContentLength = int64(len(newBody))
					} else {
						// Fallback if re-marshal fails
						r.Body = io.NopCloser(bytes.NewBuffer(body))
					}
				} else {
					r.Body = io.NopCloser(bytes.NewBuffer(body))
				}
			} else {
				r.Body = io.NopCloser(bytes.NewBuffer(body))
			}
		} else {
			// Not JSON or parse failed; restore original body
			r.Body = io.NopCloser(bytes.NewBuffer(body))
		}

		next.ServeHTTP(w, r)
	})
}
