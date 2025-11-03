use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use timeline_core::TimelineConfig;
use timeline_fhir::summarize_bundle_str;

#[derive(Parser, Debug)]
#[command(
    name = "timeline-cli",
    about = "Tạo tóm tắt timeline từ bundle FHIR JSON."
)]
struct Args {
    /// Đường dẫn tới file JSON bundle.
    #[arg(short, long)]
    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let data = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Không đọc được file {:?}", args.input))?;

    let config = TimelineConfig::default();
    let snapshot = summarize_bundle_str(&data, &config)?;

    println!(
        "Generated at: {}\nCritical alerts: {}\nTimeline events: {}",
        snapshot.generated_at,
        snapshot.critical.alerts.len(),
        snapshot.events.len()
    );

    Ok(())
}
