from fastapi.testclient import TestClient
from main import app

client = TestClient(app)

def test_health():
    r = client.get("/health")
    assert r.status_code == 200
    assert r.json() == {"ok": True}

def test_render_basic():
    r = client.post("/render", json=[1,2,3,4])
    assert r.status_code == 200
    assert r.headers["content-type"].startswith("image/png")

def test_render_csv_no_numeric():
    r = client.post("/render-csv", data="a,b\nx,y\n")
    assert r.status_code == 400
