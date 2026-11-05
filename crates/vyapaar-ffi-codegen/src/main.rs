//! CLI binary for `vyapaar-ffi-codegen` (`vyapaar-codegen`).
//!
//! Generates the JSON Schema, OpenAPI 3.1, and Python type-stub artifacts from
//! the `vyapaar-core` wire types and writes them to disk, or prints a single
//! artifact to stdout. A thin wrapper around the library; argument parsing is
//! done here without any external dependencies.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use vyapaar_ffi_codegen::{domain_schema, openapi_document, pyi_stubs, write_all};

const USAGE: &str = "\
vyapaar-codegen — generate vyapaar-core wire schema artifacts

USAGE:
    vyapaar-codegen [OPTIONS]

OPTIONS:
    --schema-out DIR   Directory for domain_schema.json and openapi.json
                       [default: schema]
    --pyi-out DIR      Directory for _vyapaar.pyi
                       [default: python/vyapaar]
    --print-schema     Print the JSON Schema document to stdout and exit
    --print-openapi    Print the OpenAPI 3.1 document to stdout and exit
    --print-pyi        Print the Python type stubs to stdout and exit
    --help             Print this help text and exit

Without a --print-* flag, all three artifacts are written to disk and each
written path is printed to stdout.
";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let cli = match parse_args(&args) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match cli.print {
        Some(PrintTarget::Schema) => {
            println!("{}", pretty_json(&domain_schema()));
            ExitCode::SUCCESS
        }
        Some(PrintTarget::Openapi) => {
            println!("{}", pretty_json(&openapi_document()));
            ExitCode::SUCCESS
        }
        Some(PrintTarget::Pyi) => {
            print!("{}", pyi_stubs());
            ExitCode::SUCCESS
        }
        None => write_artifacts(&cli.schema_out, &cli.pyi_out),
    }
}

/// Print the target artifact to stdout and exit successfully.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrintTarget {
    Schema,
    Openapi,
    Pyi,
}

struct Cli {
    schema_out: PathBuf,
    pyi_out: PathBuf,
    print: Option<PrintTarget>,
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut schema_out = PathBuf::from("schema");
    let mut pyi_out = PathBuf::from("python/vyapaar");
    let mut print = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--schema-out" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--schema-out requires a directory argument".to_owned())?;
                schema_out = PathBuf::from(value);
            }
            "--pyi-out" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--pyi-out requires a directory argument".to_owned())?;
                pyi_out = PathBuf::from(value);
            }
            "--print-schema" => set_print(&mut print, PrintTarget::Schema)?,
            "--print-openapi" => set_print(&mut print, PrintTarget::Openapi)?,
            "--print-pyi" => set_print(&mut print, PrintTarget::Pyi)?,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(Cli {
        schema_out,
        pyi_out,
        print,
    })
}

fn set_print(print: &mut Option<PrintTarget>, target: PrintTarget) -> Result<(), String> {
    if print.is_some() {
        return Err(
            "only one of --print-schema, --print-openapi, --print-pyi may be given".to_owned(),
        );
    }
    *print = Some(target);
    Ok(())
}

use std::path::Path;

fn write_artifacts(schema_out: &Path, pyi_out: &Path) -> ExitCode {
    match write_all(schema_out, pyi_out) {
        Ok(paths) => {
            for path in paths {
                println!("{}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: failed to write artifacts: {error}");
            ExitCode::FAILURE
        }
    }
}

fn pretty_json(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).expect("serializing a Value cannot fail");
    text.push('\n');
    text
}
