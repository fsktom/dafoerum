# dafoerum

trying out leptos and stuff for creating a forum or wiki (not sure yet)

<https://youtu.be/V1cqQRmVAK0> + 2 years ago greg with primeagen

## Development

You have to have installed:

- tailwindcss v4
- leptosfmt
- cargo-leptos
- Rust nightly with wasm32-unknown-unknown target

### VSC

- see [settings](.vscode/settings.json)
- rust-analyzer extension
- tailwindcss extension
- markdownlint extension for linting .md files

### Dev Build

```sh
LEPTOS_TAILWIND_VERSION=v4.1.3 cargo leptos watch --hot-reload
```

### WSL Port Forwarding

When developing on Windows using WSL and you want to access the page on another device on the LAN,
you have to set up port forwarding each time WSL boots up (new IP)

On Admin Powershell:

```powershell
netsh interface portproxy add v4tov4 listenport=3000 listenaddress=0.0.0.0 connectport=3000 connectaddress=$((wsl hostname -I).Trim())
```

## General Plan

I've been thinking about what I want to do with this project. I like creating things,
or rather when I see something, I'm thinking
"hmm maybe I could do this too" - for learning and as a challenge etc.

This started out as a forum, but I think I'm gonna expand it to what I originally wanted to do:
something akin to a private tracker with a forum, wiki,
torrent listing + separately the actual torrent tracker and IRC server + user management

Absolutely huge-ass project, taking inspiration from Gazelle and UNIT3D I guess,

But sounds fun I think... or at least the little bit of fun/distraction from other things :))) 🦊 :/

For the forum aspect, maybe also take some inspiration from more modern takes like <https://users.rust-lang.org/>
but combine it with the old-school and clear way like the AB Forum

## Useful Links

- <https://tailwindcss.com/docs/responsive-design>
