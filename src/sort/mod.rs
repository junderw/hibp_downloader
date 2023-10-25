use anyhow::Context;
use extsort::*;
use std::{
    io::{BufRead, Read, Write},
    path::Path,
};
use tracing_indicatif::span_ext::IndicatifSpanExt;
mod row;
use row::MyStruct;

use crate::progress_style::{get_span, progress_style_sort};

pub fn run_sort(input: &Path, output: &Path, temp_dir: &Path) -> anyhow::Result<()> {
    // Create the dir if it doesn't exist
    // mkdir -p ${temp_dir}
    std::fs::create_dir_all(temp_dir)?;

    let input_byte_size = std::fs::metadata(input)?.len();
    if input_byte_size < 60 {
        anyhow::bail!("File too small");
    }

    let hash_size = {
        let mut file = std::fs::File::open(input)?;
        // Read first 60 bytes of the file which will definitely hold at least 1 row.
        let mut buf = [0; 60];
        file.read_exact(&mut buf)?;
        let s = String::from_utf8_lossy(&buf);
        let (hash, _) = s
            .split('\n')
            .next()
            .context("No new line in sort file")?
            .split_once(':')
            .context("No colon in sort file")?;
        let len = hash.len();
        // NTLM or SHA1 (in hex string)
        assert!(len == 32 || len == 40);
        len
    };
    // colon + average of 5 length number (max 8, min 1) + new line
    let row_size = hash_size + 7;
    let rows_in_file = input_byte_size / row_size as u64;

    // This is a rough estimate.
    let span = get_span(rows_in_file * 2, progress_style_sort());
    let _enter = span.enter();

    // 11.64 million x 45 bytes per struct = 500.5 MB chunks
    let sorter = ExternalSorter::new()
        .with_parallel_sort()
        .with_sort_dir(temp_dir.to_path_buf())
        .with_segment_size(11_640_000);
    let reader = std::io::BufReader::with_capacity(16 * 1024 * 1024, std::fs::File::open(input)?);
    let mut writer =
        std::io::BufWriter::with_capacity(16 * 1024 * 1024, std::fs::File::create(output)?);
    sorter
        .sort(reader.lines().map(|s| {
            span.pb_inc(1);
            s.unwrap().parse::<MyStruct>().unwrap()
        }))?
        .for_each(|data| {
            span.pb_inc(1);
            writer.write_all(data.hash.as_bytes()).unwrap();
            writer.write_all(b":").unwrap();
            writer.write_all(data.count.to_string().as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        });
    // Remove the temp dir after the writing is finished.
    std::fs::remove_dir_all(temp_dir)?;
    Ok(())
}
