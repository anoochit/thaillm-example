# Readme

Install litellm

```bash
uv tool install 'litellm[proxy]' 
```

Run litellm proxy with debug

```bash
litellm --config config.yaml --detailed_debug
```

Test with OpenAI compatible API (LiteLLM Proxy)

```bash
curl -X POST 'http://127.0.0.1:4000/chat/completions' \
-H 'Content-Type: application/json' \
-H 'Authorization: Bearer dummy' \
-d '{
    "model": "openthaigpt",
    "messages": [
      {"role": "user", "content": "สวัสดี"}
    ]
  }'
```
