"""Command-line interface for the Vyapaar Python SDK.

Subcommands:
  version   print SDK + schema versions
  backtest  stub backtest runner (accepts ``--bars-json PATH``, exit 0)
  stub/serve  placeholders (not implemented yet)
"""
import argparse
import json
import sys

from vyapaar import _vyapaar


def _cmd_version(args) -> int:
    print(
        f"vyapaar {_vyapaar.__version__} "
        f"(API {_vyapaar.API_VERSION}, schema {_vyapaar.SCHEMA_VERSION})"
    )
    return 0


def _cmd_backtest(args) -> int:
    try:
        with open(args.bars_json, encoding="utf-8") as fh:
            data = json.load(fh)
    except (OSError, ValueError) as exc:
        print(f"backtest: could not load {args.bars_json}: {exc}", file=sys.stderr)
        return 2
    n_bars = len(data) if isinstance(data, list) else "?"
    print(
        f"backtest stub: {n_bars} bars loaded from {args.bars_json} "
        "(execution is a placeholder; use vyapaar.run() in Python)"
    )
    return 0


def _cmd_not_implemented(args) -> int:
    print(f"'{args.command}' is not implemented yet")
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(prog="vyapaar", description="Vyapaar Python SDK CLI")
    sub = parser.add_subparsers(dest="command", required=True)

    p_version = sub.add_parser("version", help="print the SDK version and schema version")
    p_version.set_defaults(func=_cmd_version)

    p_backtest = sub.add_parser("backtest", help="run a backtest from a bars JSON file (stub)")
    p_backtest.add_argument(
        "--bars-json", required=True, metavar="PATH", help="path to a JSON bars file"
    )
    p_backtest.set_defaults(func=_cmd_backtest)

    for name in ("stub", "serve"):
        p = sub.add_parser(name, help=f"'{name}' is not implemented yet")
        p.set_defaults(func=_cmd_not_implemented)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
