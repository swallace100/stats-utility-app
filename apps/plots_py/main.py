from contextlib import suppress
import math
import io
import os
import csv
from typing import Optional

from fastapi import FastAPI, Body, HTTPException, Request
from fastapi.openapi.utils import get_openapi
from fastapi.responses import Response

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

from .models import (
    SummaryOut,
    DistOut,
    EcdfOut,
    QqOut,
    CorrMatrixOut,
    SeriesWithOutliers,
    fmt_num,
    fig_to_png,
)

DEBUG = os.getenv("DEBUG", "1") == "1"

app = FastAPI(
    title="plots_py",
    version="0.1.0",
    docs_url="/docs" if DEBUG else None,
    redoc_url="/redoc" if DEBUG else None,
    openapi_url="/openapi.json" if DEBUG else None,
)

# Point to your compose service if you add “proxy-to-Rust” endpoints
RUST_BASE = os.getenv("RUST_BASE", "http://stats_rs:9000/api/v1")


# ---------------- Health ----------------


@app.get("/health")
async def health():
    return {"ok": True}


# ---------------- Convenience renderers (raw values / CSV) ----------------
# These are handy for quick checks; your structured /plot/* routes are preferred for production flows.


@app.post("/render")
async def render(values: list[float] = Body(..., embed=False)):
    if not isinstance(values, list) or len(values) == 0:
        raise HTTPException(400, "expected non-empty array of numbers")

    arr = np.array(values, dtype=float)
    count = int(arr.size)
    mean = float(arr.mean()) if count else 0.0
    median = float(np.median(arr)) if count else 0.0
    std = float(arr.std(ddof=1)) if count >= 2 else 0.0

    fig, ax = plt.subplots(figsize=(6, 4))
    ax.plot(range(1, count + 1), arr, marker="o")
    ax.set_title(f"n={count} mean={mean:.3f} median={median:.3f} sd={std:.3f}")
    ax.set_xlabel("index")
    ax.set_ylabel("value")
    ax.grid(True)
    return Response(fig_to_png(fig), media_type="image/png")


@app.post("/render-csv")
async def render_csv(request: Request):
    body = await request.body()
    try:
        text = body.decode("utf-8")
    except UnicodeDecodeError:
        raise HTTPException(400, "could not decode body as utf-8")

    nums: list[float] = []
    with io.StringIO(text, newline="") as f:
        rdr = csv.reader(f)
        for row in rdr:
            for field in row:
                s = field.strip()
                if not s:
                    continue
                with suppress(ValueError, TypeError):
                    val = float(s)
                    if math.isfinite(val):
                        nums.append(val)

    if not nums:
        raise HTTPException(400, "no numeric data found")
    return await render(nums)


# ---------------- Plot endpoints (consume Rust-shaped JSON) ----------------


@app.post("/plot/summary", response_class=Response)
async def plot_summary(stats: SummaryOut, title: Optional[str] = None):
    fig, ax = plt.subplots(figsize=(4.8, 3.0))
    ax.axis("off")
    lines = [
        f"n = {stats.count}",
        f"mean = {fmt_num(stats.mean)}",
        f"median = {fmt_num(stats.median)}",
        f"sd = {fmt_num(stats.std)}",
        f"min/max = {fmt_num(stats.min)} / {fmt_num(stats.max)}",
        f"IQR = {fmt_num(stats.iqr)}   MAD = {fmt_num(stats.mad)}",
    ]
    ax.text(0.05, 0.95, "\n".join(lines), va="top", ha="left")
    fig.suptitle(title or "Summary")
    return Response(fig_to_png(fig), media_type="image/png")


@app.post("/plot/distribution", response_class=Response)
async def plot_distribution(dist: DistOut, title: Optional[str] = None):
    e = np.array(dist.edges, dtype=float)
    c = np.array(dist.counts, dtype=float)
    if e.size != c.size + 1:
        raise HTTPException(400, "edges must be length counts+1")

    widths = e[1:] - e[:-1]
    fig, ax = plt.subplots(figsize=(6, 4))
    ax.bar(e[:-1], c, width=widths, align="edge", edgecolor="black")
    ax.set_xlabel("value")
    ax.set_ylabel("count")
    ax.set_title(title or "Histogram")

    notes = []
    if dist.skewness is not None:
        notes.append(f"skew={dist.skewness:.3g}")
    if dist.excess_kurtosis is not None:
        notes.append(f"kurt(excess)={dist.excess_kurtosis:.3g}")
    if dist.entropy_bits is not None:
        notes.append(f"H={dist.entropy_bits:.3g} bits")
    if dist.quantiles:
        wanted = {0.05, 0.5, 0.95}
        sel = [
            f"Q{int(round(p * 100))}={v:.5g}"
            for (p, v) in dist.quantiles
            if p in wanted
        ]
        if sel:
            notes.append(", ".join(sel))
    if notes:
        ax.text(
            0.99,
            0.02,
            "  ".join(notes),
            ha="right",
            va="bottom",
            transform=ax.transAxes,
        )

    return Response(fig_to_png(fig), media_type="image/png")


