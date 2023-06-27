// Dissables the console window on windows when not in debug mode
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use midir::{MidiInput, MidiInputConnection};
use rodio::source::Source;
use rodio::OutputStream;

// Import synth module
mod oscillator;
mod synth;

use oscillator::Oscillator;
use synth::{Envelope, Synth};

use serde::{Deserialize, Serialize};
use tauri::http::header;
use std::sync::{Arc, Mutex};
use tauri::{Manager, Window, Wry};
use tauri::api::dialog;

#[derive(Default)]
struct MidiState {
    pub input: Mutex<Option<MidiInputConnection<()>>>,
}

struct SynthState {
    synth: Arc<Mutex<Synth>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct MidiMessage {
    message: Vec<u8>,
}

#[tauri::command]
fn open_midi_connection(midi_state: tauri::State<'_, MidiState>, window: Window<Wry>) {
    let handle = Arc::new(window).clone();
    let midi_in = MidiInput::new("ui_synth");
    match midi_in {
        Ok(midi_in) => {
            let midi_in_ports = midi_in.ports();
            let port = midi_in_ports.get(0);
            match port {
                Some(port) => {
                    // Print the name of the port
                    println!("Port: {}", midi_in.port_name(port).unwrap());
                    let midi_in_conn = midi_in.connect(
                        port,
                        "midir",
                        move |_, message, _| {
                            println!("Message: {:?}", message);
                            handle
                                .emit_and_trigger(
                                    "midi_message",
                                    MidiMessage {
                                        message: message.to_vec(),
                                    },
                                )
                                .map_err(|e| {
                                    println!("Error sending midi message: {}", e);
                                })
                                .ok();
                        },
                        (),
                    );
                    match midi_in_conn {
                        Ok(midi_in_conn) => {
                            midi_state.input.lock().unwrap().replace(midi_in_conn);
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                None => {
                    println!("No port found at index {}", 0);
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[tauri::command(async)]
fn update_synth(synth_state: tauri::State<'_, SynthState>) {
    loop {
        let synth_state = &synth_state.synth;
        let mut synth = synth_state.lock().unwrap();
        synth.update();
    }
}

#[tauri::command(async)]
fn file_upload(window: Window<Wry>) {
    dialog::FileDialogBuilder::default()
        .add_filter("Midi", &["midi", "mid"])
        .pick_file(|path_buf| match path_buf {
        Some(p) => {
            let data = std::fs::read(p.clone()).unwrap();
            let mut smf = midly::Smf::parse(&data).unwrap();

            let track_count = smf.tracks.len();
            println!("Track count: {}", track_count);
            let header = smf.header;
            let timing = header.timing;
            println!("Timing: {:?}", timing);
            let mut track_tempo = 0;
            let meta_track = smf.tracks.remove(1);
            for event in meta_track.iter() {
                println!("Event: {:?}", event);
                match event.kind {
                    midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) => {
                        println!("Tempo: {}", tempo);
                        track_tempo = tempo.into();
                    }
                    _ => {}
                }
            }
            let time_per_tick;
            match timing {
                midly::Timing::Metrical(ticks_per_beat) => {
                    time_per_tick = ((track_tempo as f32) / ticks_per_beat.as_int() as f32);
                }
                midly::Timing::Timecode(fps, _) => {
                    time_per_tick = 1.0 / fps.as_f32();
                }
            }
            println!("Time per tick: {}", time_per_tick);
            let first_track = smf.tracks.remove(2);

            let handle = Arc::new(window).clone();
            handle
                .emit(
                    "midi_file_data",
                    p.to_str().unwrap().to_string(),
                )
                .map_err(|e| {
                    println!("Error sending midi message: {}", e);
                })
                .ok();

            // Get std instant of current time
            let now = std::time::Instant::now();
            let mut track_time = 0;
            // Loop over the first track
            for event in first_track.iter() {
                println!("Event: {:?}", event);
                // Get the delta time of the event
                let delta_time = event.delta.as_int();
                track_time += delta_time*time_per_tick as u32;
                // Get the time to wait before playing the event
                let wait_time = now + std::time::Duration::from_micros(track_time as u64);
                // Wait until the time to play the event
                while std::time::Instant::now() < wait_time {}
                // Match the event
                match event.kind {
                    // If the event is a note on event
                    midly::TrackEventKind::Midi { channel: _, message } => {
                        match message {
                            // If the message is a note on message
                            midly::MidiMessage::NoteOn { key, vel } => {
                                if vel > 0 {
                                    println!("Note on");
                                    handle.emit_and_trigger("midi_message", MidiMessage { message: vec![144, key.into(), vel.into()] }).map_err(|e| {
                                        println!("Error sending midi message: {}", e);
                                    })
                                    .ok();
                                } else {
                                    handle.emit_and_trigger("midi_message", MidiMessage { message: vec![128, key.into(), vel.into()] }).map_err(|e| {
                                        println!("Error sending midi message: {}", e);
                                    })
                                    .ok();
                                }
                            }
                            // If the message is a note off message
                            midly::MidiMessage::NoteOff { key, vel } => {
                                handle.emit_and_trigger("midi_message", MidiMessage { message: vec![128, key.into(), vel.into()] }).map_err(|e| {
                                    println!("Error sending midi message: {}", e);
                                })
                                .ok();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
        });
}

fn main() {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let synth = Arc::new(Mutex::new(Synth::new(stream_handle)));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_midi_connection, update_synth, file_upload])
        .manage(MidiState::default())
        .manage(SynthState { synth })
        .setup(|app| {
            let handle = app.handle();
            let _id = app.listen_global("midi_message", move |event| {
                // Get the synth state
                let synth_state = &handle.state::<SynthState>().synth;
                let mut synth = synth_state.lock().unwrap();

                // Deserialize the payload
                let message =
                    serde_json::from_str::<MidiMessage>(event.payload().unwrap()).unwrap();
                let message = message.message;

                let hz = 440.0 * 2.0_f32.powf((message[1] as f32 - 69.0) / 12.0);
                let pressure = message[2] as f32 / 127.0;

                if message[0] == 144 {
                    // 144 is the event for note on
                    let audio_source = Oscillator::sawtooth_wave(hz).amplify(pressure);
                    let envelope = Envelope::new(0.0, 2.0, 0.0, 0.0); // example envelope
                    synth.play_source(Box::new(audio_source), message[1], envelope)
                }
                if message[0] == 128 {
                    // 128 is the event for note off
                    synth.release_source(message[1])
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
