[![Discord Server](https://img.shields.io/discord/1015005816094478347.svg?logo=discord&style=flat-square)](https://discord.gg/SNcFbYmzbq)

# Valin ⚒️

**Valin** ⚒️ is a **Work-In-Progress** cross-platform code editor, made with [Freya 🦀](https://github.com/marc2332/freya) and Rust.

> **Valin** name is derived from [Dvalinn](https://en.wikipedia.org/wiki/Dvalinn) and it was previously known as `freya-editor`.

![Demo](./demo.png)

You can download it from the [Releases](https://github.com/marc2332/valin/releases) page or run it from source code, with `--release` mode if you want max performance.

## Notes
- SVG support in Linux seems to be giving issues, see the [Tracking Issue](https://github.com/marc2332/valin/issues/52).
- It currently uses Jetbrains Mono for the text editor and Fira Sans for the UI.
- The syntax highlighter is still very generic and is targeted to Rust code at the moment.

## Features

- [x] Open folders
- [x] Open files
- [x] Save files
- [x] Generic Syntax highlighting
- [x] Text editing
- [x] Text selection
- [x] Copy
- [x] Paste
- [x] Undo
- [x] Redo
- [x] Files explorer
- [x] Settings
- [ ] Intellisense (Enable with `--lsp`)
  - [x] Hover (exprimental, only rust-analyzer atm)
  - [ ] Autocomplete
  - [ ] Code actions

# Shortcuts
- `Alt E`: Toggle focus between the files explorer and the code editors
- `Alt +`: Increase font size
- `Alt -`: Decrease font size
- `Esc`: Open Commander
- `Arrows`: Navigate the files explorer when focused
- `Alt Arrows`: Scroll the editor and the cursor with increased speed
- `Alt Arrows`: Scroll the cursor with increased speed  
- `Ctrl Arrows`: Scroll the cursor with increased speed  
- `Ctrl/Meta Z`: Undo
- `Ctrl/Meta Y`: Redo
- `Ctrl/Meta X`: Cut
- `Ctrl/Meta C`: Copy
- `Ctrl/Meta V`: paste
- `Ctrl/Meta S`: Save
