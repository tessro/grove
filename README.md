# grove
Where thoughts grow between minds.

## Building

```bash
make build            # build everything (server + frontend)
make build-server     # build only the Rust backend
make build-frontend   # build only the Vite frontend
```

## Installing

```bash
make install
```

This builds everything and installs to `~/.local`:

- `~/.local/bin/grove` — server binary
- `~/.local/share/grove/frontend/dist/` — frontend assets
- `~/.config/systemd/user/grove.service` — systemd unit

## Running as a systemd user service

1. Create an environment file at `~/.config/grove/env`:

```bash
mkdir -p ~/.config/grove
cat > ~/.config/grove/env <<'EOF'
ANTHROPIC_API_KEY=sk-ant-...
EOF
```

2. Enable and start the service:

```bash
systemctl --user daemon-reload
systemctl --user enable grove.service
systemctl --user start grove.service
```

3. To have the service start at boot (without requiring login):

```bash
loginctl enable-linger $USER
```

4. Check status and logs:

```bash
systemctl --user status grove.service
journalctl --user -u grove.service -f
```
