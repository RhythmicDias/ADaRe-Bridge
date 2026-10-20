pub mod events;

use std::sync::mpsc::Sender;
use std::time::Duration;
use anyhow::{anyhow, Result};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use tracing::{info, warn};

pub use events::MidiEvent;

pub struct MidiManager {
    device_filter: String,
}

impl MidiManager {
    pub fn new(device_filter: &str) -> Self {
        Self {
            device_filter: device_filter.to_lowercase(),
        }
    }

    pub fn find_port(&self, midi_in: &MidiInput) -> Option<(MidiInputPort, String)> {
        for port in midi_in.ports() {
            if let Ok(name) = midi_in.port_name(&port) {
                if name.to_lowercase().contains(&self.device_filter) {
                    return Some((port, name));
                }
            }
        }
        None
    }

    pub fn connect(
        &self,
        event_sender: Sender<MidiEvent>,
    ) -> Result<MidiInputConnection<()>> {
        let midi_in = MidiInput::new("resolve-mpk-input")
            .map_err(|e| anyhow!("Failed to initialize MIDI input: {e}"))?;

        let (port, port_name) = self
            .find_port(&midi_in)
            .ok_or_else(|| anyhow!("Device matching '{}' not found", self.device_filter))?;

        info!("Connected to MIDI device: {}", port_name);

        let connection = midi_in
            .connect(
                &port,
                "resolve-mpk-listener",
                move |_timestamp, bytes, _| {
                    if let Some(event) = MidiEvent::parse(bytes) {
                        let _ = event_sender.send(event);
                    }
                },
                (),
            )
            .map_err(|e| anyhow!("Failed to establish MIDI connection: {e}"))?;

        Ok(connection)
    }

    pub fn wait_for_connection(
        &self,
        event_sender: Sender<MidiEvent>,
        poll_interval: Duration,
    ) -> MidiInputConnection<()> {
        loop {
            match self.connect(event_sender.clone()) {
                Ok(conn) => return conn,
                Err(err) => {
                    warn!("{}. Retrying in {:?}...", err, poll_interval);
                    std::thread::sleep(poll_interval);
                }
            }
        }
    }
}
