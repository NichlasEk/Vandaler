const { invoke } = window.__TAURI__.core;

const state = {
  source: "",
  output: "audio/converted",
  summary: null,
  bank: null,
  bankPath: "",
  selectedInstrument: null,
  busyCount: 0,
  settingsLoaded: false,
  presets: {
    previewPreset: "balanced",
    bassGain: "1",
    leadGain: "0.85",
    psgGain: "0.85",
    dacGain: "1",
  },
};

const $ = (id) => document.getElementById(id);

function setLog(message) {
  $("log").textContent = message;
}

function paint() {
  return new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
}

function setBusy(progressId, buttonId, busy, label) {
  const progress = $(progressId);
  const button = $(buttonId);
  const globalBusy = $("globalBusy");
  const globalBusyLabel = $("globalBusyLabel");
  if (progress) progress.hidden = !busy;
  state.busyCount = Math.max(0, state.busyCount + (busy ? 1 : -1));
  if (globalBusy) globalBusy.hidden = state.busyCount === 0;
  if (globalBusyLabel && busy) globalBusyLabel.textContent = label || "Working...";
  if (!button) return;
  if (busy) {
    button.dataset.idleLabel = button.textContent;
    button.textContent = label;
    button.disabled = true;
  } else {
    button.textContent = button.dataset.idleLabel || button.textContent;
    delete button.dataset.idleLabel;
  }
}

function setSource(path) {
  state.source = path || "";
  $("sourcePath").textContent = state.source || "No file selected";
  $("analyseBtn").disabled = !state.source;
  $("loadOriginalBtn").disabled = !state.source;
}

function setOutput(path) {
  state.output = path || "audio/converted";
  $("outputPath").textContent = state.output;
}

const presetValues = {
  balanced: { bassGain: 1, leadGain: 0.85, psgGain: 0.85, dacGain: 1 },
  clean: { bassGain: 1, leadGain: 0.7, psgGain: 0.35, dacGain: 0.45 },
  "dac-heavy": { bassGain: 1, leadGain: 0.8, psgGain: 0.7, dacGain: 1.65 },
  "lead-cut": { bassGain: 1.2, leadGain: 0.35, psgGain: 0.75, dacGain: 1.1 },
};

function readMixControls() {
  return {
    bassGain: Number($("bassGain").value),
    leadGain: Number($("leadGain").value),
    psgGain: Number($("psgGain").value),
    dacGain: Number($("dacGain").value),
  };
}

function setSlider(id, value) {
  $(id).value = String(value);
  $(`${id}Value`).textContent = Number(value).toFixed(2);
}

function applyMixControls(values, preset = "custom") {
  $("previewPreset").value = preset;
  for (const [id, value] of Object.entries(values)) {
    setSlider(id, value);
  }
  state.presets = {
    ...state.presets,
    previewPreset: preset,
    bassGain: String($("bassGain").value),
    leadGain: String($("leadGain").value),
    psgGain: String($("psgGain").value),
    dacGain: String($("dacGain").value),
  };
}

function currentSettings() {
  return {
    source: state.source || "",
    output_dir: state.output || "audio/converted",
    instrument_bank: state.bankPath || "",
    presets: state.presets || {},
  };
}

async function saveSettings() {
  if (!state.settingsLoaded) return;
  try {
    await invoke("save_settings", { settings: currentSettings() });
  } catch (err) {
    setLog(`Settings save failed:\n${err}`);
  }
}

async function loadSettings() {
  try {
    const settings = await invoke("load_settings");
    state.settingsLoaded = true;
    state.presets = { ...state.presets, ...(settings?.presets || {}) };
    applyMixControls(
      {
        bassGain: Number(state.presets.bassGain || 1),
        leadGain: Number(state.presets.leadGain || 0.85),
        psgGain: Number(state.presets.psgGain || 0.85),
        dacGain: Number(state.presets.dacGain || 1),
      },
      state.presets.previewPreset || "balanced",
    );
    setOutput(settings?.output_dir || "audio/converted");
    if (settings?.source) {
      setSource(settings.source);
    }
    if (settings?.instrument_bank) {
      try {
        const result = await invoke("load_instrument_bank", { path: settings.instrument_bank });
        setBank(result);
        setLog(`Loaded settings.\n${settings.instrument_bank}`);
      } catch (err) {
        state.bankPath = settings.instrument_bank;
        $("bankPath").textContent = settings.instrument_bank;
        setLog(`Loaded settings, but bank load failed:\n${err}`);
      }
    } else {
      setLog("Ready.");
    }
  } catch (err) {
    state.settingsLoaded = true;
    setLog(`Settings load failed:\n${err}`);
  }
}

