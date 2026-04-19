# Readme

Install dependencies

```bash
pip install -r requirements.txt
```

Run proxy

```bash
python typhoon_proxy.py
```

Run litellm proxy with debug

```bash
litellm --config litellm_config.yaml --detailed_debug
```

Test with OpenAI compatible API (Proxy)

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "typhoon",
    "messages": [
      {"role": "user", "content": "สวัสดี"}
    ],
    "max_tokens": 100
  }'
```

Test with OpenAI compatible API (LiteLLM Proxy)

```bash
curl http://localhost:4000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer dummy" \
  -d '{
    "model": "typhoon",
    "messages": [
      {"role": "user", "content": "สวัสดี"}
    ],
    "max_tokens": 100
  }'
```
