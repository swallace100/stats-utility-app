use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde::Deserialize;
use std::sync::Arc;
use tower::ServiceExt;

use stats_rs::{build_app, state::AppState};

#[derive(Deserialize)]
struct DescribeOut {
    count: usize,
    mean: f64,
    median: f64,
    std_dev: f64,
}

#[derive(Deserialize)]
struct SummaryOut {
    count: usize,
    mean: Option<f64>,
    median: Option<f64>,
    std: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
}

fn make_app() -> axum::Router {
    build_app(Arc::new(AppState))
}

#[tokio::test]
async fn health_ok() {
    let app = make_app().into_service(); // <-- only change

    let res = app
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn describe_json_ok() {
    let app = make_app().into_service(); // <-- only change

    let res = app
        .oneshot(
            Request::post("/api/v1/describe")
                .header("content-type", "application/json")
                .body(Body::from("[1,2,3,4]"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: DescribeOut = serde_json::from_slice(&body).unwrap();

    assert_eq!(out.count, 4);
    assert!((out.mean - 2.5).abs() < 1e-12);
    assert!((out.median - 2.5).abs() < 1e-12);
    assert!((out.std_dev - 1.290_994_448_735_805_6).abs() < 1e-12);
}

#[tokio::test]
async fn describe_json_empty_is_400() {
    let app = make_app();

    let res = app
        .oneshot(
            Request::post("/api/v1/describe")
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn describe_csv_ok_with_header() {
    let app = make_app();
    let csv = "value\n1\n2\n3\n4\n5\n";

    let res = app
        .oneshot(
            Request::post("/api/v1/describe-csv")
                .header("content-type", "text/csv")
                .body(Body::from(csv))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: DescribeOut = serde_json::from_slice(&body).unwrap();

    assert_eq!(out.count, 5);
    assert!((out.mean - 3.0).abs() < 1e-12);
    assert!((out.median - 3.0).abs() < 1e-12);
    assert!((out.std_dev - 1.581_138_830_084_189_8).abs() < 1e-12); // sample SD
}

#[tokio::test]
async fn describe_csv_mixed_values_ignores_non_numeric() {
    let app = make_app();
    let csv = "a,b,c\nx,1,foo\n2,bar,3\n";

    let res = app
        .oneshot(
            Request::post("/api/v1/describe-csv")
                .header("content-type", "text/csv")
                .body(Body::from(csv))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: DescribeOut = serde_json::from_slice(&body).unwrap();

    // numeric cells found: 1, 2, 3
    assert_eq!(out.count, 3);
    assert!((out.mean - 2.0).abs() < 1e-12);
    assert!((out.median - 2.0).abs() < 1e-12);
}

#[tokio::test]
async fn describe_csv_no_numeric_400() {
    let app = make_app();
    let csv = "a,b\nx,y\nfoo,bar\n";

    let res = app
        .oneshot(
            Request::post("/api/v1/describe-csv")
                .header("content-type", "text/csv")
                .body(Body::from(csv))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn openapi_json_exists() {
    let app = make_app();

    let res = app
        .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["openapi"], "3.0.3");
}

#[tokio::test]
async fn stats_summary_basic() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/summary")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [1,2,3,4,5]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: SummaryOut = serde_json::from_slice(&buf).unwrap();

    assert_eq!(out.count, 5);
    assert!((out.mean.unwrap() - 3.0).abs() < 1e-12);
    assert!((out.median.unwrap() - 3.0).abs() < 1e-12);
    assert!(out.std.unwrap() > 0.0);
    assert_eq!(out.min.unwrap(), 1.0);
    assert_eq!(out.max.unwrap(), 5.0);
}

// ========== distribution ==========
#[derive(Deserialize)]
struct DistOut {
    counts: Vec<usize>,
    edges: Vec<f64>,
    quantiles: Vec<(f64, f64)>,
}

#[tokio::test]
async fn stats_distribution_basic() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/distribution")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [1,2,3,4,5],
                        "bins": 4,
                        "quantiles": [0.25, 0.5, 0.75]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: DistOut = serde_json::from_slice(&buf).unwrap();

    assert_eq!(out.edges.len(), out.counts.len() + 1);
    assert_eq!(out.quantiles.len(), 3);
}

// ========== pairwise ==========
#[derive(Deserialize)]
struct PairOut {
    pearson: Option<f64>,
    spearman: Option<f64>,
}

#[tokio::test]
async fn stats_pairwise_same_series_is_one() {
    let app = make_app().into_service();
    let x = [1.0, 2.0, 3.0, 4.0];

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/pairwise")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "x": x, "y": x
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: PairOut = serde_json::from_slice(&buf).unwrap();

    assert!((out.pearson.unwrap() - 1.0).abs() < 1e-12);
    assert!((out.spearman.unwrap() - 1.0).abs() < 1e-12);
}

// ========== ecdf ==========
#[derive(Deserialize)]
struct EcdfOut {
    xs: Vec<f64>,
    ps: Vec<f64>,
}

#[tokio::test]
async fn stats_ecdf_monotone_and_last_is_one() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/ecdf")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [3,1,2,2,4],
                        "max_points": 100
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: EcdfOut = serde_json::from_slice(&buf).unwrap();

    assert_eq!(out.xs.len(), out.ps.len());
    assert!((out.ps.last().copied().unwrap_or(0.0) - 1.0).abs() < 1e-12);
    assert!(out.ps.windows(2).all(|w| w[0] <= w[1]));
}

// ========== qq-normal ==========
#[derive(Deserialize)]
struct QqOut {
    sample_quantiles: Vec<f64>,
    theoretical_quantiles: Vec<f64>,
    sigma_hat: f64,
}

#[tokio::test]
async fn stats_qq_shapes_match() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/qq-normal")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [1.0, 2.0, 2.1, 2.9, 3.5],
                        "robust": false
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: QqOut = serde_json::from_slice(&buf).unwrap();

    assert_eq!(out.sample_quantiles.len(), out.theoretical_quantiles.len());
    assert!(out.sigma_hat.is_finite());
}

// ========== corr-matrix ==========
#[derive(Deserialize)]
struct CorrMatrixOut {
    size: usize,
    matrix: Vec<f64>,
}

#[tokio::test]
async fn stats_corr_matrix_square_and_diag_one() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/corr-matrix")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "series": [[1,2,3,4], [1,2,3,4]],
                        "names": ["a","b"],
                        "method": "pearson"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: CorrMatrixOut = serde_json::from_slice(&buf).unwrap();

    assert_eq!(out.size, 2);
    assert_eq!(out.matrix.len(), 4);
    assert!((out.matrix[0] - 1.0).abs() < 1e-12);
    assert!((out.matrix[3] - 1.0).abs() < 1e-12);
}

// ========== outliers ==========
#[derive(Deserialize)]
struct OutliersOut {
    values: Vec<f64>,
}

#[tokio::test]
async fn stats_outliers_iqr_finds_extreme() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/outliers")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [1,2,3,4,100],
                        "method": "iqr"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: OutliersOut = serde_json::from_slice(&buf).unwrap();

    assert!(out.values.contains(&100.0));
}

// ========== normalize ==========
#[derive(Deserialize)]
struct NormalizeOut {
    values: Vec<f64>,
}

#[tokio::test]
async fn stats_normalize_minmax_range() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/normalize")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [10, 20],
                        "method": "minmax",
                        "range": [0.0, 1.0]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: NormalizeOut = serde_json::from_slice(&buf).unwrap();

    assert_eq!(out.values[0], 0.0);
    assert_eq!(out.values[1], 1.0);
}

// ========== binrule ==========
#[derive(Deserialize)]
struct BinRuleOut {
    bins: usize,
}

#[tokio::test]
async fn stats_binrule_returns_positive_bins() {
    let app = make_app().into_service();

    let res = app
        .oneshot(
            Request::post("/api/v1/stats/binrule")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "values": [1,2,3,4,5,6,7,8,9,10],
                        "rule": "sturges"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let buf = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let out: BinRuleOut = serde_json::from_slice(&buf).unwrap();

    assert!(out.bins >= 2);
}
