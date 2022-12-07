# freya-editor

This is a code editor that showcases how to use tree-sitter and freya together.

WARNING ⚠️: the code sucks (sorry), it's not super efficient either, although it can handle a few thousands of lines without problem.

this is how it looks:

![Demo](./demo.png)

note: if you are in Linux and wanna give it a try, you must specify the "x11" (wayland support is coming) feature in Cargo.toml:

```
freya = { git = "https://github.com/marc2332/freya", branch = "syntax-highlighting", features = ["x11"]}
```