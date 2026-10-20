mod config;
mod engine;
mod input;
mod midi;
mod window;

use std::sync::mpsc;
use std::time::Duration;
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use config::AppConfig;
use engine::ActionEngine;
use midi::{MidiEvent, MidiManager};

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=======================================================");
    println!("  ADaRe-Bridge: Akai MPK Mini -> DaVinci Resolve Bridge ");
    println!("=======================================================");

    let config = AppConfig::load_or_create("config.toml")?;
    info!("Target Application: {}", config.target_process);
    info!("MIDI Device Filter: {}", config.device_name);

    let midi_mgr = MidiManager::new(&config.device_name);
    let mut engine = ActionEngine::new(config);

    let (tx, rx) = mpsc::channel::<MidiEvent>();

    info!("Searching for '{}'...", midi_mgr.find_port(&midir::MidiInput::new("init")?).map(|p| p.1).unwrap_or_else(|| "device".to_string()));
    let _conn = midi_mgr.wait_for_connection(tx.clone(), Duration::from_secs(2));

    info!("Bridge active! Focus DaVinci Resolve to use hardware controls.");
    info!("Press Ctrl+C to exit.");

    while let Ok(event) = rx.recv() {
        engine.handle_event(event);
    }

    Ok(())
}
