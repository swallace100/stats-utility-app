# Python Plotting Microservice

## Responsibilities

- FastAPI endpoints
- Render matplotlib charts from JSON spec
- Cache images by SHA256

## Example Node → Python Spec

```json
{
  "chartId": "c_789",
  "type": "violin_with_box",
  "title": "Height by Group",
  "data": {
    "series": [
      { "name": "A", "values": [170, 172, 169] },
      { "name": "B", "values": [163, 165, 168] }
    ]
  },
  "enc": { "y": "values", "x": "name" },
  "style": { "width": 900, "height": 600, "dpi": 144, "font": "DejaVu Sans" }
}
```

## Example Response

```json
{
  "chartId": "c_789",
  "url": "http://plots_py:7000/render/c_789.png",
  "sha256": "ae4f...c2",
  "format": "png"
}
```

## Commands

```bash
curl -fsS http://127.0.0.1:7000/health
curl -fsS -X POST http://127.0.0.1:7000/render -H "content-type: application/json" -d "[1,2,3,4]" --output out.png
```
