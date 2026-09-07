"""Create a Parquet archive with short Bash.org quotes and integer scores.

Run from this directory (or provide explicit paths):
    python filter_bash_archive.py
"""

from pathlib import Path
import argparse

import pyarrow as pa
import pyarrow.parquet as pq


DEFAULT_INPUT = Path(__file__).with_name("bash-org-archive.com.parquet")
DEFAULT_OUTPUT = Path(__file__).with_name(
    "bash-org-archive.com.max-5-lines.parquet"
)


def has_at_most_five_lines(value: object) -> bool:
    """Return whether a quote is text containing no more than five lines."""
    return isinstance(value, str) and len(value.splitlines()) <= 5


def parse_score(value: object) -> int | None:
    """Convert a nullable string score to an integer."""
    if value is None:
        return None
    return int(value)


def filter_archive(input_path: Path, output_path: Path) -> int:
    """Filter the archive in batches and return the number of retained quotes."""
    source = pq.ParquetFile(input_path)
    if "quote" not in source.schema_arrow.names:
        raise ValueError("The input Parquet file does not have a 'quote' column.")
    if "score" not in source.schema_arrow.names:
        raise ValueError("The input Parquet file does not have a 'score' column.")

    score_index = source.schema_arrow.get_field_index("score")
    score_field = source.schema_arrow.field(score_index).with_type(pa.int64())
    output_schema = source.schema_arrow.set(score_index, score_field)

    retained = 0
    writer = None
    try:
        for batch in source.iter_batches(batch_size=65_536):
            quote_index = batch.schema.get_field_index("quote")
            score_values = pa.array(
                [parse_score(value.as_py()) for value in batch.column(score_index)],
                type=pa.int64(),
            )
            batch = batch.set_column(score_index, score_field, score_values)
            keep = [
                has_at_most_five_lines(value.as_py())
                for value in batch.column(quote_index)
            ]
            filtered = pa.Table.from_batches([batch]).filter(pa.array(keep))

            if writer is None:
                writer = pq.ParquetWriter(output_path, output_schema)
            writer.write_table(filtered)
            retained += filtered.num_rows
    finally:
        if writer is not None:
            writer.close()

    return retained


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("output", nargs="?", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    count = filter_archive(args.input, args.output)
    print(f"Wrote {count:,} quotes to {args.output}")


if __name__ == "__main__":
    main()
