# YTUI (I'm not sure if the name is taken already)

This is a TUI client for youtube (largely based on my other project, [twitch-tui-client](https://github.com/bolshoytoster/twitch-tui-client)).

I made this because the existing youtube clients didn't really meet my expectations.

This is a work in progress, you'll currently run into `todo!()`s in a few places:
- Channels
- Playlists

## Config

This program has a fairly detailed config file (it's `src/config.rs`, minor programming knowledge would help), everything before line 53 is prelude stuff (could be moved to a different file).

You can find information of individual configs in the file, next to the configs.

By default, it uses mpv for playing videos, and ffplay for playing streams, since they're the most convenient for both.

## Running

This is not on crates.io, so you will have to download it directly from the repo and run:
```sh
$ cargo run # Optionally `--release`
```

## Controls

Basically the same as twitch-tui-client, but has `n` to see 'next' videos (recommendations)
```rust
match key {
  'Q' => quit,
  UpArrow | 'J' => up,
  DownArrow | 'K' => down,
  PageUp => page up,
  PageDown => page down,
  RightArrow | 'L' => match current_selection {
    Header => Show category, if there is one,
	Video => Play with specified player,
	Transcript => Show transcript,
	CommentSection => Show comments,
	Comment => Show replies, if any,
	Channel | Paylist => todo!(),
  },
  LeftArrow | 'B' => go back,
  'H' => go back to home,
  'S' | '/' => open search box, until enter key is pressed,
  'N' => View recommendations
  'R' => refresh page,
}
```
