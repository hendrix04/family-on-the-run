use std::{
    env, fs,
    path::{Path, PathBuf},
};
fn files(root: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("Build the frontend first: npm run build") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, output);
        } else {
            output.push(path);
        }
    }
}
fn main() {
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../dist")
        .canonicalize()
        .expect("Build the frontend first: npm run build");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut paths = Vec::new();
    files(&root, &mut paths);
    paths.sort();
    let mut assets = String::from("static ASSETS: &[(&str, &str, &[u8])] = &[\n");
    let mut routes =
        String::from("fn asset_router() -> Router { Router::new().get(\"/\", serve_assets)");
    for path in paths {
        let relative = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let route = format!("/{}", relative.replace('.', "-").to_ascii_lowercase());
        let mime = match path
            .extension()
            .and_then(|e| e.to_str())
            .or_else(|| path.file_name()?.to_str()?.rsplit('-').next())
        {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "text/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            Some("woff2") => "font/woff2",
            _ => "application/octet-stream",
        };
        assets.push_str(&format!(
            "({route:?}, {mime:?}, include_bytes!({:?})),\n",
            path.to_str().unwrap()
        ));
        routes.push_str(&format!(".get({route:?}, serve_assets)"));
    }
    assets.push_str("];\n");
    routes.push_str(" }\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("static_assets.rs"),
        assets + &routes,
    )
    .unwrap();
}
