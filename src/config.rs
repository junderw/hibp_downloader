use std::{path::PathBuf, sync::OnceLock};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct Config {
    #[command(subcommand)]
    pub subcommands: Option<Commands>,
    /// The number of workers to use for requests
    #[arg(long, default_value_t = default_workers())]
    pub workers: usize,
    /// The number of requests per worker
    #[arg(long, default_value_t = default_multiplier())]
    pub multiplier: usize,
    /// Download NTLM hashes instead of SHA1 hashes
    #[arg(short, long)]
    pub ntlm: bool,
    /// The file where the output will be written.
    /// This file will be sorted by hash.
    #[arg(
        long,
        default_value = "./hibp_password_hashes.txt",
        verbatim_doc_comment
    )]
    pub output_file: PathBuf,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Sort the downloaded password hashes in descending frequency order.
    #[command(name = "sort")]
    Sort {
        /// The file to be sorted.
        #[arg(
            long,
            default_value = "./hibp_password_hashes.txt",
            verbatim_doc_comment
        )]
        input_file: PathBuf,
        /// The file where the frequency sorted output will be written.\n
        /// This file will be sorted by descending frequency.
        /// See the sort subcommand if you want to sort after the fact.
        /// WARNING: This option requires 2x the file size in empty space.
        /// (It writes smaller chunked files and sorts the items from those files)
        #[arg(
            long,
            default_value = "./hibp_password_hashes_sorted.txt",
            verbatim_doc_comment
        )]
        output_file: PathBuf,
        /// This directory will be used to store the temporary files
        /// used in the sorting algorithm. It will hold many files
        /// that add up to the size of the original file and will be
        /// deleted upon completion.
        #[arg(
            long,
            default_value = "./tmp_scratch_disk_for_hibp_sort",
            verbatim_doc_comment
        )]
        temp_dir: PathBuf,
    },
}

fn default_workers() -> usize {
    std::thread::available_parallelism()
        .expect("Couldn't get CPU count")
        .get()
}

fn default_multiplier() -> usize {
    // Cloudflare seems to throttle at 128 in-flight connections at a time
    // (Note: available_parallelism is guaranteed to be non-zero)
    128 / default_workers()
}

/// This function gives a static reference to a Config struct.
pub fn get_config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(Config::parse)
}
