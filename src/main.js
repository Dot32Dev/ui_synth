if (window.__TAURI__) {
  const { invoke } = window.__TAURI__.tauri;
  // Import event module and use listen function
  const { emit, listen } = window.__TAURI__.event;
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

window.addEventListener("DOMContentLoaded", () => {
  open_midi_connection();
  update_synth();

  // Get body element
  const body = document.querySelector("body");
  console.log(body);
  // Creat a new div inside the body with the class piano
  const piano = document.createElement("div");
  piano.classList.add("piano");
  body.appendChild(piano);
  // Spawn keys inside the piano
  for (let i = 36; i < 85; i++) {
    // Create a key parent div
    const key_parent = document.createElement("div");
    key_parent.classList.add("key-parent");
    // Create a new div with the class key
    const key = document.createElement("div");
    key.classList.add("key");
    key.classList.add(`k${i}`);
    key.innerHTML = i;

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
  if (key == "a") {
    send_note_key(60, type);
  }
  if (key == "w") {
    send_note_key(61, type);
  }
  if (key == "s") {
    send_note_key(62, type);
  }
  if (key == "e") {
    send_note_key(63, type);
  }
  if (key == "d") {
    send_note_key(64, type);
  }
  if (key == "f") {
    send_note_key(65, type);
  }
  if (key == "t") {
    send_note_key(66, type);
  }
  if (key == "g") {
    send_note_key(67, type);
  }
  if (key == "y") {
    send_note_key(68, type);
  }
  if (key == "h") {
    send_note_key(69, type);
  }
  if (key == "u") {
    send_note_key(70, type);
  }
  if (key == "j") {
    send_note_key(71, type);
  }
  if (key == "k") {
    send_note_key(72, type);
  }
  if (key == "o") {
    send_note_key(73, type);
  }
  if (key == "l") {
    send_note_key(74, type);
  }
  if (key == "p") {
    send_note_key(75, type);
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