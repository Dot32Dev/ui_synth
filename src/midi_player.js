if (window.__TAURI__) {
	var { invoke } = window.__TAURI__.tauri;
	// Import event module and use listen function
	var { emit, listen } = window.__TAURI__.event;
  }
  
  async function fild_upload() {
	if (window.__TAURI__) {
	  await invoke("file_upload");
	}
  }

  console.log("MIDI player loaded")

  export function midi_player() {
	// Get body element
	const body = document.querySelector("body");
	console.log(body);
	// Create a new div inside the body with the class widget-container
	const widget_container = document.createElement("div");
	widget_container.classList.add("widget-container");
	body.appendChild(widget_container);
	// Create a new tag to label the widget
	const widget_label = document.createElement("h2");
	widget_label.innerHTML = "Midi Player";
	widget_container.appendChild(widget_label);
	// Create a new div inside the body with the class widget
	const widget = document.createElement("div");
	widget.classList.add("midi_player");
	widget_container.appendChild(widget);
	// Create a button inside of the midi_player widget
	const button = document.createElement("button");
	button.innerHTML = "Upload Midi File";
	widget.appendChild(button);
	// On button click, open file dialog
	button.addEventListener("click", () => {
	  fild_upload();
	});

	if (window.__TAURI__) {
		const unlisten = listen("midi_file_data", (event) => {
			// Get the midi_player widget
			const midi_player_widget = document.querySelector(".midi_player");
			// Create a new p tag inside of the midi_player widget
			const midi_player_text = document.createElement("p");
			midi_player_text.innerHTML = event.payload;
			midi_player_widget.appendChild(midi_player_text);

			console.log(event.payload);
		})
	  }
  }