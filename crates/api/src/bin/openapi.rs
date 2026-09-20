//! Emit the OpenAPI document to stdout. CI diffs this against the committed
//! spec and fails on drift.

use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    let spec = serde_json::to_string_pretty(&mediadive_api::ApiDoc::openapi())?;
    println!("{spec}");
    Ok(())
}
