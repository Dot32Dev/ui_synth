const { invoke } = window.__TAURI__.tauri;
// Import event module and use listen function
const { listen } = window.__TAURI__.event;

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
  for (let i = 36; i < 84; i++) {
    const key = document.createElement("div");
    key.classList.add("key");

    // Detect whether the key is black or white
    if (i % 12 == 1 || i % 12 == 3 || i % 12 == 6 || i % 12 == 8 || i % 12 == 10) {
      key.classList.add("black");
    } else {
      key.classList.add("white");
    }

    piano.appendChild(key);
  }
});

// listen for event "midi-message"
listen("midi-message", (event) => {
  console.log(event);
  console.log("pogger")
})