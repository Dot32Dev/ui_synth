// Dissables the console window on windows when not in debug mode
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rodio::OutputStream;
use rodio::source::Source;
// use midir::MidiInput;
use midir::{MidiInput, MidiInputConnection};
// Import synth module
mod synth;
mod oscillator;

use synth::Synth;
use oscillator::Oscillator;

use std::sync::{Arc, Mutex};
use tauri::{Manager, Window, Wry};
use serde::{Serialize, Deserialize};

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
fn open_midi_connection(
  midi_state: tauri::State<'_, MidiState>,
  window: Window<Wry>,
) {
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
fn update_synth(
    synth_state: tauri::State<'_, SynthState>,
) {
    loop {
        let synth_state = &synth_state.synth;
        let mut synth = synth_state.lock().unwrap();
        synth.update();
    }
}

fn main() {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let synth =  Arc::new(Mutex::new(Synth::new(stream_handle)));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_midi_connection, update_synth])
        .manage(MidiState::default())
        .manage(SynthState { synth })
        .setup(|app| {
            let handle = app.handle();
            let _id = app.listen_global("midi_message", move |event| {
                // Get the synth state
                let synth_state = &handle.state::<SynthState>().synth;
                let mut synth = synth_state.lock().unwrap();

                // Deserialize the payload
                let message = serde_json::from_str::<MidiMessage>(event.payload().unwrap()).unwrap();
                let message = message.message;

                let hz = 440.0 * 2.0_f32.powf((message[1] as f32 - 69.0) / 12.0);
                let pressure = message[2] as f32 / 127.0;

                if message[0] == 144 { // 144 is the event for note on
                    let audio_source = Oscillator::sawtooth_wave(hz).amplify(pressure);
                    synth.play_source(Box::new(audio_source), message[1])
                }
                if message[0] == 128 { // 128 is the event for note off
                    synth.release_source(message[1])
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
