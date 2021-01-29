# fonv-cracker

Currently this tool is a helper utility for cracking fallout: new vegas terminals

The plan is to extend it to also be able to play a clone of the cracking game.

![Tests](https://github.com/scottnm/fonv-cracker/workflows/Tests/badge.svg)

## Game Demo Progress

- [Improve the word generation so that in general our words are more similar and match more frequently](https://github.com/scottnm/fonv-cracker/commit/8693452)

![Demo showing improved word generation](demo/07-improved-word-generation.gif)

- [Support entering in words from the TUI and using that to power a full game loop](https://github.com/scottnm/fonv-cracker/commit/11a9bc4)

![Game screen showing full game loop](demo/06-tui-game-loop.gif)

- [Added cursor navigation and word selection to the TUI](https://github.com/scottnm/fonv-cracker/commit/1b7074c8)

![Game screen with showing cursor navigation and word selection](demo/05-tui-selection.gif)

- [Fill in hex dump screen with selected words and memory noise](https://github.com/scottnm/fonv-cracker/commit/108b30f)

![Game screen with random words and memory noise in hex dump view](demo/04-fill-in-words.png)

- [Add a mocked out game screen](https://github.com/scottnm/fonv-cracker/commit/1bcb410)

![Mocked out game screen image](demo/03-game-screen-mock.png)

- [Non-TUI version of game loop](https://github.com/scottnm/fonv-cracker/commit/93181fa)

![Animation of non-TUI game loop](demo/02-non-tui-game.gif)

- [Generating words](https://github.com/scottnm/fonv-cracker/commit/bf43b7ce1ba3e12ff41b8950f6de8fe6e9169a57)

![Animation of words being generated](demo/01-generate-words.gif)
