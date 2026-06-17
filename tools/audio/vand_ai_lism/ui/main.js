const { invoke } = window.__TAURI__.core;

const state = {
  source: "",
  output: "audio/converted",
  summary: null,
  bank: null,
  bankPath: "",
  selectedInstrument: null,
  debug: null,
  debugHitboxes: [],
  selectedDebugNote: null,
  regionStopTimer: null,
  aiBackend: "off",
  aiHome: "/home/nichlas/ai",
  aiToolDir: "",
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
  $("originalPlayer").removeAttribute("src");
  $("originalPlayer").load();
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
    previewSource: $("previewSource").value || "rust",
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
    ai_backend: state.aiBackend || "off",
    ai_home: state.aiHome || "/home/nichlas/ai",
    ai_tool_dir: state.aiToolDir || "",
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
    state.aiBackend = settings?.ai_backend || "off";
    state.aiHome = settings?.ai_home || "/home/nichlas/ai";
    state.aiToolDir = settings?.ai_tool_dir || "";
    $("aiBackend").value = state.aiBackend;
    state.presets = { ...state.presets, ...(settings?.presets || {}) };
    $("previewSource").value = state.presets.previewSource || "rust";
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
  for (const id of ["dacPlayer", "arrangementPlayer", "notePlayer", "aiPlayer", "runtimePlayer"]) {
    $(id).removeAttribute("src");
    $(id).load();
  }
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
  $("loadAiBtn").disabled = !summary?.ai_notes;
  $("loadRuntimeBtn").disabled = !summary?.runtime_preview;
  $("reloadDebugBtn").disabled = !(summary?.arrangement && summary?.render_plan);

  if (!summary) {
    $("exportList").textContent = "No analysis yet.";
    clearDebug();
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
    ["AI Manifest", summary.ai_manifest],
    ["AI Notes", summary.ai_notes],
  ]) {
    if (!value) continue;
    const row = document.createElement("div");
    row.textContent = `${label}: ${value}`;
    $("exportList").appendChild(row);
  }
  void loadDebug();
}

async function readJson(path) {
  if (!path) return null;
  const text = await invoke("read_text_file", { path });
  return JSON.parse(text);
}

function clearDebug(message = "No debug data.") {
  state.debug = null;
  state.debugHitboxes = [];
  state.selectedDebugNote = null;
  const canvas = $("debugCanvas");
  const ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#090b10";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#6f7788";
  ctx.font = "24px sans-serif";
  ctx.fillText(message, 28, 52);
  $("trackSummary").textContent = message;
  $("noteInspector").innerHTML = "";
  $("selectedNote").textContent = "No note selected.";
  setRegion(0, 4);
  enableRegionButtons(false);
  $("debugMeta").innerHTML = [
    ["Grid", "-"],
    ["Offset", "-"],
    ["FM Notes", "-"],
    ["PSG Notes", "-"],
  ]
    .map(([label, value]) => `<div><span>${label}</span><strong>${value}</strong></div>`)
    .join("");
}

function clamp01(value) {
  return Math.max(0, Math.min(1, Number(value) || 0));
}

function trackColor(role) {
  return {
    bass: "#ff6a3d",
    lead: "#f1b24a",
    pad: "#5fd38d",
    chord: "#49a6ff",
    counter: "#d783ff",
    psg_lead: "#fff1a8",
    psg_counter: "#9fe7ff",
    psg_bass: "#ff8fba",
    ai_basic_pitch: "#65ffd2",
  }[role] || "#d9dfec";
}

function compactTime(frames, hop, sampleRate) {
  if (!sampleRate || !hop) return "0.00s";
  return `${((frames * hop) / sampleRate).toFixed(2)}s`;
}

function frameToSeconds(frames, hop, sampleRate) {
  if (!sampleRate || !hop) return 0;
  return (frames * hop) / sampleRate;
}

function aiTracksForRenderPlan(aiNotes, renderPlan) {
  if (!aiNotes?.tracks?.length || !renderPlan?.sample_rate || !renderPlan?.hop) return [];
  const framesPerSecond = renderPlan.sample_rate / renderPlan.hop;
  return aiNotes.tracks.map((track) => ({
    role: track.role || "ai_basic_pitch",
    chip: track.chip || "ai",
    channel: track.channel ?? 0,
    events: (track.events || []).map((event) => ({
      ...event,
      start_frame: Math.round((event.start_time || 0) * framesPerSecond),
      frames: Math.max(1, Math.round((event.duration || 0.01) * framesPerSecond)),
      velocity: Number(event.velocity || 0),
    })),
  }));
}

