//! Deterministic code generator for the `vyapaar-core` wire contract.
//!
//! Produces three artifacts from the canonical [`vyapaar_core`] wire types:
//!
//! 1. a combined JSON Schema document ([`domain_schema`]),
//! 2. an OpenAPI 3.1 document ([`openapi_document`]), and
//! 3. Python type stubs for the `vyapaar._vyapaar` extension module
//!    ([`pyi_stubs`]).
//!
//! All artifacts are generated deterministically: identical inputs always
//! produce identical output, so the artifacts can be committed and diffed.
//! The pure generators (`domain_schema`, `openapi_document`, `pyi_stubs`)
//! perform no I/O at all. The single sanctioned I/O entry point is
//! [`write_all`], which the `vyapaar-codegen` CLI binary (`main.rs`) drives.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use vyapaar_core as core;

/// Schema version of the generated JSON Schema document, mirroring the wire
/// contract version owned by `vyapaar_core`.
pub const SCHEMA_VERSION: u32 = core::SCHEMA_VERSION;

/// JSON Schema dialect the generated schema document conforms to.
const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Build the `definitions` map for the domain schema: one entry per canonical
/// `vyapaar-core` wire type, keyed by its Rust type name.
fn definitions() -> Value {
    let mut defs = Map::new();
    defs.insert(
        "OrderRequest".to_owned(),
        schema_for_type::<core::OrderRequest>(),
    );
    defs.insert("OrderAck".to_owned(), schema_for_type::<core::OrderAck>());
    defs.insert(
        "FillReport".to_owned(),
        schema_for_type::<core::FillReport>(),
    );
    defs.insert(
        "OrderEvent".to_owned(),
        schema_for_type::<core::OrderEvent>(),
    );
    defs.insert(
        "CancelRequest".to_owned(),
        schema_for_type::<core::CancelRequest>(),
    );
    defs.insert(
        "ModifyRequest".to_owned(),
        schema_for_type::<core::ModifyRequest>(),
    );
    defs.insert(
        "FeeBreakdown".to_owned(),
        schema_for_type::<core::FeeBreakdown>(),
    );
    defs.insert("Bar".to_owned(), schema_for_type::<core::Bar>());
    defs.insert("Tick".to_owned(), schema_for_type::<core::Tick>());
    defs.insert(
        "DepthLevel".to_owned(),
        schema_for_type::<core::DepthLevel>(),
    );
    defs.insert("Decision".to_owned(), schema_for_type::<core::Decision>());
    defs.insert("PriceBand".to_owned(), schema_for_type::<core::PriceBand>());
    defs.insert(
        "Instrument".to_owned(),
        schema_for_type::<core::Instrument>(),
    );
    defs.insert(
        "InstrumentId".to_owned(),
        schema_for_type::<core::InstrumentId>(),
    );
    defs.insert("Price".to_owned(), schema_for_type::<core::Price>());
    defs.insert("Quantity".to_owned(), schema_for_type::<core::Quantity>());
    defs.insert("Money".to_owned(), schema_for_type::<core::Money>());
    defs.insert(
        "PositionSizingRequest".to_owned(),
        schema_for_type::<core::PositionSizingRequest>(),
    );
    defs.insert(
        "PositionSizingResult".to_owned(),
        schema_for_type::<core::PositionSizingResult>(),
    );
    defs.insert(
        "PortfolioSnapshot".to_owned(),
        schema_for_type::<core::PortfolioSnapshot>(),
    );
    defs.insert(
        "DomainEvent".to_owned(),
        schema_for_type::<core::DomainEvent>(),
    );
    Value::Object(defs)
}

/// Serialize the schemars root schema of `T` to a `serde_json::Value`.
///
/// Serializing a `RootSchema` cannot fail, so the `unwrap` is infallible.
fn schema_for_type<T: schemars::JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("serializing a RootSchema cannot fail")
}

