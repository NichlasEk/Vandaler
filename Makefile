GDK ?= /home/nichlas/SGDK
PREFIX ?= ./tools/m68k-elf-
SGDK_WINE ?= 1

.PHONY: assets assets-preview audio-test

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

.DEFAULT_GOAL := all

include $(GDK)/makefile.gen