function collectTrackStats(renderPlan, aiNotes = null) {
  const tracks = [...(renderPlan?.tracks || []), ...aiTracksForRenderPlan(aiNotes, renderPlan)];
  return tracks.map((track) => {
    const events = track.events || [];
    const activeFrames = events.reduce((sum, event) => sum + (event.frames || 0), 0);
    const avgVelocity = events.length
      ? events.reduce((sum, event) => sum + (event.velocity || 0), 0) / events.length
      : 0;
    return {
      role: track.role,
      chip: track.chip,
      channel: track.channel,
      events,
      count: events.length,
      activeFrames,
      avgVelocity,
    };
  });
}

function drawDebugCanvas(arrangement, renderPlan, aiNotes = null) {
  const canvas = $("debugCanvas");
  const ctx = canvas.getContext("2d");
  const width = canvas.width;
  const height = canvas.height;
  const frames = arrangement?.events || [];
  const tracks = collectTrackStats(renderPlan, aiNotes);
  const aiTotalFrames = aiTracksForRenderPlan(aiNotes, renderPlan)
    .flatMap((track) => track.events || [])
    .reduce((max, event) => Math.max(max, (event.start_frame || 0) + (event.frames || 1)), 0);
  const totalFrames = Math.max(renderPlan?.total_frames || arrangement?.events?.length || 1, aiTotalFrames, 1);
  const topHeight = 88;
  const rowHeight = Math.max(16, Math.floor((height - topHeight - 18) / Math.max(1, tracks.length)));
  const selected = state.selectedDebugNote;
  state.debugHitboxes = [];

  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = "#090b10";
  ctx.fillRect(0, 0, width, height);

  ctx.strokeStyle = "rgba(255,255,255,0.08)";
  ctx.lineWidth = 1;
  for (let i = 0; i <= 8; i += 1) {
    const x = Math.round((i / 8) * width);
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
  }

  for (const frame of frames) {
    const x = Math.floor((frame.index / totalFrames) * width);
    const nextX = Math.max(x + 1, Math.floor(((frame.index + 1) / totalFrames) * width));
    const rms = clamp01(frame.rms);
    const transient = clamp01(frame.transient);
    const noise = clamp01(frame.noise_amp);
    const barHeight = Math.max(1, Math.round(rms * topHeight));
    ctx.fillStyle = `rgba(73, 166, 255, ${0.10 + rms * 0.38})`;
    ctx.fillRect(x, topHeight - barHeight, nextX - x, barHeight);
    if (transient > 0.18) {
      ctx.fillStyle = `rgba(255, 106, 61, ${Math.min(0.80, transient)})`;
      ctx.fillRect(x, 0, Math.max(1, nextX - x), topHeight);
    } else if (noise > 0.18) {
      ctx.fillStyle = `rgba(241, 178, 74, ${Math.min(0.30, noise * 0.45)})`;
      ctx.fillRect(x, 0, Math.max(1, nextX - x), topHeight);
    }
  }

  ctx.fillStyle = "#b9c2d2";
  ctx.font = "14px sans-serif";
  ctx.fillText("RMS / transient / noise", 12, 20);

  tracks.forEach((track, index) => {
    const y = topHeight + 10 + index * rowHeight;
    ctx.fillStyle = index % 2 === 0 ? "rgba(255,255,255,0.035)" : "rgba(255,255,255,0.015)";
    ctx.fillRect(0, y, width, rowHeight - 2);
    ctx.fillStyle = "#8f99ad";
    ctx.font = "12px sans-serif";
    ctx.fillText(`${track.role} ${track.chip}${track.channel}`, 10, y + 12);

    ctx.fillStyle = trackColor(track.role);
    for (const event of track.events) {
      const start = Math.floor(((event.start_frame || 0) / totalFrames) * width);
      const end = Math.max(
        start + 2,
        Math.floor((((event.start_frame || 0) + (event.frames || 1)) / totalFrames) * width),
      );
      const velocity = clamp01(event.velocity);
      const noteY = y + 3 + Math.round((1 - velocity) * Math.max(1, rowHeight - 9));
      ctx.globalAlpha = 0.35 + velocity * 0.65;
      ctx.fillRect(start, noteY, end - start, Math.max(3, Math.round(rowHeight * 0.32)));
      ctx.globalAlpha = 1;
      if (
        selected &&
        selected.role === track.role &&
        selected.channel === track.channel &&
        selected.start_frame === event.start_frame &&
        selected.midi === event.midi
      ) {
        ctx.strokeStyle = "#ffffff";
        ctx.lineWidth = 2;
        ctx.strokeRect(start - 1, noteY - 2, end - start + 2, Math.max(6, Math.round(rowHeight * 0.38)));
      }
      state.debugHitboxes.push({
        x1: start,
        x2: end,
        y1: y,
        y2: y + rowHeight,
        note: {
          role: track.role,
          chip: track.chip,
          channel: track.channel,
          ...event,
        },
      });
    }
  });
}