function setSummary(summary) {
  state.summary = summary;
  $("frames").textContent = summary ? summary.frames : "-";
  $("activeFrames").textContent = summary ? summary.active_frames : "-";
  $("runtimeEvents").textContent = summary ? summary.runtime_events : "-";
  $("dacChunks").textContent = summary ? summary.dac_chunks : "-";
  $("musicKey").textContent = summary?.key || "-";
  $("musicBpm").textContent = summary?.bpm ? Math.round(summary.bpm).toString() : "-";
  $("musicNotes").textContent = summary ? `${summary.bass_notes || 0}/${summary.lead_notes || 0}` : "-";
  $("musicDrums").textContent = summary ? summary.drum_events || 0 : "-";
  $("musicLoop").textContent =
    summary?.loop_end ? `${summary.loop_start.toFixed(1)}-${summary.loop_end.toFixed(1)}s` : "-";
  $("loadDacBtn").disabled = !summary?.dac_preview;
  $("loadArrangementBtn").disabled = !summary?.arrangement;
  $("loadNoteBtn").disabled = !(summary?.render_plan || summary?.note_arrangement);
  $("loadRuntimeBtn").disabled = !summary?.runtime_preview;

  if (!summary) {
    $("exportList").textContent = "No analysis yet.";
    return;
  }

  $("exportList").innerHTML = "";
  for (const [label, value] of [
    ["Bundle", summary.bundle_dir],
    ["Arrangement", summary.arrangement],
    ["Note Arrangement", summary.note_arrangement],
    ["Render Plan", summary.render_plan],
    ["Preview Report", summary.preview_report],
    ["Import", summary.import_metadata],
    ["DAC Preview", summary.dac_preview],
    ["Runtime Preview", summary.runtime_preview],
  ]) {
    const row = document.createElement("div");
    row.textContent = `${label}: ${value}`;
    $("exportList").appendChild(row);
  }
}

function setBank(result) {
  state.bank = result?.manifest || null;
  state.bankPath = result?.manifest_path || "";
  state.selectedInstrument = null;
  $("bankPath").textContent = state.bankPath || "No bank selected";
  $("bankCount").textContent = state.bank ? state.bank.instrument_count : "-";
  $("bankReview").textContent = state.bank ? state.bank.license_review : "-";
  $("previewInstrumentBtn").disabled = true;

  const list = $("instrumentList");
  list.innerHTML = "";
  if (!state.bank?.instruments?.length) {
    list.textContent = "No instruments loaded.";
    return;
  }

  for (const instrument of state.bank.instruments) {
    const row = document.createElement("button");
    row.className = "instrumentRow";
    row.type = "button";
    row.title = instrument.file;

    const main = document.createElement("strong");
    main.textContent = instrument.name;
    const meta = document.createElement("span");
    meta.textContent = `${instrument.chip} / ${instrument.category}`;

    row.append(main, meta);
    row.addEventListener("click", () => {
      state.selectedInstrument = instrument;
      $("previewInstrumentBtn").disabled = false;
      document
        .querySelectorAll(".instrumentRow.active")
        .forEach((node) => node.classList.remove("active"));
      row.classList.add("active");
      setLog(
        [
          `Instrument: ${instrument.name}`,
          `Chip: ${instrument.chip}`,
          `Category: ${instrument.category}`,
          `Source: ${instrument.source}`,
          `File: ${instrument.file}`,
        ].join("\n"),
      );
    });
    list.appendChild(row);
  }
}

async function previewInstrument() {
  const instrument = state.selectedInstrument;
  if (!instrument?.file) return;
  const player = $("instrumentPlayer");
  setBusy("instrumentProgress", "previewInstrumentBtn", true, "Rendering...");
  setLog(`Rendering instrument preview...\n${instrument.file}`);
  try {
    await paint();
    player.src = await invoke("instrument_preview_data_url", { path: instrument.file });
    await player.play();
    setLog(`Playing instrument preview:\n${instrument.name}\n${instrument.file}`);
  } catch (err) {
    setLog(`Instrument preview failed:\n${err}`);
  } finally {
    setBusy("instrumentProgress", "previewInstrumentBtn", false);
    $("previewInstrumentBtn").disabled = !state.selectedInstrument;
  }
}

async function chooseAudio() {
  try {
    const path = await invoke("pick_audio_file");
    if (path) {
      setSource(path);
      setSummary(null);
      setLog(`Selected source:\n${path}`);
      await saveSettings();
    }
  } catch (err) {
    setLog(`Open failed:\n${err}`);
  }
}

async function openBank() {
  try {
    const path = await invoke("pick_instrument_bank");
    if (!path) return;
    const result = await invoke("load_instrument_bank", { path });
    setBank(result);
    setLog(`${result.log}\n${result.manifest_path}`);
    await saveSettings();
  } catch (err) {
    setLog(`Bank load failed:\n${err}`);
  }
}

async function importBank() {
  try {
    const inputDir = await invoke("pick_instrument_dir");
    if (!inputDir) return;
    setLog(`Importing instruments...\n${inputDir}`);
    const result = await invoke("import_instrument_dir", {
      inputDir,
      outputDir: state.output,
    });
    setBank(result);
    setLog(result.log || `Imported bank:\n${result.manifest_path}`);
    await saveSettings();
  } catch (err) {
    setLog(`Instrument import failed:\n${err}`);
  }
}

