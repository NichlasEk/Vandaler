GDK ?= /home/nichlas/SGDK
PREFIX ?= ./tools/m68k-elf-
SGDK_WINE ?= 1

.PHONY: assets assets-preview audio-test audio-test-generated audio-lab audio-lab-headless audio-analyse-test audio-generated-loop

assets:
	tools/assets.sh build

assets-preview:
	tools/assets.sh preview

audio-test:
	$(MAKE) clean-release
	$(MAKE) EXTRA_FLAGS="$(EXTRA_FLAGS) -DVAND_AUDIO_TEST_ROM"
	cp out/release/rom.bin out/audio-test.bin
	$(MAKE) clean-release
	$(MAKE)

audio-test-generated:
	$(MAKE) clean-release
	$(MAKE) EXTRA_FLAGS="$(EXTRA_FLAGS) -DVAND_AUDIO_TEST_ROM -DVAND_AUDIO_GENERATED"
	cp out/release/rom.bin out/audio-test.bin
	$(MAKE) clean-release
	$(MAKE)

audio-lab: audio-test
	cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- render-rom out/audio-test.bin --wav out/audio-test.wav --report out/audio-test-report.json --seconds 5

audio-lab-headless:
	cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- test-rom --seconds 5

audio-analyse-test:
	cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- analyse-wav out/audio-test.wav --out out/audio-test-analysis.vand-audio.json

audio-generated-loop:
	cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- analyse-wav out/audio-test.wav --out audio/converted/audio-test.vand-audio/arrangement.vand-audio.json --install-sgdk
	$(MAKE) audio-test-generated
	cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- render-rom out/audio-test.bin --wav out/audio-test-generated.wav --report out/audio-test-generated-report.json --seconds 5

.DEFAULT_GOAL := all

include $(GDK)/makefile.gen