function setRegion(start, end) {
  const safeStart = Math.max(0, Number(start) || 0);
  const safeEnd = Math.max(safeStart + 0.15, Number(end) || safeStart + 4);
  $("abStart").value = safeStart.toFixed(2);
  $("abEnd").value = safeEnd.toFixed(2);
}

function setSelectedDebugNote(note) {
  state.selectedDebugNote = note;
  if (!note) {
    $("selectedNote").textContent = "No note selected.";
    return;
  }
  const plan = state.debug?.renderPlan;
  const start = frameToSeconds(note.start_frame || 0, plan?.hop, plan?.sample_rate);
  const end = frameToSeconds((note.start_frame || 0) + (note.frames || 1), plan?.hop, plan?.sample_rate);
  setRegion(start, end);
  $("selectedNote").textContent = [
    `${note.role} ${note.chip}${note.channel}`,
    `midi ${note.midi ?? "-"} · ${Math.round(note.hz || 0)}Hz · v${Number(note.velocity || 0).toFixed(2)}`,
    `${start.toFixed(2)}-${end.toFixed(2)}s · ${note.frames || 0} frames`,
  ].join("\n");
  if (state.debug) drawDebugCanvas(state.debug.arrangement, state.debug.renderPlan, state.debug.aiNotes);
}

function pickDebugNote(event) {
  if (!state.debugHitboxes.length) return;
  const canvas = $("debugCanvas");
  const rect = canvas.getBoundingClientRect();
  const x = ((event.clientX - rect.left) / rect.width) * canvas.width;
  const y = ((event.clientY - rect.top) / rect.height) * canvas.height;
  const hit = state.debugHitboxes
    .filter((box) => x >= box.x1 && x <= box.x2 && y >= box.y1 && y <= box.y2)
    .sort((a, b) => (a.x2 - a.x1) - (b.x2 - b.x1))[0];
  if (hit) {
    setSelectedDebugNote(hit.note);
  }
}

function hoverDebugNote(event) {
  const canvas = $("debugCanvas");
  if (!state.debugHitboxes.length) {
    canvas.title = "";
    canvas.style.cursor = "crosshair";
    return;
  }
  const rect = canvas.getBoundingClientRect();
  const x = ((event.clientX - rect.left) / rect.width) * canvas.width;
  const y = ((event.clientY - rect.top) / rect.height) * canvas.height;
  const hit = state.debugHitboxes.find((box) => x >= box.x1 && x <= box.x2 && y >= box.y1 && y <= box.y2);
  if (!hit) {
    canvas.title = "";
    canvas.style.cursor = "crosshair";
    return;
  }
  canvas.style.cursor = "pointer";
  canvas.title = `${hit.note.role} midi ${hit.note.midi ?? "-"} ${Math.round(hit.note.hz || 0)}Hz`;
}

function renderTrackSummary(renderPlan, aiNotes = null) {
  const list = $("trackSummary");
  list.innerHTML = "";
  for (const track of collectTrackStats(renderPlan, aiNotes)) {
    const card = document.createElement("article");
    const title = document.createElement("strong");
    title.textContent = `${track.role} · ${track.chip}${track.channel}`;
    title.style.color = trackColor(track.role);
    const meta = document.createElement("span");
    meta.textContent = `${track.count} notes · avg ${track.avgVelocity.toFixed(2)}`;
    card.append(title, meta);
    list.appendChild(card);
  }
}

