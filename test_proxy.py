import urllib.request
import json
import os

req = urllib.request.Request(
    'http://localhost:4141/sift', 
    data=json.dumps({"prompt": "Hello there, I was wondering if you could please kindly help me write a function to reverse a string?", "level": "high"}).encode('utf-8'),
    headers={'Content-Type': 'application/json'}
)

try:
    with urllib.request.urlopen(req) as response:
        result = json.loads(response.read().decode('utf-8'))
        
        with open("/Users/light/Sieve/proxy_output.txt", "w") as f:
            f.write(f"Original: {result['original']}\n\n")
            f.write(f"Sifted:   {result['sifted']}\n\n")
            f.write(f"Stats:    {result['tokens_removed']} tokens removed (ratio: {result['compression_ratio']:.2f})\n")
except Exception as e:
    with open("/Users/light/Sieve/proxy_output.txt", "w") as f:
        f.write(f"Error: {str(e)}")
