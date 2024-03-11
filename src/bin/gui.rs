use std::path::PathBuf;

use eframe::egui::{self, Widget};
use egui::widgets::ProgressBar;
use hibp_downloader::{
    config::{get_config, Commands, Config},
    consts::LENGTH,
};

struct App {
    workers: String,
    multiplier: String,
    output_path: String,
    ntlm: bool,
    sort: bool,
    input_path: String,
    temp_dir: String,
    running: bool,
}

impl From<&Config> for App {
    fn from(value: &Config) -> Self {
        let output_path = match &value.subcommands {
            Some(Commands::Sort { output_file, .. }) => output_file.to_string_lossy().into_owned(),
            None => value.output_path.to_string_lossy().into_owned(),
        };
        Self {
            workers: value.workers.to_string(),
            multiplier: value.multiplier.to_string(),
            output_path,
            ntlm: value.ntlm,
            sort: matches!(value.subcommands, Some(Commands::Sort { .. })),
            input_path: value.subcommands.as_ref().map_or(
                format!("./hibp_password_hashes.txt"),
                |Commands::Sort { input_file, .. }| input_file.to_string_lossy().into_owned(),
            ),
            temp_dir: value.subcommands.as_ref().map_or(
                format!("./tmp_scratch_disk_for_hibp_sort"),
                |Commands::Sort { temp_dir, .. }| temp_dir.to_string_lossy().into_owned(),
            ),
            running: false,
        }
    }
}

impl App {
    fn name() -> &'static str {
        "HaveIBeenPwned Downloader"
    }

    fn get_clones_sort(&self) -> (PathBuf, PathBuf, PathBuf) {
        (
            PathBuf::from(self.input_path.clone()),
            PathBuf::from(self.output_path.clone()),
            PathBuf::from(self.temp_dir.clone()),
        )
    }

    fn get_clones_dl(&self) -> (usize, usize, bool, PathBuf) {
        (
            self.workers.parse().unwrap(),
            self.multiplier.parse().unwrap(),
            self.ntlm,
            PathBuf::from(self.output_path.clone()),
        )
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workers");
            ui.text_edit_singleline(&mut self.workers);
            ui.heading("Multiplyer");
            ui.text_edit_singleline(&mut self.multiplier);
            ui.heading("Output Path");
            ui.text_edit_singleline(&mut self.output_path);
            ui.heading("Get NTLM");
            if ui.radio(self.ntlm, "").clicked() {
                self.ntlm = !self.ntlm;
            }
            ui.heading("Do sort instead of download");
            if ui.radio(self.sort, "").clicked() {
                self.sort = !self.sort;
            }
            if self.sort {
                ui.heading("Input Path (sort)");
                ui.text_edit_singleline(&mut self.input_path);
                ui.heading("Temp Dir (sort)");
                ui.text_edit_singleline(&mut self.temp_dir);
            }

            if ui.button("Quit").clicked() {
                std::process::exit(0);
            };

            if ui.button("Run").clicked() {
                if self.running {
                    return;
                }
                if self.sort {
                    let data = self.get_clones_sort();
                    self.running = true;
                    std::thread::spawn(move || {
                        hibp_downloader::run_sort(&data.0, &data.1, &data.2).unwrap();
                    });
                } else {
                    let data = self.get_clones_dl();
                    self.running = true;
                    std::thread::spawn(move || {
                        hibp_downloader::run_download(data.0, data.1, data.2, &data.3).unwrap();
                    });
                }
            };

            if self.running {
                let progress = if self.sort {
                    let sort_count = hibp_downloader::stats::PROGRESS_OF_SORT
                        .load(std::sync::atomic::Ordering::Acquire);
                    let sort_length = hibp_downloader::stats::LENGTH_OF_SORT
                        .load(std::sync::atomic::Ordering::Acquire);
                    sort_count as f32 / sort_length as f32
                } else {
                    let dl_count = hibp_downloader::stats::DOWNLOADED
                        .load(std::sync::atomic::Ordering::Acquire);
                    dl_count as f32 / LENGTH as f32
                };
                ProgressBar::new(progress).animate(true).ui(ui);
                ui.label(format!("{:.2}%", (progress * 100.0)));
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((400.0, 600.0)),
        ..eframe::NativeOptions::default()
    };

    let config = get_config();
    let app = App::from(config);
    eframe::run_native(App::name(), native_options, Box::new(|_| Box::new(app)))
}