function renderNoteInspector(renderPlan, aiNotes = null) {
  const box = $("noteInspector");
  box.innerHTML = "";
  const notes = [...(renderPlan?.tracks || []), ...aiTracksForRenderPlan(aiNotes, renderPlan)]
    .flatMap((track) =>
      (track.events || []).map((event) => ({
        role: track.role,
        chip: track.chip,
        channel: track.channel,
        ...event,
      })),
    )
    .sort((a, b) => (a.start_frame || 0) - (b.start_frame || 0))
    .slice(0, 80);
  for (const note of notes) {
    const row = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = `${note.role} ${note.midi ?? "-"} ${Math.round(note.hz || 0)}Hz`;
    title.style.color = trackColor(note.role);
    const meta = document.createElement("span");
    meta.textContent = `${compactTime(note.start_frame || 0, renderPlan?.hop, renderPlan?.sample_rate)} · ${note.frames || 0}f · v${Number(note.velocity || 0).toFixed(2)}`;
    row.append(title, meta);
    row.addEventListener("click", () => setSelectedDebugNote(note));
    box.appendChild(row);
  }
}

function renderDebugMeta(report, renderPlan, aiNotes = null) {
  const tracks = collectTrackStats(renderPlan, aiNotes);
  const fmNotes = tracks
    .filter((track) => track.chip === "ym2612")
    .reduce((sum, track) => sum + track.count, 0);
  const psgNotes = tracks
    .filter((track) => track.chip === "psg")
    .reduce((sum, track) => sum + track.count, 0);
  const aiNoteCount = tracks
    .filter((track) => track.chip === "ai")
    .reduce((sum, track) => sum + track.count, 0);
  $("debugMeta").innerHTML = [
    ["Grid", report?.grid_frames ?? renderPlan?.tempo?.grid_frames ?? "-"],
    ["Offset", report?.grid_offset_frame ?? renderPlan?.tempo?.grid_offset_frame ?? "-"],
    ["FM Notes", fmNotes],
    ["PSG Notes", psgNotes],
    ["AI Notes", aiNoteCount || "-"],
  ]
    .map(([label, value]) => `<div><span>${label}</span><strong>${value}</strong></div>`)
    .join("");
}

async function loadDebug() {
  if (!state.summary?.arrangement || !state.summary?.render_plan) {
    clearDebug();
    return;
  }
  $("reloadDebugBtn").disabled = true;
  try {
    const [arrangement, renderPlan, report, aiNotes] = await Promise.all([
      readJson(state.summary.arrangement),
      readJson(state.summary.render_plan),
      readJson(state.summary.preview_report).catch(() => null),
      state.summary.ai_notes ? readJson(state.summary.ai_notes).catch(() => null) : Promise.resolve(null),
    ]);
    state.debug = { arrangement, renderPlan, report, aiNotes };
    state.selectedDebugNote = null;
    renderDebugMeta(report, renderPlan, aiNotes);
    drawDebugCanvas(arrangement, renderPlan, aiNotes);
    renderTrackSummary(renderPlan, aiNotes);
    renderNoteInspector(renderPlan, aiNotes);
    enableRegionButtons(true);
  } catch (err) {
    clearDebug("Debug load failed.");
    setLog(`Debug load failed:\n${err}`);
  } finally {
    $("reloadDebugBtn").disabled = !(state.summary?.arrangement && state.summary?.render_plan);
  }
}

