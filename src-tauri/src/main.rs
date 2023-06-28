// Dissables the console window on windows when not in debug mode
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use midir::{MidiInput, MidiInputConnection};
use midly::Track;
use rodio::source::Source;
use rodio::OutputStream;

// Import synth module
mod oscillator;
mod synth;

use oscillator::Oscillator;
use synth::{Envelope, Synth};

use serde::{Deserialize, Serialize};
// use core::time;
// use tauri::http::header;
use std::sync::{Arc, Mutex};
use tauri::{Manager, Window, Wry};
use tauri::api::dialog;

#[derive(Default)]
struct MidiState {
    pub input: Mutex<Option<MidiInputConnection<()>>>,
}

struct SynthState {
    synth: Mutex<Synth>,
}

#[derive(Default)]
struct MidiPlayerState<'a> {
    arangements: Mutex<Vec<TrackPlus<'a>>>,
    tempo: Mutex<u32>,
    track_time: Mutex<u32>,
    length_in_ticks: Mutex<u32>,
}

struct TrackPlus<'a> {
    track: Track<'a>,
    timing: midly::Timing,
}

#[derive(Clone, Serialize, Deserialize)]
struct MidiMessage {
    message: Vec<u8>,
}

struct MidiData {
    tempo: u32,
    length_in_ticks: u32,
    meta_track_index: Option<usize>,
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

static mut file_data: Vec<u8> = Vec::new();

#[tauri::command]
async fn file_upload(window: Window<Wry>) {
    let path_buf = dialog::blocking::FileDialogBuilder::default()
        .add_filter("Midi", &["midi", "mid"])
        .pick_file();
    unsafe {
        if let Some(p) = path_buf {
            file_data = std::fs::read(p).unwrap();
        }
        let mut smf = midly::Smf::parse(&file_data).unwrap();

        let track_count = smf.tracks.len();
        println!("Track count: {}", track_count);
        let header = smf.header;
        let timing = header.timing;
        // let track_tempo = get_tempo(&smf);
        let data = get_midi_data(&smf);
        println!("Length in ticks: {}", data.length_in_ticks);
        println!("Tempo: {}", data.tempo);
        println!("Meta track index: {:?}", data.meta_track_index);

        smf.tracks.remove(data.meta_track_index.unwrap());

        let handle = Arc::new(window).clone();
        handle
            .emit("midi_file_data", ()) //p.to_str().unwrap().to_string())
            .map_err(|e| {
                println!("Error sending midi message: {}", e);
            })
            .ok();

        // Use the handle to get the midi player state
        let midi_player_state = handle.state::<MidiPlayerState>();

        // Add the data to the midi player state
        let mut arangements = midi_player_state.arangements.lock().unwrap();
        arangements.clear();
        for track in smf.tracks {
            let track_plus = TrackPlus {
                track: track.clone(),
                timing: timing,
            };
            arangements.push(track_plus);
        }
    }
    // let mut arangements = Vec::new();
    // for track in smf.tracks.iter() {
    //     let track_plus = TrackPlus {
    //         track: track.clone(),
    //         timing,
    //     };
    //     arangements.push(track_plus);
    // }
    // let midi_player_state = handle.state::<Arc<Mutex<MidiPlayerState>>>();
    // let mut midi_player_state = midi_player_state.lock().unwrap();
    // midi_player_state.arangements = Arc::new(Mutex::new(arangements));
}

#[tauri::command(async)]
fn play_arrangement(midi_player_state: tauri::State<'_, MidiPlayerState>, window: Window<Wry>) {
    let handle = Arc::new(window).clone();
    let midi_player_state = &midi_player_state;
    let tempo = midi_player_state.tempo.lock().unwrap();
    let arangements = midi_player_state.arangements.lock().unwrap();
    let length_in_ticks = midi_player_state.length_in_ticks.lock().unwrap();
    let full_track_time = midi_player_state.track_time.lock().unwrap();
    let mut full_track_time = *full_track_time;

    let mut next_track_times = vec![0; arangements.len()];
    let mut track_iterators = vec![];
    for track in arangements.iter() {
        let track_iterator = track.track.iter();
        track_iterators.push(track_iterator);
    }
    let mut track_timings = vec![];
    for track in arangements.iter() {
        let track_timing = track.timing;
        track_timings.push(track_timing);
    }
    let mut time_per_ticks = vec![];
    let mut length_in_microseconds = 0.0;
    for track_timing in track_timings.iter() {
        match track_timing {
            midly::Timing::Metrical(ticks_per_beat) => {
                let time_per_tick = (tempo.clone() as f32) / ticks_per_beat.as_int() as f32;
                time_per_ticks.push(time_per_tick);
                // Update length in microseconds if new calculated length is longer
                if length_in_microseconds < (*length_in_ticks as f32 * time_per_tick) {
                    length_in_microseconds = *length_in_ticks as f32 * time_per_tick;
                }
            }
            midly::Timing::Timecode(fps, _) => {
                let time_per_tick = 1.0 / fps.as_f32();
                time_per_ticks.push(time_per_tick);
                // Update length in microseconds if new calculated length is longer
                if length_in_microseconds < (*length_in_ticks as f32 * time_per_tick) {
                    length_in_microseconds = *length_in_ticks as f32 * time_per_tick;
                }
            }
        }
    }

    let now = std::time::Instant::now();
    loop {
        let mut num_finished_tracks = 0;
        for (i, track_time) in next_track_times.iter_mut().enumerate() {
            if *track_time <= full_track_time {
                let event = track_iterators[i].next();
                match event {
                    Some(event) => {
                        let delta_time = event.delta.as_int();
                        *track_time += delta_time * time_per_ticks[i] as u32;
                        mildy_event_handler(*event, handle.clone());
                    }
                    None => {
                        // If .next() returns None, the track is finished
                        num_finished_tracks += 1;
                    }
                }
            }
        }
        if num_finished_tracks == track_iterators.len() {
            println!("Finished playing");
            break;
        }

        let progress_bar_value = (full_track_time as f32 / length_in_microseconds) * 100.0;
        handle.emit(
            "update_progress_bar",
            progress_bar_value,
        )
        .map_err(|e| {
            println!("Error sending midi message: {}", e);
        })
        .ok();

        // Wait until the closest track time
        full_track_time = *next_track_times.iter().min().unwrap();
        let wait_time = now + std::time::Duration::from_micros(full_track_time as u64);
        while std::time::Instant::now() < wait_time {}
    }
}

fn mildy_event_handler(event: midly::TrackEvent, handle: Arc<tauri::Window>) {
    // Match the event
    match event.kind {
        // If the event is a note on event
        midly::TrackEventKind::Midi { channel: _, message } => {
            match message {
                // If the message is a note on message
                midly::MidiMessage::NoteOn { key, vel } => {
                    if vel > 0 {
                        // println!("Note on");
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

fn main() {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let synth = Mutex::new(Synth::new(stream_handle));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_midi_connection, update_synth, file_upload])
        .manage(MidiState::default())
        .manage(SynthState { synth })
        .manage(MidiPlayerState {
            tempo: 500000.into(), // 120 bpm
            ..Default::default()
        })
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

fn get_midi_data(smf: &midly::Smf) -> MidiData {
    let mut tempo = 500000;
    let mut length_in_ticks = 0;
    let mut meta_track_index = None;
    for (i, track) in smf.tracks.iter().enumerate() {
        let mut length_in_ticks_tmp = 0;
        for event in track.iter() {
            let delta_time = event.delta.as_int();
            length_in_ticks_tmp += delta_time;
            match event.kind {
                midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo_event)) => {
                    tempo = tempo_event.as_int();
                    meta_track_index = Some(i);
                }
                midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack) => {
                    if length_in_ticks_tmp > length_in_ticks {
                        length_in_ticks = length_in_ticks_tmp;
                    }
                    break;
                }
                _ => {}
            }
        }
    };
    MidiData { tempo, length_in_ticks, meta_track_index }
}