async function chooseOutput() {
  try {
    const path = await invoke("pick_output_dir");
    if (path) {
      setOutput(path);
      setLog(`Selected output folder:\n${path}`);
      await saveSettings();
    }
  } catch (err) {
    setLog(`Output picker failed:\n${err}`);
  }
}

async function analyse() {
  if (!state.source) return;
  setBusy("analysisProgress", "analyseBtn", true, "Analysing...");
  setLog("Analysing...");
  try {
    await paint();
    const result = await invoke("analyse_audio", {
      input: state.source,
      outputDir: state.output,
    });
    setSummary(result.summary);
    setLog(result.log || "Analysis complete.");
    await saveSettings();
  } catch (err) {
    setLog(`Analysis failed:\n${err}`);
  } finally {
    setBusy("analysisProgress", "analyseBtn", false);
    $("analyseBtn").disabled = !state.source;
  }
}

async function loadPlayer(playerId, path, label, progressId = "", buttonId = "") {
  if (!path) return;
  const player = $(playerId);
  if (progressId && buttonId) setBusy(progressId, buttonId, true, "Loading...");
  setLog(`Loading ${label}...\n${path}`);
  try {
    await paint();
    player.src = await invoke("audio_data_url", { path });
    await player.play();
    setLog(`Playing ${label}:\n${path}`);
  } catch (err) {
    setLog(`${label} playback failed:\n${err}`);
  } finally {
    if (progressId && buttonId) {
      setBusy(progressId, buttonId, false);
      $(buttonId).disabled = false;
    }
  }
}

function loadOriginal() {
  if (!state.source) return;
  loadPlayer("originalPlayer", state.source, "original");
}

function loadDac() {
  if (!state.summary?.dac_preview) return;
  loadPlayer("dacPlayer", state.summary.dac_preview, "DAC preview", "dacProgress", "loadDacBtn");
}

function loadRuntime() {
  if (!state.summary?.runtime_preview) return;
  loadPlayer(
    "runtimePlayer",
    state.summary.runtime_preview,
    "runtime preview",
    "runtimeProgress",
    "loadRuntimeBtn",
  );
}

async function loadArrangement() {
  if (!state.summary?.arrangement) return;
  const player = $("arrangementPlayer");
  setBusy("arrangementProgress", "loadArrangementBtn", true, "Rendering...");
  setLog(`Rendering arrangement preview...\n${state.summary.arrangement}`);
  try {
    await paint();
    player.src = await invoke("arrangement_preview_data_url", {
      path: state.summary.arrangement,
      bankPath: state.bankPath || "",
    });
    await player.play();
    setLog(`Playing arrangement preview:\n${state.summary.arrangement}`);
  } catch (err) {
    setLog(`Arrangement preview failed:\n${err}`);
  } finally {
    setBusy("arrangementProgress", "loadArrangementBtn", false);
    $("loadArrangementBtn").disabled = !state.summary?.arrangement;
  }
}

async function loadNote() {
  const previewPath = state.summary?.render_plan || state.summary?.note_arrangement;
  if (!previewPath) return;
  const player = $("notePlayer");
  setBusy("noteProgress", "loadNoteBtn", true, "Rendering...");
  setLog(`Rendering note preview...\n${previewPath}`);
  try {
    await paint();
    player.src = await invoke("note_preview_data_url", {
      path: previewPath,
      bankPath: state.bankPath || "",
      ...readMixControls(),
    });
    await player.play();
    setLog(`Playing note preview:\n${previewPath}`);
  } catch (err) {
    setLog(`Note preview failed:\n${err}`);
  } finally {
    setBusy("noteProgress", "loadNoteBtn", false);
    $("loadNoteBtn").disabled = !(state.summary?.render_plan || state.summary?.note_arrangement);
  }
}

$("openBtn").addEventListener("click", chooseAudio);
$("outputBtn").addEventListener("click", chooseOutput);
$("openBankBtn").addEventListener("click", openBank);
$("importBankBtn").addEventListener("click", importBank);
$("analyseBtn").addEventListener("click", analyse);
$("loadOriginalBtn").addEventListener("click", loadOriginal);
$("loadDacBtn").addEventListener("click", loadDac);
$("loadRuntimeBtn").addEventListener("click", loadRuntime);
$("loadArrangementBtn").addEventListener("click", loadArrangement);
$("loadNoteBtn").addEventListener("click", loadNote);
$("previewInstrumentBtn").addEventListener("click", previewInstrument);
$("previewPreset").addEventListener("change", async () => {
  const preset = $("previewPreset").value;
  if (presetValues[preset]) {
    applyMixControls(presetValues[preset], preset);
    await saveSettings();
  }
});
for (const id of ["bassGain", "leadGain", "psgGain", "dacGain"]) {
  $(id).addEventListener("input", () => {
    setSlider(id, $(id).value);
    $("previewPreset").value = "custom";
    state.presets.previewPreset = "custom";
    state.presets[id] = String($(id).value);
  });
  $(id).addEventListener("change", saveSettings);
}

setOutput(state.output);
setSummary(null);
setBank(null);
applyMixControls(presetValues.balanced, "balanced");
loadSettings();
