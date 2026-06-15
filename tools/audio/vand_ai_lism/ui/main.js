const { invoke } = window.__TAURI__.core;

const state = {
  source: "",
  output: "audio/converted",
  summary: null,
  bank: null,
  bankPath: "",
  selectedInstrument: null,
};

const $ = (id) => document.getElementById(id);

function setLog(message) {
  $("log").textContent = message;
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

function setSummary(summary) {
  state.summary = summary;
  $("frames").textContent = summary ? summary.frames : "-";
  $("activeFrames").textContent = summary ? summary.active_frames : "-";
  $("runtimeEvents").textContent = summary ? summary.runtime_events : "-";
  $("dacChunks").textContent = summary ? summary.dac_chunks : "-";
  $("loadDacBtn").disabled = !summary?.dac_preview;

  if (!summary) {
    $("exportList").textContent = "No analysis yet.";
    return;
  }

  $("exportList").innerHTML = "";
  for (const [label, value] of [
    ["Bundle", summary.bundle_dir],
    ["Arrangement", summary.arrangement],
    ["Import", summary.import_metadata],
    ["DAC Preview", summary.dac_preview],
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
  setLog(`Rendering instrument preview...\n${instrument.file}`);
  try {
    player.src = await invoke("instrument_preview_data_url", { path: instrument.file });
    await player.play();
    setLog(`Playing instrument preview:\n${instrument.name}\n${instrument.file}`);
  } catch (err) {
    setLog(`Instrument preview failed:\n${err}`);
  }
}

async function chooseAudio() {
  try {
    const path = await invoke("pick_audio_file");
    if (path) {
      setSource(path);
      setSummary(null);
      setLog(`Selected source:\n${path}`);
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
    }
  } catch (err) {
    setLog(`Output picker failed:\n${err}`);
  }
}

async function analyse() {
  if (!state.source) return;
  $("analyseBtn").disabled = true;
  setLog("Analysing...");
  try {
    const result = await invoke("analyse_audio", {
      input: state.source,
      outputDir: state.output,
    });
    setSummary(result.summary);
    setLog(result.log || "Analysis complete.");
  } catch (err) {
    setLog(`Analysis failed:\n${err}`);
  } finally {
    $("analyseBtn").disabled = !state.source;
  }
}

async function loadPlayer(playerId, path, label) {
  if (!path) return;
  const player = $(playerId);
  setLog(`Loading ${label}...\n${path}`);
  try {
    player.src = await invoke("audio_data_url", { path });
    await player.play();
    setLog(`Playing ${label}:\n${path}`);
  } catch (err) {
    setLog(`${label} playback failed:\n${err}`);
  }
}

function loadOriginal() {
  if (!state.source) return;
  loadPlayer("originalPlayer", state.source, "original");
}

function loadDac() {
  if (!state.summary?.dac_preview) return;
  loadPlayer("dacPlayer", state.summary.dac_preview, "DAC preview");
}

$("openBtn").addEventListener("click", chooseAudio);
$("outputBtn").addEventListener("click", chooseOutput);
$("openBankBtn").addEventListener("click", openBank);
$("importBankBtn").addEventListener("click", importBank);
$("analyseBtn").addEventListener("click", analyse);
$("loadOriginalBtn").addEventListener("click", loadOriginal);
$("loadDacBtn").addEventListener("click", loadDac);
$("previewInstrumentBtn").addEventListener("click", previewInstrument);

setOutput(state.output);
setSummary(null);
setBank(null);