function enableRegionButtons(enabled) {
  for (const id of ["playOriginalRegionBtn", "playPreviewRegionBtn", "playBothRegionBtn"]) {
    $(id).disabled = !enabled;
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
      aiBackend: state.aiBackend || "off",
      aiHome: state.aiHome || "/home/nichlas/ai",
      aiToolDir: state.aiToolDir || "",
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

async function ensureOriginalLoaded() {
  const player = $("originalPlayer");
  if (!state.source) throw new Error("No original source selected.");
  if (!player.src) {
    player.src = await invoke("audio_data_url", { path: state.source });
  }
  return player;
}

async function ensureNoteLoaded() {
  const player = $("notePlayer");
  const previewPath = state.summary?.render_plan || state.summary?.note_arrangement;
  if (!previewPath) throw new Error("No note preview available.");
  if (!player.src) {
    player.src = await invoke("note_preview_data_url", {
      path: previewPath,
      bankPath: state.bankPath || "",
      ...readMixControls(),
    });
  }
  return player;
}

function stopRegionPlayback() {
  if (state.regionStopTimer) {
    window.clearTimeout(state.regionStopTimer);
    state.regionStopTimer = null;
  }
}

async function playRegion(players) {
  stopRegionPlayback();
  const start = Math.max(0, Number($("abStart").value) || 0);
  const end = Math.max(start + 0.15, Number($("abEnd").value) || start + 4);
  for (const player of players) {
    player.pause();
    player.currentTime = Math.min(start, Number.isFinite(player.duration) ? player.duration : start);
  }
  await Promise.all(players.map((player) => player.play()));
  state.regionStopTimer = window.setTimeout(() => {
    for (const player of players) player.pause();
    state.regionStopTimer = null;
  }, Math.max(150, (end - start) * 1000));
}

async function playOriginalRegion() {
  try {
    const player = await ensureOriginalLoaded();
    await playRegion([player]);
    setLog(`Playing original region ${$("abStart").value}-${$("abEnd").value}s`);
  } catch (err) {
    setLog(`Original region failed:\n${err}`);
  }
}

async function playPreviewRegion() {
  try {
    const player = await ensureNoteLoaded();
    await playRegion([player]);
    setLog(`Playing note preview region ${$("abStart").value}-${$("abEnd").value}s`);
  } catch (err) {
    setLog(`Preview region failed:\n${err}`);
  }
}

async function playBothRegion() {
  try {
    const original = await ensureOriginalLoaded();
    const preview = await ensureNoteLoaded();
    await playRegion([original, preview]);
    setLog(`Playing A/B region ${$("abStart").value}-${$("abEnd").value}s`);
  } catch (err) {
    setLog(`A/B region failed:\n${err}`);
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
  const source = $("previewSource").value || "rust";
  setBusy("noteProgress", "loadNoteBtn", true, "Rendering...");
  setLog(`Rendering ${source} note preview...\n${previewPath}`);
  try {
    await paint();
    if (source === "ai") {
      if (!state.summary?.ai_notes) throw new Error("No AI notes available.");
      player.src = await invoke("ai_note_preview_data_url", {
        path: state.summary.ai_notes,
        bankPath: state.bankPath || "",
        leadGain: Number($("leadGain").value),
      });
    } else if (source === "hybrid") {
      if (!state.summary?.ai_notes) throw new Error("No AI notes available.");
      player.src = await invoke("hybrid_note_preview_data_url", {
        path: previewPath,
        aiPath: state.summary.ai_notes,
        bankPath: state.bankPath || "",
        ...readMixControls(),
      });
    } else {
      player.src = await invoke("note_preview_data_url", {
        path: previewPath,
        bankPath: state.bankPath || "",
        ...readMixControls(),
      });
    }
    await player.play();
    setLog(`Playing ${source} note preview:\n${previewPath}`);
  } catch (err) {
    setLog(`Note preview failed:\n${err}`);
  } finally {
    setBusy("noteProgress", "loadNoteBtn", false);
    $("loadNoteBtn").disabled = !(state.summary?.render_plan || state.summary?.note_arrangement);
  }
}

async function loadAi() {
  if (!state.summary?.ai_notes) return;
  const player = $("aiPlayer");
  setBusy("aiProgress", "loadAiBtn", true, "Rendering...");
  setLog(`Rendering AI preview...\n${state.summary.ai_notes}`);
  try {
    await paint();
    player.src = await invoke("ai_note_preview_data_url", {
      path: state.summary.ai_notes,
      bankPath: state.bankPath || "",
      leadGain: Number($("leadGain").value),
    });
    await player.play();
    setLog(`Playing AI preview:\n${state.summary.ai_notes}`);
  } catch (err) {
    setLog(`AI preview failed:\n${err}`);
  } finally {
    setBusy("aiProgress", "loadAiBtn", false);
    $("loadAiBtn").disabled = !state.summary?.ai_notes;
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
$("loadAiBtn").addEventListener("click", loadAi);
$("previewInstrumentBtn").addEventListener("click", previewInstrument);
$("reloadDebugBtn").addEventListener("click", loadDebug);
$("debugCanvas").addEventListener("click", pickDebugNote);
$("debugCanvas").addEventListener("mousemove", hoverDebugNote);
$("playOriginalRegionBtn").addEventListener("click", playOriginalRegion);
$("playPreviewRegionBtn").addEventListener("click", playPreviewRegion);
$("playBothRegionBtn").addEventListener("click", playBothRegion);
$("aiBackend").addEventListener("change", async () => {
  state.aiBackend = $("aiBackend").value || "off";
  await saveSettings();
});
$("previewPreset").addEventListener("change", async () => {
  const preset = $("previewPreset").value;
  if (presetValues[preset]) {
    applyMixControls(presetValues[preset], preset);
    await saveSettings();
  }
});
$("previewSource").addEventListener("change", async () => {
  state.presets.previewSource = $("previewSource").value || "rust";
  $("notePlayer").removeAttribute("src");
  $("notePlayer").load();
  await saveSettings();
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
clearDebug();
applyMixControls(presetValues.balanced, "balanced");
loadSettings();
