import os
import httpx, uvicorn
from dotenv import load_dotenv
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

load_dotenv()

app = FastAPI()
TARGET = "http://thaillm.or.th/api/typhoon/v1"
APIKEY = os.getenv("THAILLM_APIKEY", "your_api_key_here")

@app.api_route("/{path:path}", methods=["GET", "POST", "PUT", "DELETE"])
async def proxy(path: str, request: Request):
    print(f">>> HIT: {request.method} /{path}")
    body = await request.json()
    print(f">>> Body: {body}")
    body["model"] = "/model"

    async with httpx.AsyncClient() as client:
        resp = await client.post(
            f"{TARGET}/chat/completions",
            json=body,
            headers={"Content-Type": "application/json", "apikey": APIKEY},
            timeout=60,
        )

    print(f">>> Upstream status: {resp.status_code}")
    print(f">>> Upstream body: {resp.text}")

    return JSONResponse(content=resp.json(), status_code=resp.status_code)

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8080)