GDK ?= /home/nichlas/SGDK
PREFIX ?= ./tools/m68k-elf-
SGDK_WINE ?= 1

.PHONY: assets assets-preview

assets:
	tools/assets.sh build

assets-preview:
	tools/assets.sh preview

.DEFAULT_GOAL := all

include $(GDK)/makefile.gen
