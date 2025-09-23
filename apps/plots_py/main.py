from fastapi import FastAPI, Body, HTTPException, Request
from fastapi.openapi.utils import get_openapi
from fastapi.responses import Response
import io
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import httpx
import csv
import os

DEBUG = os.getenv("DEBUG", "1") == "1"

app = FastAPI(
    title="plots_py",
    version="0.1.0",
    docs_url="/docs" if DEBUG else None,       # Swagger UI
    redoc_url="/redoc" if DEBUG else None,     # ReDoc
    openapi_url="/openapi.json" if DEBUG else None,  # OpenAPI JSON
)

RUST_URL = "http://stats_rs:9000"  # compose service name

@app.get("/health")
async def health():
    return {"ok": True}

@app.post("/render")
async def render(values: list[float] = Body(..., embed=False)):
    # validate
    if not isinstance(values, list) or len(values) == 0:
        raise HTTPException(status_code=400, detail="expected non-empty array of numbers")

    # compute statistics locally (or call stats_rs if you prefer)
    arr = np.array(values, dtype=float)
    count = int(arr.size)
    mean = float(arr.mean())
    median = float(np.median(arr))
    std = float(arr.std(ddof=1)) if count >= 2 else 0.0

    # create plot
    fig, ax = plt.subplots(figsize=(6,4))
    ax.plot(range(1, count+1), arr, marker="o")
    ax.set_title(f"n={count} mean={mean:.3f} median={median:.3f} sd={std:.3f}")
    ax.set_xlabel("index")
    ax.set_ylabel("value")
    ax.grid(True)

    buf = io.BytesIO()
    fig.tight_layout()
    fig.savefig(buf, format="png")
    plt.close(fig)
    buf.seek(0)
    return Response(content=buf.getvalue(), media_type="image/png")

@app.post("/render-from-stats")
async def render_from_stats():
    # Example: call stats_rs's /describe (if it provided data, but here we assume describe expects input)
    # If you want to fetch data from another service, implement that here.
    async with httpx.AsyncClient() as client:
        r = await client.get(f"{RUST_URL}/health", timeout=5.0)
        if r.status_code != 200:
            raise HTTPException(status_code=502, detail="stats_rs unhealthy")
    return {"info": "ok"}
    
@app.post("/render-csv")
async def render_csv(request: Request):
    body = await request.body()
    try:
        text = body.decode("utf-8")
    except Exception:
        raise HTTPException(status_code=400, detail="could not decode body as utf-8")

    nums = []
    rdr = csv.reader(text.splitlines())
    for rec in rdr:
        for f in rec:
            try:
                nums.append(float(f.strip()))
            except:
                continue
    if not nums:
        raise HTTPException(status_code=400, detail="no numeric data found")
    # reuse render function behavior: generate PNG
    return await render(nums)

def custom_openapi():
    if app.openapi_schema:
        return app.openapi_schema
    schema = get_openapi(
        title="plots_py",
        version="0.1.0",
        description="Rendering service: JSON/CSV → PNG",
        routes=app.routes,
    )
    # add tags, servers, or anything else here if you want
    app.openapi_schema = schema
    return app.openapi_schema

app.openapi = custom_openapi  # attach the custom generator

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="127.0.0.1", port=7000, reload=True)
