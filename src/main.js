if (window.__TAURI__) {
  var { invoke } = window.__TAURI__.tauri;
  // Import event module and use listen function
  var { emit, listen } = window.__TAURI__.event;
}

async function open_midi_connection() {
  if (window.__TAURI__) {
    await invoke("open_midi_connection");
  }
}

async function update_synth() {  
  if (window.__TAURI__) {
    await invoke("update_synth");
  }
}

import {midi_player} from './midi_player.js';

let computer_keyboard_keys = [
  "a",
  "w",
  "s",
  "e",
  "d",
  "f",
  "t",
  "g",
  "y",
  "h",
  "u",
  "j",
  "k",
  "o",
  "l",
  "p",
];

window.addEventListener("DOMContentLoaded", () => {
  open_midi_connection();
  update_synth();

  // Get body element
  const body = document.querySelector("body");
  console.log(body);
  // Create a new div inside the body with the class widget-container
  const widget_container = document.createElement("div");
  widget_container.classList.add("widget-container");
  body.appendChild(widget_container);
  // Create a new tag to label the piano
  const piano_label = document.createElement("h2");
  piano_label.innerHTML = "Live Piano";
  piano_label.classList.add("piano-label");
  widget_container.appendChild(piano_label);
  // Creat a new div inside the body with the class piano
  const piano = document.createElement("div");
  piano.classList.add("piano");
  widget_container.appendChild(piano);

  // Spawn keys inside the piano
  for (let i = 36; i < 85; i++) {
  // for (let i = 24; i < 97; i++) {
    // Create a key parent div
    const key_parent = document.createElement("div");
    key_parent.classList.add("key-parent");
    // Create a new div with the class key
    const key = document.createElement("div");
    key.classList.add("key");
    key.classList.add(`k${i}`);

    // If the key is mapped to the computer keyboard, add the corrosponding letter
    if (i >= 60 && i <= 75) {
      key.innerHTML = `<kbd>${computer_keyboard_keys[i - 60]}</kbd>`;
    }

    // Detect whether the key is black or white
    if (i % 12 == 1 || i % 12 == 3 || i % 12 == 6 || i % 12 == 8 || i % 12 == 10) {
      key_parent.classList.add("black");
      key.classList.add("black");
    } else {
      key_parent.classList.add("white");
      key.classList.add("white");
    }

    // Add mousedown event listener to the key
    key.addEventListener("mousedown", () => {
      console.log("MIDI message sent!")
      emit('midi_message', {
        message: [144, i, 100],
      })
    });
    // Add mouseup event listener to the key
    key.addEventListener("mouseup", () => {
      console.log("MIDI message sent!")
      emit('midi_message', {
        message: [128, i, 100],
      })
    });

    key_parent.appendChild(key);
    piano.appendChild(key_parent);
  }

  console.log("Hello from JS!")
  // listen for event "midi-message"
  if (window.__TAURI__) {
    const unlisten = listen("midi_message", (event) => {
      console.log(event);
      // console.log("MIDI message received!")
      console.log(event.payload.message[1]);
      const key = document.querySelector(`.k${event.payload.message[1]}`);
      if (event.payload.message[0] == 144) {
        key.classList.add("pressed");
      } else {
        key.classList.remove("pressed");
      }
    })
  }

  midi_player();
});

document.addEventListener("keypress", function(event) {
  key_event(event.key, "down");
});
document.addEventListener("keyup", function(event) {
  key_event(event.key, "up");
});

// Function to call send_note_key when corrosponding computer keys are pressed
function key_event(key, type) { 
  // Map computer keyboard keys to piano keys
  for (let i = 0; i < computer_keyboard_keys.length; i++) {
    if (key == computer_keyboard_keys[i]) {
      send_note_key(60 + i, type);
    }
  }
}

// Function to play or stop a note
function send_note_key(key, type) {
  if (type == "down") {
    emit('midi_message', {
      message: [144, key, 100],
    })
  } else {
    emit('midi_message', {
      message: [128, key, 100],
    })
  }
}