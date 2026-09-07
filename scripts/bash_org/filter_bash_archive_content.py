"""Remove clearly offensive or explicit quotes from the short Bash.org archive.

Run from this directory (or provide explicit paths):
    python filter_bash_archive_content.py
"""

from pathlib import Path
import argparse
import re

import pyarrow as pa
import pyarrow.parquet as pq


DEFAULT_INPUT = Path(__file__).with_name("bash-org-archive.com.max-5-lines.parquet")
DEFAULT_OUTPUT = Path(__file__).with_name(
    "bash-org-archive.com.max-5-lines.clean.parquet"
)

# Intentionally limited to unambiguous slurs and overtly sexual terms. Add terms
# here if the output still contains content you do not want to publish.
BLOCKED_TERMS = (
    # Slurs and dehumanizing language.
    r"n[1i!][gq]{2}[ae3]r",
    r"n[1i!][gq]{2}a",
    r"fag(?:got|s)?",
    r"kike(?:s)?",
    r"chink(?:s)?",
    r"spic(?:s)?",
    r"wetback(?:s)?",
    r"retard(?:ed|s)?",
    # Explicit sexual content.
    r"anal",
    r"blowjob(?:s)?",
    r"cunnilingus",
    r"dick(?:s)?",
    r"ejaculat(?:e|ed|es|ing|ion)",
    r"fellatio",
    r"fingering",
    r"handjob(?:s)?",
    r"masturbat(?:e|ed|es|ing|ion)",
    r"orgasm(?:s|ic)?",
    r"penis(?:es)?",
    r"porn(?:ography|ographic)?",
    r"puss(?:y|ies)",
    r"sex(?:ual(?:ly)?|ing)?",
    r"sperm",
    r"tit(?:s|ties)",
    r"vagina(?:s)?",
)
BLOCKED_PATTERN = re.compile(r"\b(?:" + "|".join(BLOCKED_TERMS) + r")\b", re.IGNORECASE)


def is_acceptable_quote(value: object) -> bool:
    """Return whether a quote is text without blocked terms."""
    return isinstance(value, str) and BLOCKED_PATTERN.search(value) is None


def filter_archive(input_path: Path, output_path: Path) -> tuple[int, int]:
    """Write acceptable rows and return the retained and removed row counts."""
    source = pq.ParquetFile(input_path)
    if "quote" not in source.schema_arrow.names:
        raise ValueError("The input Parquet file does not have a 'quote' column.")

    retained = 0
    removed = 0
    with pq.ParquetWriter(output_path, source.schema_arrow) as writer:
        for batch in source.iter_batches(batch_size=65_536):
            quote_index = batch.schema.get_field_index("quote")
            keep = [
                is_acceptable_quote(value.as_py())
                for value in batch.column(quote_index)
            ]
            filtered = pa.Table.from_batches([batch]).filter(pa.array(keep))
            writer.write_table(filtered)
            retained += filtered.num_rows
            removed += batch.num_rows - filtered.num_rows

    return retained, removed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("output", nargs="?", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    retained, removed = filter_archive(args.input, args.output)
    print(f"Wrote {retained:,} quotes to {args.output} ({removed:,} removed)")


if __name__ == "__main__":
    main()
