# Sieve

**Sieve** is a language-agnostic, infrastructure-first library for LLM Token Optimization and Middleware. It is the "gorilla/mux" of the AI era — a tool developers can drop into their stack to automatically "sift" prompts, reducing token costs and preventing prompt injection in-flight.

## Why Sieve?

*   **Cost Reduction**: LLM APIs charge by the token. Conversational filler ("Hey, could you please...", "I was just wondering if...") wastes money and dilutes semantic intent. Sieve strips this noise before it ever hits the network.
*   **Performance**: Less tokens = faster inference (TTFT).
*   **Portability**: Available as a high-performance native library for Rust and Python, and as a pure-Go library/middleware for easy deployment.
*   **Security (Coming Soon)**: In-flight prompt injection detection and PII scrubbing.

## Architecture

At its heart is **Sieve-Core**, a regex-based noise reducer written in Rust for maximum throughput. It is exposed to Python via PyO3, and to Go via a pure-Go reimplementation (for ease of use) with identical behavior.

Depending on your stack, you can use Sieve as a library or as an ultra-fast local HTTP proxy ("Sidecar mode").

```text
       +-----------------------------------------------------+
       |                     Your App                        |
       |  +-------------+  +-------------+  +-------------+  |
       |  |  Python     |  |    Go API   |  | Any Backend |  |
       |  | (PyO3 Lib)  |  | (Middleware)|  | (Sidecar)   |  |
       |  +------+------+  +------+------+  +------+------+  |
       +---------|----------------|----------------|---------+
                 |                |                |
             <Sifted>         <Sifted>         <Sifted>
                 |                |                |
                 v                v                v
       +-----------------------------------------------------+
       |                      LLM API                        |
       |              (OpenAI, Anthropic, etc.)              |
       +-----------------------------------------------------+
```

## Quick Start

### Python (Native Binding)

```bash
pip install sieve-core
```

```python
from sieve import sift, SiftLevel

prompt = "Hello there, I was wondering if you could please summarize this document for me?"
result = sift(prompt, level=SiftLevel.High)

print(result.sifted)
# "Summarize this document"
print(f"Compressed by {result.compression_ratio:.2f}x")
```

### Go (Library & Middleware)

```bash
go get github.com/sieve-ai/sieve-go
```

#### Basic Library Usage

```go
import "github.com/sieve-ai/sieve-go"

result := sieve.Sift("Could you please help me write a function?", sieve.Medium)
fmt.Println(result.Sifted) // "help write a function?"
```

#### HTTP Middleware Example

Sieve makes it trivial to transparently sift requests passing through your API before they hit your LLM handlers.

```go
package main

import (
    "io"
    "net/http"
    "fmt"
    "github.com/sieve-ai/sieve-go"
)

func main() {
    // 1. Create your LLM handler (expects JSON with a "prompt" field)
    llmHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        body, _ := io.ReadAll(r.Body)
        // body here is already sifted by the middleware!
        fmt.Fprintf(w, "Received prompt: %s", string(body))
    })

    // 2. Wrap it with Sieve Middleware
    // We tell Sieve to look for the "prompt" key in the JSON body
    // and apply High compression.
    sieveMiddleware := sieve.NewMiddleware(
        sieve.WithLevel(sieve.High),
        sieve.WithTargetKey("prompt"),
    )

    http.Handle("/generate", sieveMiddleware.Wrap(llmHandler))
    
    fmt.Println("Server listening on :8080...")
    http.ListenAndServe(":8080", nil)
}
```

### Any Language ("Sidecar Proxy" Mode)

For languages without native bindings, run Sieve as a local background process.

```bash
# Start the Go proxy on port 4141
go run github.com/sieve-ai/sieve-go/cmd/sieve-proxy -port 4141
```

Then query it from anywhere:

```bash
curl -X POST http://localhost:4141/sift \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Could you please help me write a function?", "level": "high"}'

# Response:
# {"original":"Could you please help me write a function?","sifted":"help write a function?","tokens_removed":5,"compression_ratio":0.44}
```

## Sift Levels

*   `Low`: Removes highly obvious conversational fillers ("please", "could you kindly").
*   `Medium`: Removes fillers + hedge words ("maybe", "I think", "sort of").
*   `High`: Aggressive token scrubbing — strips almost everything non-essential ("help me", "if possible").