/// Combined JSON Schema document for the `vyapaar-core` wire contract.
pub fn domain_schema() -> Value {
    json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "title": "vyapaar-core wire schema",
        "description": "Generated deterministically by vyapaar-ffi-codegen; do not edit by hand.",
        "version": core::SCHEMA_VERSION as u64,
        "definitions": definitions(),
    })
}

/// OpenAPI 3.1 document for the headless `vyapaar` runtime REST API.
///
/// The `components.schemas` object is cloned from the domain schema's
/// `definitions` so the two documents can never drift apart.
pub fn openapi_document() -> Value {
    let schemas = domain_schema()
        .get("definitions")
        .cloned()
        .expect("domain_schema always contains definitions");

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "vyapaar runtime API",
            "version": core::API_VERSION,
            "description": "Generated deterministically by vyapaar-ffi-codegen; do not edit by hand.",
        },
        "paths": {
            "/health": {
                "get": {
                    "operationId": "health",
                    "summary": "System health",
                    "responses": { "200": { "description": "OK" } },
                }
            },
            "/version": {
                "get": {
                    "operationId": "version",
                    "summary": "Runtime and API version",
                    "responses": { "200": { "description": "OK" } },
                }
            },
            "/orders": {
                "post": {
                    "operationId": "placeOrder",
                    "summary": "Place a validated order",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/OrderRequest" },
                            }
                        },
                    },
                    "responses": {
                        "200": { "description": "OrderAck" },
                        "400": { "description": "Risk refused" },
                    },
                }
            },
            "/positions": {
                "get": {
                    "operationId": "positions",
                    "summary": "Current open positions",
                    "responses": { "200": { "description": "OK" } },
                }
            },
            "/portfolio": {
                "get": {
                    "operationId": "portfolio",
                    "summary": "Balance and holdings",
                    "responses": { "200": { "description": "OK" } },
                }
            },
            "/risk": {
                "get": {
                    "operationId": "risk",
                    "summary": "Live risk gate state",
                    "responses": { "200": { "description": "OK" } },
                }
            },
            "/market-data/quote/{id}": {
                "get": {
                    "operationId": "quote",
                    "summary": "Latest quote",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" },
                        }
                    ],
                    "responses": { "200": { "description": "OK" } },
                }
            },
        },
        "components": { "schemas": schemas },
        "tags": [
            { "name": "health" },
            { "name": "orders" },
            { "name": "positions" },
            { "name": "portfolio" },
            { "name": "risk" },
            { "name": "market-data" },
        ],
    })
}

