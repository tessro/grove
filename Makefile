PREFIX ?= $(HOME)/.local
FRONTEND_DIR = frontend

.PHONY: build build-server build-frontend install clean

build: build-server build-frontend

build-server:
	cargo build --release

build-frontend:
	cd $(FRONTEND_DIR) && npm install && npm run build

install: build
	-systemctl --user stop grove.service
	install -d $(PREFIX)/bin
	install -m 755 target/release/grove $(PREFIX)/bin/grove
	install -d $(PREFIX)/share/grove/frontend/dist
	cp -r $(FRONTEND_DIR)/dist/* $(PREFIX)/share/grove/frontend/dist/
	install -d $(HOME)/.config/systemd/user
	install -m 644 grove.service $(HOME)/.config/systemd/user/grove.service
	systemctl --user daemon-reload
	systemctl --user start grove.service

clean:
	cargo clean
	rm -rf $(FRONTEND_DIR)/dist
