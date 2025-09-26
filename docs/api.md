# API

## API Surface (Sketch)

- `POST /upload` → returns datasetId. (Streams to disk; infer schema.)
- `POST /jobs` → `{ datasetId, analysis: "ttest_two_sample", params: {...} }` → returns jobId.
- `GET /jobs/:jobId/stream` → SSE for progress.
- `GET /results/:jobId` → JSON: stats, tables, pretty text, csv/markdown exports.

### Example Data Contracts

```json
{
  "jobId": "j_123",
  "datasetId": "d_abc",
  "analysis": "ttest_two_sample",
  "inputs": { "x": "height_cm", "y": "group" },
  "result": {
    "t": 2.153,
    "df": 38.7,
    "p": 0.0371,
    "ci": [0.8, 12.4],
    "meanX": 172.4,
    "meanY": 166.1,
    "cohenD": 0.68,
    "assumptions": { "welch": true }
  },
  "meta": { "nX": 21, "nY": 19, "missing": 2, "seed": 0 }
}
```
