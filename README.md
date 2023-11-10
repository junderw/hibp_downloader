# hibp_downloader

This project is a CLI tool written in Rust to download and/or sort the HIBP password hashes from Cloudflare.

## Download (trust GitHub to compile it for you)

For those who don't want to install Rust or compile themselves, Github CI automatically builds the binaries and
checksum files and uploads them under each release in the releases section.
([You can view the releases here.](https://github.com/junderw/hibp_downloader/releases))

## Installation (compile it yourself)

This requires Rust to install. [Installing `rustup` is the easiest way.](https://www.rust-lang.org/tools/install)

MSRV is fairly recent, PRs are welcome to lower MSRV if that's important to you.

```
# To install to PATH
$ cargo install --release
# To build in ./target/release/hibp_downloader(.exe)
$ cargo build --release
```

## Usage

Note: `--workers` should probably stay default, as it decides the number of worker threads for tokio's
async runtime, but if you start getting "refused stream before processing any application logic" errors,
lower the `--multiplier` option from its default.

`workers * multiplier` is how it decides the number of concurrent downloads.

Without subcommand:

```
A CLI app for downloading and/or sorting HaveIBeenPwned password hashes.

Usage: hibp_downloader [OPTIONS] [COMMAND]

Commands:
  sort  Sort the downloaded password hashes in descending frequency order
  help  Print this message or the help of the given subcommand(s)

Options:
      --workers <WORKERS>          The number of workers to use for requests [default: NUM_CPU]
      --multiplier <MULTIPLIER>    The number of requests per worker [default: 128/NUM_CPU]
  -n, --ntlm                       Download NTLM hashes instead of SHA1 hashes
      --output-path <OUTPUT_PATH>  The file or folder where the output will be written.
                                   Defaults to a single file that writes all hashes to one file.
                                   If an existing directory is chosen, it will save the downloaded data
                                   as-is to files name ${THISVAR}/00000 to ${THISVAR}/FFFFF.
                                   This means each row in each file will be missing the first 5 characters.
                                   When using a directory, it must be empty. [default: ./hibp_password_hashes.txt]
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