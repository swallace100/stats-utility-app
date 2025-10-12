from fastapi.testclient import TestClient
from plots_py.main import app

client = TestClient(app)


# ---------- helpers ----------


def assert_png(resp):
    assert resp.status_code == 200
    ctype = resp.headers.get("content-type", "")
    assert ctype.startswith("image/png"), f"unexpected content-type: {ctype}"
    assert resp.content and len(resp.content) > 100  # non-empty image


# ---------- basic endpoints ----------


def test_health():
    r = client.get("/health")
    assert r.status_code == 200
    assert r.json() == {"ok": True}


def test_render_basic():
    r = client.post("/render", json=[1, 2, 3, 4])
    assert_png(r)


def test_render_csv_no_numeric():
    r = client.post("/render-csv", data="a,b\nx,y\n")
    assert r.status_code == 400


# ---------- /plot/summary ----------


def test_plot_summary_ok():
    payload = {
        "count": 7,
        "mean": 2.5,
        "median": 2.0,
        "std": 1.1,
        "min": 0.0,
        "max": 5.0,
        "iqr": 1.3,
        "mad": 0.9,
    }
    r = client.post("/plot/summary", json=payload)
    assert_png(r)


# ---------- /plot/distribution ----------


def test_plot_distribution_ok_prebinned():
    payload = {
        "counts": [1, 3, 2],
        "edges": [0.0, 1.0, 2.0, 3.0],  # len(edges) == len(counts)+1
        "quantiles": [[0.05, 0.2], [0.5, 1.5], [0.95, 2.8]],
        "skewness": 0.12,
        "excess_kurtosis": -0.5,
        "entropy_bits": 1.23,
    }
    r = client.post("/plot/distribution", json=payload)
    assert_png(r)


def test_plot_distribution_edges_mismatch_400():
    payload = {
        "counts": [1, 3, 2],
        "edges": [0.0, 1.0, 2.0],  # WRONG: should be 4 edges
        "quantiles": [],
    }
    r = client.post("/plot/distribution", json=payload)
    assert r.status_code == 400


# ---------- /plot/ecdf ----------


def test_plot_ecdf_ok():
    payload = {"xs": [1.0, 2.0, 3.0], "ps": [0.33, 0.66, 1.0]}
    r = client.post("/plot/ecdf", json=payload)
    assert_png(r)


def test_plot_ecdf_len_mismatch_400():
    payload = {"xs": [1.0, 2.0], "ps": [0.5]}  # mismatch
    r = client.post("/plot/ecdf", json=payload)
    assert r.status_code == 400


# ---------- /plot/qq ----------


def test_plot_qq_ok():
    payload = {
        "sample_quantiles": [0.0, 1.0, 2.0, 3.0],
        "theoretical_quantiles": [-0.5, 0.0, 0.5, 1.0],
        "mu_hat": 1.5,
        "sigma_hat": 0.8,
    }
    r = client.post("/plot/qq", json=payload)
    assert_png(r)


def test_plot_qq_len_mismatch_400():
    payload = {
        "sample_quantiles": [0.0, 1.0],
        "theoretical_quantiles": [-0.5],  # mismatch
        "mu_hat": 1.0,
        "sigma_hat": 1.0,
    }
    r = client.post("/plot/qq", json=payload)
    assert r.status_code == 400


# ---------- /plot/corr-heatmap ----------


def test_plot_corr_heatmap_ok():
    # 3x3 matrix flattened row-major
    payload = {
        "size": 3,
        "names": ["a", "b", "c"],
        "matrix": [1.0, 0.1, -0.2, 0.1, 1.0, 0.3, -0.2, 0.3, 1.0],
    }
    r = client.post("/plot/corr-heatmap", json=payload)
    assert_png(r)


def test_plot_corr_heatmap_size_mismatch_400():
    payload = {
        "size": 3,
        "matrix": [1.0, 0.0, 0.0, 0.0],  # only 4 values; need 9
    }
    r = client.post("/plot/corr-heatmap", json=payload)
    assert r.status_code == 400


# ---------- /plot/series ----------


def test_plot_series_ok_with_outliers():
    payload = {
        "values": [1, 2, 100, 3, 4, -50, 5],
        "outliers": {
            "indices": [2, 5, 99],  # 99 is out-of-range; should be ignored safely
            "values": [100, -50, 999],
        },
    }
    r = client.post("/plot/series", json=payload)
    assert_png(r)
