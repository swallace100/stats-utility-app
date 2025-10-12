use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde::Deserialize;
use std::sync::Arc;
use tower::ServiceExt; // for .oneshot()

use stats_rs::{build_app, state::AppState}; // <-- import from the lib crate

#[derive(Deserialize)]
struct DescribeOut {
    count: usize,
    mean: f64,
    median: f64,
    std_dev: f64,
}

fn make_app() -> axum::Router {
    build_app(Arc::new(AppState::default()))
}

#[tokio::test]
async fn health_ok() {
    let app = make_app();

    let res = app
        .oneshot(Request::get("/api/v1/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn describe_json_ok() {
    let app = make_app();

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
    assert!((out.std_dev - 1.290_994_448_735_805_6).abs() < 1e-12); // sample SD
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
