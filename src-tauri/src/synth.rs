use std::collections::HashMap;
use std::time::Instant;

use rodio::source::Source;
use rodio::Sink;

// The envelope struct
pub struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl Envelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Envelope {
        Envelope {
            attack,
            decay,
            sustain,
            release,
        }
    }
}

// The active note struct
struct ActiveNote {
    envelope: Envelope,
    start_time: Instant,
    is_releasing: bool,
    time_released: Option<Instant>,
    sink: Sink,
}

impl ActiveNote {
    fn time_since_start(&self) -> f32 {
        Instant::now().duration_since(self.start_time).as_secs_f32()
    }

    fn time_since_release(&self) -> Option<f32> {
        self.time_released
            .map(|time| Instant::now().duration_since(time).as_secs_f32())
    }
}

pub struct Synth {
    active_notes: HashMap<u8, ActiveNote>,
    stream_handle: rodio::OutputStreamHandle,
}

impl Synth {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Synth {
        Synth {
            active_notes: HashMap::new(),
            stream_handle,
        }
    }

    pub fn play_source(
        &mut self,
        audio_source: Box<dyn Source<Item = f32> + Send>, // This will likely be created with the Oscillator
        source_id: u8, // This is to differentiate between different "sources", so that multiple can be played at once
        envelope: Envelope, // The envelope will effect the volume of the audio source over time
    ) {
        // Sink is a handle to the audio output device
        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(audio_source);

        let active_note = ActiveNote {
            envelope,
            start_time: Instant::now(),
            is_releasing: false,
            time_released: None,
            sink,
        };

        self.active_notes.insert(source_id, active_note);
    }

    pub fn release_source(&mut self, source_id: u8) {
        if let Some(active_note) = self.active_notes.get_mut(&source_id) {
            active_note.is_releasing = true;
            active_note.time_released = Some(Instant::now());
        }
    }

    pub fn update(&mut self) {
        let mut to_remove = Vec::new();

        for (source_id, active_note) in self.active_notes.iter_mut() {
            let elapsed = active_note.time_since_start();

            let envelope = &active_note.envelope;

            let volume = if elapsed < envelope.attack {
                // Attack
                elapsed / envelope.attack
            } else if elapsed < envelope.attack + envelope.decay {
                // Decay
                1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
            } else if active_note.is_releasing {
                // Release
                let time_since_release = active_note
                    .time_since_release()
                    .unwrap_or(0.0)
                    .min(envelope.release);
                envelope.sustain - time_since_release / envelope.release * envelope.sustain
            } else {
                // Sustain
                envelope.sustain
            };

            active_note.sink.set_volume(volume);

            if active_note.is_releasing {
                if active_note.time_since_release().unwrap_or(0.0) >= envelope.release + 0.1 {
                    to_remove.push(*source_id);
                }
            }
        }

        for source_id in to_remove {
            // This is done as a seperate loop to avoid a second mutable borrow of self.active_notes
            // First borrow is when .iter_mut() is called, second is when .remove() is called
            self.active_notes.remove(&source_id);
        }
    }
}
