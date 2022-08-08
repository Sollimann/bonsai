.PHONY: install-dev-deps dry-run publish

install-dev-deps:
	sudo apt-get update && sudo apt-get install libudev-dev pkg-config librust-alsa-sys-dev

dry-run:
	cargo publish --package bonsai-bt --dry-run

publish:
	cargo publish --package bonsai-bt
