# Simple code editor made with [Freya 🦀](https://github.com/marc2332/freya)

This is a simple code editor that showcases how to use tree-sitter and freya together.

WARNING ⚠️: the code sucks (sorry), it's not super efficient either, although it can handle a few thousands lines without problem. Also, don't expect ctrl-c, ctrl-v, text selection, etc.. to work.

this is how it looks:

![Demo](./demo.png)

note: if you are in Linux and wanna give it a try, you must specify the "x11" (wayland support is coming) feature in Cargo.toml:

```
freya = { git = "https://github.com/marc2332/freya", branch = "syntax-highlighting", features = ["x11"]}
```