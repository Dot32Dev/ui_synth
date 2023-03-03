// Dissables the console window on windows when not in debug mode
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rodio::OutputStream;
use rodio::source::Source;
use rodio::Sink;
use midir::MidiInput;
// Import synth module
mod synth;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Create a new midi input
    let midi_in = MidiInput::new("midir reading input").unwrap();

    // Get an input port (Automatically choosing the first one) 
    // (It will panic if no midi device is connected)
    let in_port = &midi_in.ports()[0];

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_stamp, message, _| {
        // Message is in the format of [event, key, pressure]
        let hz = 440.0 * 2.0_f32.powf((message[1] as f32 - 69.0) / 12.0);
        let pressure = message[2] as f32 / 127.0;

        if message[0] == 144 { // 144 is the event for note on
            sink.stop();
            sink.append(synth::Synth::square_wave(hz).amplify(pressure));
            println!("hz: {}", hz);
            // stream_handle.play_raw(synth::Synth::square_wave(hz).amplify(0.1)).unwrap();
        }
        if message[0] == 128 { // 128 is the event for note off
            sink.stop();
            println!("Stop");
        }
    }, ()).unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
