const { invoke } = window.__TAURI__.core;

const state = {
  source: "",
  output: "audio/converted",
  summary: null,
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
$("analyseBtn").addEventListener("click", analyse);
$("loadOriginalBtn").addEventListener("click", loadOriginal);
$("loadDacBtn").addEventListener("click", loadDac);

setOutput(state.output);
setSummary(null);
