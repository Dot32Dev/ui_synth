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

// The envelope state struct
struct EnvelopeState {
    envelope: Envelope,
    start_time: Instant,
    is_releasing: bool,
    time_released: Option<Instant>,
}

impl EnvelopeState {
    fn time_since_start(&self) -> f32 {
        Instant::now().duration_since(self.start_time).as_secs_f32()
    }

    fn time_since_release(&self) -> Option<f32> {
        self.time_released.map(|time| {
            Instant::now()
                .duration_since(time)
                .as_secs_f32()
                .min(self.envelope.release)
        })
    }
}

pub struct Synth {
    audio_sinks: HashMap<u8, Sink>,
    envelope_states: HashMap<u8, EnvelopeState>,
    stream_handle: rodio::OutputStreamHandle,
}

impl Synth {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Synth {
        Synth {
            audio_sinks: HashMap::new(),
            envelope_states: HashMap::new(),
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

        let envelope_state = EnvelopeState {
            envelope,
            start_time: Instant::now(),
            is_releasing: false,
            time_released: None,
        };

        self.audio_sinks.insert(source_id, sink);
        self.envelope_states.insert(source_id, envelope_state);
    }

    pub fn release_source(&mut self, source_id: u8) {
        if let Some(envelope_state) = self.envelope_states.get_mut(&source_id) {
            envelope_state.is_releasing = true;
            envelope_state.time_released = Some(Instant::now());
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        let mut to_remove = Vec::new();

        for (source_id, envelope_state) in self.envelope_states.iter_mut() {
            let elapsed = envelope_state.time_since_start();

            let envelope = &envelope_state.envelope;
            let sink = self.audio_sinks.get_mut(source_id).unwrap();

            let volume = if elapsed < envelope.attack {
                // Attack
                elapsed / envelope.attack
            } else if elapsed < envelope.attack + envelope.decay {
                // Decay
                1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
            } else if envelope_state.is_releasing {
                // Release
                envelope_state.time_since_release().unwrap() / envelope.release * envelope.sustain
            } else {
                // Sustain
                envelope.sustain
            };

            sink.set_volume(volume);

            if envelope_state.is_releasing {
                if envelope_state.time_since_release().unwrap() >= envelope.release + 0.1 {
                    to_remove.push(*source_id);
                }
            }
        }

        for source_id in to_remove {
            // This is done as a seperate loop to avoid a second mutable borrow of self.envelope_states
            // First borrow is when .iter_mut() is called, second is when .remove() is called
            self.envelope_states.remove(&source_id);
            self.audio_sinks.remove(&source_id);
        }
    }
}
