use clap::Parser;
use screenpipe_vision::{continuous_capture, monitor::get_default_monitor, OcrEngine};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::channel;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Save text files
    #[arg(long, default_value_t = false)]
    save_text_files: bool,

    /// Disable cloud OCR processing
    #[arg(long, default_value_t = false)]
    cloud_ocr_off: bool, // Add this flag

    /// FPS
    #[arg(long, default_value_t = 1.0)]
    fps: f32,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let (result_tx, mut result_rx) = channel(512);

    let save_text_files = cli.save_text_files;

    let monitor = get_default_monitor().await;
    let id = monitor.id();

    tokio::spawn(async move {
        continuous_capture(
            result_tx,
            Duration::from_secs_f32(1.0 / cli.fps),
            save_text_files,
            Arc::new(if cfg!(target_os = "macos") {
                OcrEngine::AppleNative
            } else {
                OcrEngine::Tesseract
            }),
            id,
        )
        .await
    });

    // Example: Process results for 10 seconds, then pause for 5 seconds, then stop
    loop {
        if let Some(result) = result_rx.recv().await {
            println!(
                "OCR Text length across visible windows: {}",
                result
                    .window_ocr_results
                    .iter()
                    .map(|w| w.text.len())
                    .sum::<usize>()
            );
        }

        // tokio::time::sleep(Duration::from_secs_f32(1.0 / cli.fps)).await;
    }
}