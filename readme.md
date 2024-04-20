[![Discord Server](https://img.shields.io/discord/1015005816094478347.svg?logo=discord&style=flat-square)](https://discord.gg/SNcFbYmzbq)

# freya-editor 

⚠️ This is a **work in progress experimental** code editor using [Freya 🦀](https://github.com/marc2332/freya).

![Demo](./demo.png)

You can download it from the [Releases](https://github.com/marc2332/freya-editor/releases) page or run it from source code, with `--release` mode if you want max performance.

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