@app.post("/plot/ecdf", response_class=Response)
async def plot_ecdf(ecdf: EcdfOut, title: Optional[str] = None):
    xs = np.array(ecdf.xs, dtype=float)
    ps = np.array(ecdf.ps, dtype=float)
    if xs.size != ps.size:
        raise HTTPException(400, "xs and ps must be same length")

    fig, ax = plt.subplots(figsize=(6, 4))
    ax.step(xs, ps, where="post")
    ax.set_ylim(0.0, 1.0)
    ax.set_xlabel("value")
    ax.set_ylabel("F(x)")
    ax.grid(True)
    ax.set_title(title or "ECDF")
    return Response(fig_to_png(fig), media_type="image/png")


@app.post("/plot/qq", response_class=Response)
async def plot_qq(qq: QqOut, title: Optional[str] = None):
    sq = np.array(qq.sample_quantiles, dtype=float)
    tq = np.array(qq.theoretical_quantiles, dtype=float)
    if sq.size != tq.size or sq.size == 0:
        raise HTTPException(
            400,
            "sample_quantiles and theoretical_quantiles must match and be non-empty",
        )

    fig, ax = plt.subplots(figsize=(5, 5))
    ax.scatter(tq, sq, s=10)
    lo = np.nanmin([tq.min(), sq.min()])
    hi = np.nanmax([tq.max(), sq.max()])
    ax.plot([lo, hi], [lo, hi], lw=1)
    ax.set_xlabel("Theoretical (Normal)")
    ax.set_ylabel("Sample")
    tt = title or "QQ Plot (Normal)"
    ax.set_title(f"{tt}\nμ̂={qq.mu_hat:.5g}, σ̂={qq.sigma_hat:.5g}")
    ax.grid(True)
    return Response(fig_to_png(fig), media_type="image/png")


@app.post("/plot/corr-heatmap", response_class=Response)
async def plot_corr_heatmap(cm: CorrMatrixOut, title: Optional[str] = None):
    n = int(cm.size)
    mat = np.array(cm.matrix, dtype=float)
    if mat.size != n * n:
        raise HTTPException(400, "matrix length must be size*size")
    mat = mat.reshape(n, n)

    fig, ax = plt.subplots(figsize=(5.5, 4.8))
    im = ax.imshow(mat, vmin=-1, vmax=1)
    names = cm.names or [str(i + 1) for i in range(n)]
    ax.set_xticks(np.arange(n))
    ax.set_xticklabels(names, rotation=45, ha="right")
    ax.set_yticks(np.arange(n))
    ax.set_yticklabels(names)
    ax.set_title(title or "Correlation heatmap")
    fig.colorbar(im, ax=ax, fraction=0.046)
    return Response(fig_to_png(fig), media_type="image/png")


@app.post("/plot/series", response_class=Response)
async def plot_series(payload: SeriesWithOutliers, title: Optional[str] = None):
    arr = np.array(payload.values, dtype=float)
    fig, ax = plt.subplots(figsize=(6, 4))
    ax.plot(np.arange(1, arr.size + 1), arr, marker="o", ms=3, lw=1)
    ax.set_xlabel("index")
    ax.set_ylabel("value")
    ax.grid(True)
    if payload.outliers and payload.outliers.indices:
        idx = np.array(payload.outliers.indices, dtype=int)
        idx = idx[(idx >= 0) & (idx < arr.size)]
        ax.scatter(idx + 1, arr[idx], s=30)
    ax.set_title(title or "Series")
    return Response(fig_to_png(fig), media_type="image/png")


# ---------------- Custom OpenAPI (optional) ----------------


def custom_openapi():
    if app.openapi_schema:
        return app.openapi_schema
    schema = get_openapi(
        title="plots_py",
        version="0.1.0",
        description="Rendering service: JSON/CSV → PNG",
        routes=app.routes,
    )
    app.openapi_schema = schema
    return app.openapi_schema


app.openapi = custom_openapi


# ---------------- Entrypoint ----------------

if __name__ == "__main__":
    import uvicorn

    uvicorn.run("main:app", host="127.0.0.1", port=7000, reload=True)
