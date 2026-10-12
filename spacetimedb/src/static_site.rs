use spacetimedb::http::{Body, HandlerContext, Request, Response, Router, handler, router};
include!(concat!(env!("OUT_DIR"), "/static_assets.rs"));

#[handler]
fn serve_assets(_ctx: &mut HandlerContext, request: Request) -> Response {
    let uri = request.uri().path();
    let path = uri
        .split_once("/route/")
        .map(|(_, tail)| format!("/{tail}"))
        .unwrap_or_else(|| uri.to_owned());
    let path = if path == "/" { "/index-html" } else { &path };
    match ASSETS.iter().find(|(name, _, _)| *name == path) {
        Some((name, mime, bytes)) => Response::builder()
            .header("Content-Type", *mime)
            .header(
                "Cache-Control",
                if *name == "/index-html" {
                    "no-cache"
                } else {
                    "public, max-age=31536000, immutable"
                },
            )
            .header("X-Content-Type-Options", "nosniff")
            .body(Body::from_bytes(bytes.to_vec()))
            .unwrap(),
        None => Response::builder()
            .status(404)
            .body(Body::from_bytes("Not found"))
            .unwrap(),
    }
}
#[router]
fn frontend_routes() -> Router {
    asset_router()
}
