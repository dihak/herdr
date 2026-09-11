# herdr


<p align="center">
  <img src="assets/logo.png" alt="herdr" width="100" />
</p>

<p align="center">
  <a href="https://github.com/dihak/herdr">dihak/herdr fork</a> · <a href="#install">install</a> · <a href="https://herdr.dev/docs/quick-start/">upstream docs</a>
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-666666?labelColor=333333" alt="Apache 2.0 license" /></a>
  <a href="https://github.com/dihak/herdr/releases"><img src="https://img.shields.io/github/downloads/dihak/herdr/total?labelColor=333333&color=666666" alt="total GitHub release downloads" /></a>
  <a href="https://github.com/dihak/herdr/stargazers"><img src="https://img.shields.io/github/stars/dihak/herdr?labelColor=333333&color=666666&logo=github" alt="GitHub stars" /></a>
  <a href="https://github.com/dihak/herdr/releases/latest"><img src="https://img.shields.io/github/v/release/dihak/herdr?label=release&labelColor=333333&color=666666" alt="latest fork release" /></a>
</p>

---

This is a **private fork** of [herdrdev/herdr](https://github.com/herdrdev/herdr) with an **agent grid overlay** (`ctrl+b` then `a`). Docs still live on [herdr.dev](https://herdr.dev/docs/).

**the runtime your coding agents live on.**

- **detach without stopping work** — herdr keeps terminals running in a background server when you close the client or lose your SSH connection. after a server or machine restart, herdr restores the saved layout and can resume supported agent sessions; the original processes do not survive. [session state →](https://herdr.dev/docs/session-state/)
- **several machines, one window** — keep local work and saved ssh machines together, with a combined agent list and independent reconnects. [remote machines →](https://herdr.dev/docs/connecting-machines/)
- **never hunt for the stuck one** — every pane is marked working, blocked, or idle. when an agent stops and needs an answer, herdr says so.
- **agent-native** — agents drive herdr through the cli and socket api: they can spawn panes, prompt each other, and wait until another agent is genuinely blocked. [agent skill →](https://herdr.dev/docs/agent-skill/)
- **runs what you already run** — claude code, codex, cursor, opencode, grok and the rest. herdr doesn't wrap or replace them; it owns their terminals.
- **keyboard and mouse, both first-class** — tmux-style prefix keys *and* click, drag, split. pick per moment, not per tool.
- **plugins** — extend panes and workflows. [browse the marketplace →](https://herdr.dev/plugins/)
- **one rust binary, no electron** — runs in whatever terminal you already use.

---

## install

Linux / macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/dihak/herdr/master/distribution/install.sh | sh
```

Windows:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/dihak/herdr/master/distribution/install.ps1 | iex"
```

Or build from source / grab a [release binary](https://github.com/dihak/herdr/releases). Then `herdr update` pulls later builds from this repo, not herdr.dev.

then start it where the work lives:

```bash
herdr
```

run your agents, split panes, walk away. `ctrl+b q` detaches, `herdr` reattaches. [quick start →](https://herdr.dev/docs/quick-start/)

## docs

everything lives at [herdr.dev/docs](https://herdr.dev/docs/): [quick start](https://herdr.dev/docs/quick-start/) · [concepts](https://herdr.dev/docs/concepts/) · [supported agents](https://herdr.dev/docs/agents/) · [keyboard](https://herdr.dev/docs/keyboard/) · [configuration](https://herdr.dev/docs/configuration/) · [session state](https://herdr.dev/docs/session-state/) · [connecting machines](https://herdr.dev/docs/connecting-machines/) · [remote](https://herdr.dev/docs/persistence-remote/) · [integrations](https://herdr.dev/docs/integrations/) · [plugins](https://herdr.dev/docs/plugins/) · [socket api](https://herdr.dev/docs/socket-api/)

## thanks

every past sponsor and backer is listed in [SPONSORS.md](./SPONSORS.md) — thank you 🐑

enterprise / partnership: hey@herdr.dev

## agent instructions

if you are an ai agent helping with this repository, read [`AGENTS.md`](./AGENTS.md) before making changes and read [`CONTRIBUTING.md`](./CONTRIBUTING.md) before opening issues or PRs.

## development

```bash
git clone https://github.com/dihak/herdr
cd herdr
cargo build --release

just test        # unit tests
just check       # formatting, tests, and maintenance checks
```

## syncing from upstream

This fork tracks [herdrdev/herdr](https://github.com/herdrdev/herdr). GitHub Actions opens a `chore: sync upstream master` PR weekly (Monday). Manual:

```bash
git fetch upstream
git merge upstream/master
# resolve conflicts, keep fork URLs / agent-grid / latest.json
git push origin master
```

Git sync does not ship binaries. After merge, bump `Cargo.toml` if needed, tag, and push the tag so `herdr update` picks it up.

## license

Herdr is licensed under the [Apache License 2.0](LICENSE).
