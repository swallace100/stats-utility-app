from fastapi.testclient import TestClient
from plots_py.main import app

client = TestClient(app)


def test_health():
    r = client.get("/health")
    assert r.status_code == 200
    assert r.json() == {"ok": True}


def test_render_basic():
    r = client.post("/render", json=[1, 2, 3, 4])
    assert r.status_code == 200
    assert r.headers["content-type"].startswith("image/png")


def test_render_csv_no_numeric():
    r = client.request(
        "POST",
        "/render-csv",
        content=b"a,b\nx,y\n",
        headers={"content-type": "text/csv"},
    )
    assert r.status_code == 400
