const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

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
  greetMsgEl.textContent = await invoke("greet", { name: text });
  await speak(text);
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
