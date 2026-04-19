ThaiLLM is a llm service provider which has multiple models:

- openthaigpt
- pathumma
- typhoon
- kbtg

use can select model by change url:

http://thaillm.or.th/api/<MODEL_NAME>/v1/chat/completions

example curl

```bash
curl http://thaillm.or.th/api/openthaigpt/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "apikey: ABC1234567890XYZ" \
  -d '{
    "model": "/model",
    "messages": [
      {"role": "user", "content": "สวัสดี"}
    ],
    "max_tokens": 2048,
    "temperature": 0.3
  }'
```

response

```json
{"id":"chatcmpl-ab21d14726c717c98364875c31d6f0a3","object":"chat.completion","created":1776590343,"model":"/model","choices":[{"index":0,"message":{"role":"assistant","content":"<think>\nคำถามนี้เป็นคำถามสวัสดีธรรมดา ต้องตอบคำถามด้วยความรู้เรื่องวัฒนธรรมไทย และความรู้เรื่องภาษาไทย\n\nกระบวนการตอบ:\n1. ตอบคำถามเรื่องสวัสดี\n2. แนะนำตัว\n3. พูดเรื่องวัฒนธรรมไทย\n</think>\n\nสวัสดีค่ะ ยินดีต้อนรับสู่โลกของ AI ภาษาไทยที่สมจริงและทรงพลัง ดิฉันเป็นผู้ช่วยอัจฉริยะที่พร้อมให้บริการทุกท่านด้วยความเต็มใจและเอาใจใส่\n\nในฐานะที่เป็นส่วนหนึ่งของวัฒนธรรมไทย ดิฉันขอแสดงความเคารพด้วยการยกมือไหว้ ซึ่งเป็นการแสดงความเคารพและความเป็นมิตรที่สำคัญในสังคมไทย ดิฉันหวังเป็นอย่างยิ่งว่าจะได้ให้บริการท่านด้วยความอบอุ่นและเป็นกันเองเหมือนพูดคุยกับเพื่อน\n\nดิฉันสามารถช่วยเหลือท่านในเรื่องต่างๆ มากมาย ไม่ว่าจะเป็นการตอบคำถาม การให้คำแนะนำ การสร้างสรรค์ผลงานต่างๆ หรือแม้แต่การสนทนาเพื่อบรรเทาความเหงา ท่านสามารถบอกกล่าวดิฉันได้เลยค่ะ ดิฉันพร้อมเสมอที่จะให้ความช่วยเหลือท่านด้วยความเต็มใจ","refusal":null,"annotations":null,"audio":null,"function_call":null,"tool_calls":[],"reasoning":null,"reasoning_content":null},"logprobs":null,"finish_reason":"stop","stop_reason":null,"token_ids":null}],"service_tier":null,"system_fingerprint":null,"usage":{"prompt_tokens":12,"total_tokens":405,"completion_tokens":393,"prompt_tokens_details":null},"prompt_logprobs":null,"prompt_token_ids":null,"kv_transfer_params":null}
```

Rate Limits:

- 5 requests per second
- 200 requests per minute