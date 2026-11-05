//! Integration tests for `vyapaar-ffi-codegen`.
//!
//! These tests pin the public contract of the generated artifacts: the JSON
//! Schema document, the OpenAPI 3.1 document, the Python type stubs, and the
//! `write_all` file-writing entry point.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use vyapaar_ffi_codegen::{domain_schema, openapi_document, pyi_stubs, write_all};

#[test]
fn domain_schema_has_definitions() {
    let schema = domain_schema();
    let obj = schema.as_object().expect("domain_schema must be an object");
    assert_eq!(
        obj["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(obj["version"], 8);

    let definitions = obj["definitions"]
        .as_object()
        .expect("definitions must be an object");
    for key in [
        "OrderRequest",
        "Bar",
        "Tick",
        "Decision",
        "FillReport",
        "Price",
        "Quantity",
    ] {
        assert!(
            definitions.contains_key(key),
            "definitions is missing {key}"
        );
    }
}

#[test]
fn openapi_is_310() {
    let doc = openapi_document();
    assert_eq!(doc["openapi"], "3.1.0");
    assert_eq!(doc["info"]["version"], "v1");

    let schemas = doc["components"]["schemas"]
        .as_object()
        .expect("components.schemas must be an object");
    assert!(
        schemas.contains_key("OrderRequest"),
        "components.schemas.OrderRequest must exist"
    );

    let paths = doc["paths"].as_object().expect("paths must be an object");
    let orders = paths
        .get("/orders")
        .and_then(|v| v.as_object())
        .expect("paths./orders must be an object");
    assert!(
        orders.get("post").is_some(),
        "paths./orders.post must exist"
    );
}

#[test]
fn pyi_matches_contract() {
    let stubs = pyi_stubs();
    for needle in [
        "class Price:",
        "class RiskEngine:",
        "class StrategyRunner:",
        "def bars_to_stream",
        "class OrderRequest:",
        "class Decision:",
    ] {
        assert!(stubs.contains(needle), "pyi stubs are missing: {needle}");
    }
}

#[test]
fn write_all_writes_files() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vyapaar-codegen-test-{}-{nanos}",
        std::process::id()
    ));
    let schema_dir = root.join("schema");
    let pyi_dir = root.join("pyi");

    let paths = write_all(&schema_dir, &pyi_dir).expect("write_all must succeed");
    assert_eq!(paths.len(), 3, "expected exactly 3 written paths");

    for path in &paths {
        let contents = fs::read_to_string(path).unwrap_or_else(|err| {
            panic!("written path {} must be readable: {err}", path.display())
        });
        assert!(
            !contents.is_empty(),
            "written path {} must be non-empty",
            path.display()
        );
    }

    fs::remove_dir_all(&root).expect("temp dir cleanup must succeed");
}
