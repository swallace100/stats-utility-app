//! Minimal Swagger UI page (via CDN) that loads `/openapi.json`.

use axum::response::Html;

/// Minimal Swagger UI shell served from CDN.
///
/// Loads `/openapi.json` from this service and renders it in Swagger UI.
/// Intended for quick local inspection; for production, consider bundling
/// a pinned UI asset and CSP.
pub async fn swagger_ui() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>stats_rs – API Docs</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({
        url: '/openapi.json',
        dom_id: '#swagger-ui',
        presets: [SwaggerUIBundle.presets.apis],
        layout: 'BaseLayout'
      });
    </script>
  </body>
</html>
"#,
    )
}

/// Convenience alias used by `lib.rs` for `/docs`.
pub async fn docs_ui() -> Html<&'static str> {
    swagger_ui().await
}
