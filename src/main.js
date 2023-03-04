const { invoke } = window.__TAURI__.tauri;
// Import event module and use listen function
const { emit, listen } = window.__TAURI__.event;

async function open_midi_connection() {
  await invoke("open_midi_connection");
}

window.addEventListener("DOMContentLoaded", () => {
  open_midi_connection();

  // Get body element
  const body = document.querySelector("body");
  console.log(body);
  // Creat a new div inside the body with the class piano
  const piano = document.createElement("div");
  piano.classList.add("piano");
  body.appendChild(piano);
  // Spawn keys inside the piano
  for (let i = 36; i < 85; i++) {
    const key = document.createElement("div");
    key.classList.add("key");
    key.classList.add(`k${i}`);
    // key.innerHTML = i;

    // Detect whether the key is black or white
    if (i % 12 == 1 || i % 12 == 3 || i % 12 == 6 || i % 12 == 8 || i % 12 == 10) {
      key.classList.add("black");
    } else {
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

    piano.appendChild(key);
  }

  console.log("Hello from JS!")
  // listen for event "midi-message"
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

});