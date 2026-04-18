const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

let isRecording = false;
let mediaRecorder = null;
let audioChunks = [];

// ── Control de Micrófono ─────────────────────────────────────────────
async function toggleRecording() {
  if (!isRecording) {
    // Iniciar captura
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          sampleRate: 16000,  // Coincidir con config Rust
          channelCount: 1,
          echoCancellation: true
        }
      });

      mediaRecorder = new MediaRecorder(stream, {
        mimeType: 'audio/webm;codecs=pcm',
        audioBitsPerSecond: 256000
      });

      mediaRecorder.ondataavailable = e => audioChunks.push(e.data);
      mediaRecorder.start(100); // Colectar cada 100ms

      await invoke('start_listening');
      isRecording = true;
      document.getElementById('mic-btn').classList.add('recording');

    } catch (err) {
      console.error('❌ Error mic:', err);
    }
  } else {
    // Detener y procesar
    mediaRecorder?.stop();
    mediaRecorder?.stream?.getTracks().forEach(t => t.stop());

    const prompt = document.getElementById('greet-input').value;
    document.getElementById('greet-msg').textContent = "🔄 Procesando audio...";

    try {
      const response = await invoke('stop_and_process', { prompt });
      document.getElementById('greet-msg').textContent = response;

      // Opcional: TTS de la respuesta
      await speak(response);

    } catch (err) {
      console.error('❌ Error processing:', err);
      document.getElementById('greet-msg').textContent = `Error: ${err}`;
    }

    audioChunks = [];
    isRecording = false;
    document.getElementById('mic-btn').classList.remove('recording');
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

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
  // Agregar botón de mic
  const micBtn = document.createElement('button');
  micBtn.id = 'mic-btn';
  micBtn.innerHTML = '🎤';
  micBtn.onclick = toggleRecording;
  document.querySelector('.row').appendChild(micBtn);
});
