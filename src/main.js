const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
let micSelectEl;  // ✅ Nuevo referencia al select

let isRecording = false;
let audioChunks = [];

// ── Cargar lista de micrófonos ─────────────────────────────────────────────
async function loadAudioDevices() {
  try {
    const devices = await invoke('list_audio_devices');
    micSelectEl.innerHTML = '<option value="">Default</option>'; // Reset

    devices.forEach(device => {
      const option = document.createElement('option');
      option.value = device.name;
      option.textContent = device.name;
      micSelectEl.appendChild(option);
    });

    console.log(`✅ ${devices.length} micrófonos encontrados`);
  } catch (err) {
    console.error("❌ Error cargando dispositivos:", err);
    // Fallback: mostrar error en el select
    micSelectEl.innerHTML = '<option>Error loading devices</option>';
  }
}

// ── Control de Micrófono ─────────────────────────────────────────────
async function toggleRecording() {
  const selectedDevice = micSelectEl.value || null; // "" → null → default

  if (!isRecording) {
    try {
      // ✅ Pasamos el nombre del dispositivo seleccionado
      await invoke('start_listening', { deviceName: selectedDevice });
      isRecording = true;
      document.getElementById('mic-btn').innerHTML = '⏹️';
      document.getElementById('mic-btn').classList.add('recording');
    } catch (err) {
      console.error("Error al iniciar micro nativo:", err);
      alert(`Error: ${err}`);
    }
  } else {
    const prompt = document.getElementById('greet-input').value;
    try {
      const response = await invoke('stop_and_process', { prompt });
      console.log("Respuesta del motor:", response);
      await speak(response);
      // Opcional: mostrar respuesta o reproducir TTS
      greetMsgEl.textContent = response;
    } catch (err) {
      console.error("Error procesando audio:", err);
    } finally {
      isRecording = false;
      document.getElementById('mic-btn').innerHTML = '🎤';
      document.getElementById('mic-btn').classList.remove('recording');
    }
  }
}

// ── TTS ───────────────────────────────────────────────────────────────────────

async function speak(text) {
  greetMsgEl.textContent = "Sintetizando...";

  try {
    // invoke devuelve array de i8 (números con signo)
    const wavBytes = await invoke("synthesize", { text, voice: "F1" });
    
    // 🔑 Convertir i8 (con signo) → u8 (sin signo) para el Blob
    // En JS, los bytes negativos deben sumarse 256 para obtener su valor u8 equivalente
    const uint8 = new Uint8Array(wavBytes.map(b => b < 0 ? b + 256 : b));

    // Debug: verificar header WAV
    if (uint8.length < 44) throw new Error(`Datos muy cortos: ${uint8.length} bytes`);
    const header = new TextDecoder().decode(uint8.slice(0, 4));
    if (header !== "RIFF") throw new Error(`Header inválido: "${header}"`);

    greetMsgEl.textContent = "▶ Reproduciendo...";
    const blob = new Blob([uint8], { type: "audio/wav" });
    const audioUrl = URL.createObjectURL(blob);

    const audio = new Audio(audioUrl);

    audio.onloadeddata = () => console.log("✅ Audio cargado:", audio.duration, "s");
    audio.onerror = (e) => {
      URL.revokeObjectURL(audioUrl);
      console.error("❌ Error de audio:", e);
      greetMsgEl.textContent = `❌ Error: ${e.message}`;
    };
    audio.onended = () => {
      URL.revokeObjectURL(audioUrl);
      greetMsgEl.textContent = "✓ Listo";
    };

    await audio.play();

  } catch (err) {
    console.error("❌ Error general:", err);
    greetMsgEl.textContent = `Error: ${err.message || JSON.stringify(err)}`;
  }
}

// ── Greet original (ahora habla en vez de solo mostrar texto) ─────────────────

async function greet() {
  const text = greetInputEl.value;
  // Muestra el mensaje Y lo habla
  try {
    const testresponse = await invoke("test_inference", { testPrompt: text });
    await speak(testresponse);
  } catch (err) {
    console.error("❌ Error test_inference:", err);
    greetMsgEl.textContent = `Error: ${err.message || JSON.stringify(err)}`;
  }
  // greetMsgEl.textContent = await invoke("greet", { name: text });
}

// ── DOMContentLoaded ─────────────────────────────────────────────────────
window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  micSelectEl = document.querySelector("#mic-select");  // ✅ Nueva referencia

  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  // ✅ Botón de micrófono
  const micBtn = document.getElementById('mic-btn');
  if (micBtn) {
    micBtn.onclick = toggleRecording
  }

  // ✅ Botón para refrescar dispositivos
  const refreshBtn = document.getElementById('refresh-mics');
  if (refreshBtn) {
    refreshBtn.onclick = loadAudioDevices;
  }

  // ✅ Cargar dispositivos al iniciar
  loadAudioDevices();
});