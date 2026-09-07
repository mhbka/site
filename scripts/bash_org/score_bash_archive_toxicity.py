"""Add local Detoxify toxicity scores to the short Bash.org archive.

Run from this directory:
    uv run --no-project --with pyarrow --with detoxify score_bash_archive_toxicity.py
"""

from pathlib import Path
import argparse

import pyarrow as pa
import pyarrow.parquet as pq
from detoxify import Detoxify


DEFAULT_INPUT = Path(__file__).with_name("bash-org-archive.com.max-5-lines.parquet")
DEFAULT_OUTPUT = Path(__file__).with_name(
    "bash-org-archive.com.max-5-lines.scored.parquet"
)
SCORE_COLUMNS = (
    "toxicity",
    "severe_toxicity",
    "obscene",
    "threat",
    "insult",
    "identity_attack",
    "sexual_explicit",
)


def score_archive(input_path: Path, output_path: Path) -> int:
    """Copy the archive and append Detoxify's score columns."""
    source = pq.ParquetFile(input_path)
    if "quote" not in source.schema_arrow.names:
        raise ValueError("The input Parquet file does not have a 'quote' column.")

    existing_columns = set(source.schema_arrow.names)
    conflicting_columns = existing_columns.intersection(SCORE_COLUMNS)
    if conflicting_columns:
        names = ", ".join(sorted(conflicting_columns))
        raise ValueError(f"The input already contains score columns: {names}")

    model = Detoxify("unbiased", device="cpu")
    written = 0
    writer = None
    try:
        for batch in source.iter_batches(batch_size=256):
            quote_index = batch.schema.get_field_index("quote")
            quotes = [value.as_py() for value in batch.column(quote_index)]
            if not all(isinstance(quote, str) for quote in quotes):
                raise ValueError("All values in the 'quote' column must be text.")

            predictions = model.predict(quotes)
            table = pa.Table.from_batches([batch])
            for name in SCORE_COLUMNS:
                if name not in predictions:
                    raise ValueError(f"The Detoxify model did not return '{name}'.")
                table = table.append_column(
                    name,
                    pa.array(predictions[name], type=pa.float32()),
                )

            if writer is None:
                writer = pq.ParquetWriter(output_path, table.schema)
            writer.write_table(table)
            written += table.num_rows
    finally:
        if writer is not None:
            writer.close()

    return written


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("output", nargs="?", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    count = score_archive(args.input, args.output)
    print(f"Wrote {count:,} scored quotes to {args.output}")


if __name__ == "__main__":
    main()
