
# String Reuse Visualizer

Macros, linters, formatters, error messages, transpilers, etc. all transform code to code. Users and maintainers of these tools may want insight into how parts of the output code correspond to parts of the input code.

This demo illustrates how using string content by reference rather than cloning makes it easy for tools to identify these correspondences by inspecting the addresses of the characters in memory.

![Screenshot](./files/screenshot.png)

## Try It

### Local

Install [Rust](https://www.rust-lang.org/tools/install), clone the repo, `cargo run`

### Online

To try it online, sign in to https://labs.play-with-docker.com/ and run the following commands:

```bash
# spin up a rust image
docker run -it rust bash
# clone the repo
git clone https://github.com/asvarga/String-Reuse-Visualizer && cd String-Reuse-Visualizer
# run it
cargo run
```

## Details

The trick is to operate on data structures built up of `&str` slices into the input `String`, rather than building up entirely new `String`s in memory. In this demo, I use a `Vec<&str>` for simplicity, but better data structures exist like [ropes](https://en.wikipedia.org/wiki/Rope_(data_structure)). Operations for concatenation, indexing, slicing, searching, replacing, etc. are implemented without any actual string copying.

It's staightforward to idenfity the locations in memory of individual `char`s within these data structures via some safe pointer artithmetic on the container `&str`s. For cases when new `String`s are computed from other string content and need to be allocated, we maintain a simple address-to-address relation to track which `char`s are upstream or downstream of eachother.

A [ratatui](https://ratatui.rs/) interface is provided with a text editor on the left and an interactive debugger on the right.
