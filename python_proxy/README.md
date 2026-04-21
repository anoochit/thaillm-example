# Readme

Install dependencies

```bash
pip install -r requirements.txt
```

Run proxy

```bash
python typhoon_proxy.py
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
