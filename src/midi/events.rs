#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    PitchBend { channel: u8, value: i16 }, // -8192 to 8191
}

impl MidiEvent {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let status = bytes[0];
        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            0x80 => {
                if bytes.len() >= 3 {
                    Some(MidiEvent::NoteOff {
                        channel,
                        note: bytes[1],
                    })
                } else {
                    None
                }
            }
            0x90 => {
                if bytes.len() >= 3 {
                    let note = bytes[1];
                    let velocity = bytes[2];
                    if velocity == 0 {
                        Some(MidiEvent::NoteOff { channel, note })
                    } else {
                        Some(MidiEvent::NoteOn {
                            channel,
                            note,
                            velocity,
                        })
                    }
                } else {
                    None
                }
            }
            0xB0 => {
                if bytes.len() >= 3 {
                    Some(MidiEvent::ControlChange {
                        channel,
                        controller: bytes[1],
                        value: bytes[2],
                    })
                } else {
                    None
                }
            }
            0xE0 => {
                if bytes.len() >= 3 {
                    let lsb = bytes[1] as u16;
                    let msb = bytes[2] as u16;
                    let raw = (msb << 7) | lsb; // 0..16383, 8192 is center
                    let centered = (raw as i32) - 8192;
                    Some(MidiEvent::PitchBend {
                        channel,
                        value: centered as i16,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_and_off() {
        let note_on = [0x90, 60, 100];
        assert_eq!(
            MidiEvent::parse(&note_on),
            Some(MidiEvent::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100
            })
        );

        let note_on_zero_vel = [0x90, 60, 0];
        assert_eq!(
            MidiEvent::parse(&note_on_zero_vel),
            Some(MidiEvent::NoteOff {
                channel: 0,
                note: 60
            })
        );

        let note_off = [0x80, 60, 64];
        assert_eq!(
            MidiEvent::parse(&note_off),
            Some(MidiEvent::NoteOff {
                channel: 0,
                note: 60
            })
        );
    }

    #[test]
    fn test_cc_and_pitchbend() {
        let cc = [0xB0, 11, 75];
        assert_eq!(
            MidiEvent::parse(&cc),
            Some(MidiEvent::ControlChange {
                channel: 0,
                controller: 11,
                value: 75
            })
        );

        // Center pitchbend (0x2000 = 8192) -> centered value 0
        let pb_center = [0xE0, 0x00, 0x40];
        assert_eq!(
            MidiEvent::parse(&pb_center),
            Some(MidiEvent::PitchBend {
                channel: 0,
                value: 0
            })
        );
    }
}
