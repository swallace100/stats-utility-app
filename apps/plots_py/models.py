from typing import List, Optional, Tuple
from pydantic import BaseModel
import io
import matplotlib.pyplot as plt

# ---------- Pydantic mirrors of your Rust outputs ----------


class SummaryOut(BaseModel):
    count: int
    mean: Optional[float] = None
    median: Optional[float] = None
    std: Optional[float] = None
    min: Optional[float] = None
    max: Optional[float] = None
    iqr: Optional[float] = None
    mad: Optional[float] = None


class DistOut(BaseModel):
    counts: List[int]
    edges: List[float]
    # Rust Vec<(f64,f64)> arrives in JSON as a list of two-item arrays
    quantiles: List[Tuple[float, float]]
    skewness: Optional[float] = None
    excess_kurtosis: Optional[float] = None
    entropy_bits: Optional[float] = None


class EcdfOut(BaseModel):
    xs: List[float]
    ps: List[float]


class QqOut(BaseModel):
    sample_quantiles: List[float]
    theoretical_quantiles: List[float]
    mu_hat: float
    sigma_hat: float


class CorrMatrixOut(BaseModel):
    size: int
    names: Optional[List[str]] = None
    matrix: List[float]


class OutliersOut(BaseModel):
    indices: List[int]
    values: List[float]


class SeriesWithOutliers(BaseModel):
    values: List[float]
    outliers: Optional[OutliersOut] = None


# ---------- tiny formatting + PNG helpers (used by routes) ----------


def fmt_num(v: Optional[float]) -> str:
    return "—" if v is None else f"{v:.5g}"


def fig_to_png(fig) -> bytes:
    buf = io.BytesIO()
    fig.tight_layout()
    fig.savefig(buf, format="png")
    plt.close(fig)
    buf.seek(0)
    return buf.getvalue()
