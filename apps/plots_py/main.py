from fastapi import FastAPI
app = FastAPI()

@app.get("/health")
def health():
    return {"ok": True}

# add your other routes here…
if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8000)