/// Python type stubs for the `vyapaar._vyapaar` extension module.
///
/// This string is a contract: the Python SDK and the integration tests depend
/// on its exact contents. Do not reformat it.
pub fn pyi_stubs() -> String {
    r###"# Generated deterministically by vyapaar-ffi-codegen. Do not edit by hand.
# Type stubs for the `vyapaar._vyapaar` extension module (Tier 4, PyO3 ABI3).
from decimal import Decimal
from typing import Optional

__version__: str
SCHEMA_VERSION: int
API_VERSION: str

class RiskRefused(Exception): ...
class ConversionError(Exception): ...

class Price:
    def __init__(self, value: Decimal | str) -> None: ...
    @property
    def value(self) -> Decimal: ...
    def to_str(self) -> str: ...
    def is_positive(self) -> bool: ...
    def is_tick_aligned(self, tick: Decimal | str) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class Quantity:
    def __init__(self, value: Decimal | str) -> None: ...
    @property
    def value(self) -> Decimal: ...
    def to_str(self) -> str: ...
    def is_positive(self) -> bool: ...
    def is_lot_multiple(self, lot_size: int) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class Currency:
    def __init__(self, code: str) -> None: ...
    @property
    def code(self) -> str: ...
    @staticmethod
    def from_code(code: str) -> Currency: ...
    def __str__(self) -> str: ...

class Exchange:
    def __init__(self, code: str) -> None: ...
    @property
    def code(self) -> str: ...
    @property
    def name(self) -> str: ...
    @staticmethod
    def from_code(code: str) -> Exchange: ...
    def __str__(self) -> str: ...

class InstrumentId:
    def __init__(self, exchange: str, symbol: str, token: Optional[str] = None, isin: Optional[str] = None) -> None: ...
    @property
    def exchange(self) -> str: ...
    @property
    def symbol(self) -> str: ...
    @property
    def token(self) -> Optional[str]: ...
    @property
    def isin(self) -> Optional[str]: ...
    def to_str(self) -> str: ...
    @staticmethod
    def from_str(s: str) -> InstrumentId: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class OrderSide:
    def __init__(self, code: str) -> None: ...
    @property
    def code(self) -> str: ...
    def opposite(self) -> OrderSide: ...
    def __str__(self) -> str: ...

class TimeInForce:
    def __init__(self, code: str) -> None: ...
    @property
    def code(self) -> str: ...
    def __str__(self) -> str: ...

class OrderStatus:
    def __init__(self, code: str) -> None: ...
    @property
    def code(self) -> str: ...
    @staticmethod
    def from_code(code: str) -> OrderStatus: ...
    def __str__(self) -> str: ...

class DepthLevel:
    def __init__(self, price: Decimal | str, quantity: Decimal | str, orders_count: Optional[int] = None) -> None: ...
    @property
    def price(self) -> Decimal: ...
    @property
    def quantity(self) -> Decimal: ...
    @property
    def orders_count(self) -> Optional[int]: ...

class Bar:
    def __init__(self, instrument: InstrumentId, open: Decimal | str, high: Decimal | str, low: Decimal | str, close: Decimal | str, volume: Decimal | str, timestamp: str) -> None: ...
    @property
    def instrument(self) -> InstrumentId: ...
    @property
    def open(self) -> Decimal: ...
    @property
    def high(self) -> Decimal: ...
    @property
    def low(self) -> Decimal: ...
    @property
    def close(self) -> Decimal: ...
    @property
    def volume(self) -> Decimal: ...
    @property
    def timestamp(self) -> str: ...
    def to_dict(self) -> dict: ...

class Tick:
    def __init__(self, kind: str) -> None: ...
    @property
    def kind(self) -> str: ...
    @staticmethod
    def ltp(instrument: InstrumentId, price: Decimal | str, volume: Decimal | str, exchange_ts: str, receive_ts: str) -> Tick: ...
    @staticmethod
    def quote(instrument: InstrumentId, bid: Decimal | str, ask: Decimal | str, bid_qty: Decimal | str, ask_qty: Decimal | str, last_price: Optional[Decimal | str] = None, exchange_ts: str = "", receive_ts: str = "") -> Tick: ...
    def to_dict(self) -> dict: ...

class OrderRequest:
    def __init__(self, instrument: InstrumentId, side: OrderSide, order_type: str, quantity: Decimal | str, algo_id: str, *, client_order_id: Optional[str] = None, limit_price: Optional[Decimal | str] = None, trigger_price: Optional[Decimal | str] = None, tif: str = "DAY", tag: Optional[str] = None) -> None: ...
    @property
    def client_order_id(self) -> str: ...
    @property
    def instrument(self) -> InstrumentId: ...
    @property
    def side(self) -> OrderSide: ...
    @property
    def order_type(self) -> str: ...
    @property
    def quantity(self) -> Decimal: ...
    @property
    def limit_price(self) -> Optional[Decimal]: ...
    @property
    def tif(self) -> str: ...
    @property
    def algo_id(self) -> str: ...
    @property
    def tag(self) -> Optional[str]: ...
    def to_dict(self) -> dict: ...

class OrderAck:
    def __init__(self, order_id: str, client_order_id: str, status: OrderStatus, timestamp: str) -> None: ...
    @property
    def order_id(self) -> str: ...
    @property
    def client_order_id(self) -> str: ...
    @property
    def status(self) -> str: ...
    @property
    def timestamp(self) -> str: ...
    def to_dict(self) -> dict: ...

class FillReport:
    def __init__(self, fill_id: str, order_id: str, client_order_id: str, instrument: InstrumentId, side: OrderSide, price: Decimal | str, quantity: Decimal | str, algo_id: str, timestamp: str) -> None: ...
    @property
    def fill_id(self) -> str: ...
    @property
    def order_id(self) -> str: ...
    @property
    def client_order_id(self) -> str: ...
    @property
    def instrument(self) -> InstrumentId: ...
    @property
    def side(self) -> OrderSide: ...
    @property
    def price(self) -> Decimal: ...
    @property
    def quantity(self) -> Decimal: ...
    @property
    def algo_id(self) -> str: ...
    @property
    def timestamp(self) -> str: ...
    def to_dict(self) -> dict: ...

class Decision:
    @staticmethod
    def from_json(s: str) -> Decision: ...
    @property
    def action(self) -> str: ...
    def to_dict(self) -> dict: ...

class RiskEngine:
    def __init__(self) -> None: ...
    def register_algo(self, algo_id: str) -> None: ...
    def is_algo_registered(self, algo_id: str) -> bool: ...
    def suspend_algo(self, algo_id: str, reason: str) -> None: ...
    def reactivate_algo(self, algo_id: str) -> None: ...
    def unregister_algo(self, algo_id: str) -> None: ...
    def halt_global(self, reason: str, ts_iso: str) -> None: ...
    def resume_global(self) -> None: ...
    def set_reduce_only(self) -> None: ...
    def is_halted(self) -> bool: ...
    def global_state(self) -> str: ...
    def set_price_band(self, instrument: InstrumentId, lower: Decimal | str, upper: Decimal | str, reference: Decimal | str) -> None: ...
    def check_order(self, order: OrderRequest, ts_iso: Optional[str] = None) -> None: ...
    def check_order_fast(self, algo_id: str, instrument: InstrumentId, quantity: Decimal | str, limit_price: Optional[Decimal | str] = None, ts_iso: Optional[str] = None) -> None: ...
    def check_decision(self, decision_json: str, ts_iso: Optional[str] = None) -> None: ...
    def record_fill(self, fill: FillReport) -> None: ...
    def otr_for_algo(self, algo_id: str) -> Optional[str]: ...
    def total_refusals(self) -> int: ...
    def turnover_for_algo(self, algo_id: str) -> str: ...

class StrategyRunner:
    def __init__(self, strategy: object, algo_id: str) -> None: ...
    def submit_order(self, order: OrderRequest, ts_iso: Optional[str] = None) -> OrderAck: ...
    def check_order(self, order: OrderRequest, ts_iso: Optional[str] = None) -> None: ...
    def run_bars(self, bars: list[Bar]) -> int: ...
    def run_ticks(self, ticks: list[Tick]) -> int: ...

def now_iso() -> str: ...
def bars_to_stream(bars: list[Bar]) -> object: ...
def ticks_to_stream(ticks: list[Tick]) -> object: ...
def stream_capsule_name() -> str: ...
"###
    .to_owned()
}

/// Write all three generated artifacts to disk.
///
/// `out_dir` receives `domain_schema.json` and `openapi.json`; `pyi_out`
/// receives `_vyapaar.pyi`. Both directories are created (recursively) if
/// missing. Returns the paths of the written files, in a stable order.
pub fn write_all(out_dir: &Path, pyi_out: &Path) -> std::io::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(pyi_out)?;

    let schema_path = out_dir.join("domain_schema.json");
    std::fs::write(&schema_path, pretty_json(&domain_schema()))?;

    let openapi_path = out_dir.join("openapi.json");
    std::fs::write(&openapi_path, pretty_json(&openapi_document()))?;

    let pyi_path = pyi_out.join("_vyapaar.pyi");
    std::fs::write(&pyi_path, pyi_stubs())?;

    Ok(vec![schema_path, openapi_path, pyi_path])
}

/// Serialize a `Value` as pretty JSON with two-space indentation and a
/// trailing newline.
fn pretty_json(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).expect("serializing a Value cannot fail");
    text.push('\n');
    text
}
