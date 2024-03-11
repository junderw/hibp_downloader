use hibp_downloader::{
    config::{get_config, Commands},
    init_logging_and_progress, run_download, run_sort,
};

fn main() -> anyhow::Result<()> {
    init_logging_and_progress();
    let config = get_config();
    match &config.subcommands {
        None => run_download(
            config.workers,
            config.multiplier,
            config.ntlm,
            &config.output_path,
        ),
        Some(Commands::Sort {
            input_file,
            output_file,
            temp_dir,
        }) => run_sort(input_file, output_file, temp_dir),
    }
}
