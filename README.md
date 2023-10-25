# hibp_downloader

This project is a CLI tool written in Rust to download and/or sort the HIBP password hashes from Cloudflare.

## Installation

This requires Rust to install. [Installing `rustup` is the easiest way.](https://www.rust-lang.org/tools/install)

MSRV is fairly recent, PRs are welcome to lower MSRV if that's important to you.

```
# To install to PATH
$ cargo install --release
# To build in ./target/release/hibp_downloader(.exe)
$ cargo build --release
```

## Usage

Without subcommand:

```
A CLI app for downloading and/or sorting HaveIBeenPwned password hashes.

Usage: hibp_downloader [OPTIONS] [COMMAND]

Commands:
  sort  Sort the downloaded password hashes in descending frequency order
  help  Print this message or the help of the given subcommand(s)

Options:
      --workers <WORKERS>          The number of workers to use for requests [default: 8]
      --multiplier <MULTIPLIER>    The number of requests per worker [default: 20]
  -n, --ntlm                       Download NTLM hashes instead of SHA1 hashes
      --output-file <OUTPUT_FILE>  The file where the output will be written.
                                   This file will be sorted by hash. [default: ./hibp_password_hashes.txt]
  -h, --help                       Print help
  -V, --version                    Print version
```

With subcommand `sort`:

```
Sort the downloaded password hashes in descending frequency order

Usage: hibp_downloader sort [OPTIONS]

Options:
      --input-file <INPUT_FILE>    The file to be sorted. [default: ./hibp_password_hashes.txt]
      --output-file <OUTPUT_FILE>  The file where the frequency sorted output will be written.\n
                                   This file will be sorted by descending frequency.
                                   See the sort subcommand if you want to sort after the fact.
                                   WARNING: This option requires 2x the file size in empty space.
                                   (It writes smaller chunked files and sorts the items from those files) [default: ./hibp_password_hashes_sorted.txt]
      --temp-dir <TEMP_DIR>        This directory will be used to store the temporary files
                                   used in the sorting algorithm. It will hold many files
                                   that add up to the size of the original file and will be
                                   deleted upon completion. [default: ./tmp_scratch_disk_for_hibp_sort]
  -h, --help                       Print help
  -V, --version                    Print version
```