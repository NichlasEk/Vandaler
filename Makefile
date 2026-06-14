GDK ?= /home/nichlas/SGDK
PREFIX ?= ./tools/m68k-elf-
SGDK_WINE ?= 1

.PHONY: assets assets-preview audio-test audio-lab

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

audio-lab: audio-test
	cargo run --manifest-path tools/audio/oxide_probe/Cargo.toml -- render-rom out/audio-test.bin --wav out/audio-test.wav --seconds 5

.DEFAULT_GOAL := all

include $(GDK)/makefile.gen
