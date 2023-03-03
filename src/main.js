const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

window.addEventListener("DOMContentLoaded", () => {
  // greetInputEl = document.querySelector("#greet-input");
  // greetMsgEl = document.querySelector("#greet-msg");
  // document
  //   .querySelector("#greet-button")
  //   .addEventListener("click", () => greet());

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
