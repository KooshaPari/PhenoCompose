# LaunchAgent plist for PhenoCompose (`pheno-compose-driver` service-mode)

This directory ships the **source-of-truth plist** that
`pheno-macos-shell::launchd::install_plist` writes into
`~/Library/LaunchAgents/`.

| Field          | Value                                                    |
| -------------- | -------------------------------------------------------- |
| `Label`        | `ai.phenotype.pheno-compose-driver`                      |
| `Program`      | `pheno-compose-driver` (built via `cargo build --release`) |
| `RunAtLoad`    | `true` (start at user login)                             |
| `KeepAlive`    | restart on crash, exit-on-success                        |
| `ProcessType`  | `Background`                                             |
| `Logs`         | `/tmp/pheno-compose-driver.{out,err}.log`                |

## Install (operator runbook)

```bash
# 1. Build the driver
cargo build --release -p pheno-compose-driver

# 2. Install via the scaffold API
cargo run -p pheno-macos-shell --example install_plist -- /usr/local/bin/pheno-compose-driver

# 3. Bootstrap the agent
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/ai.phenotype.pheno-compose-driver.plist
launchctl enable gui/$(id -u)/ai.phenotype.pheno-compose-driver
launchctl kickstart -k gui/$(id -u)/ai.phenotype.pheno-compose-driver
```

## Uninstall

```bash
launchctl bootout gui/$(id -u)/ai.phenotype.pheno-compose-driver
rm ~/Library/LaunchAgents/ai.phenotype.pheno-compose-driver.plist
```

## Scaffold status

PILLAR-TAXONOMY-v2 **L130**. The `plist` body is rendered programmatically
by `src/launchd.rs::render_plist()` — this stub file is a copy-paste
placeholder until `cargo install --path mobile/macos-shell` lands.