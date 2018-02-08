This is a throwaway project I'm doing to get familiar with OpenGL.

![basic-lighting](media/v0.1.0-basic-lighting.gif)

## Dependencies

### rust nightly

This project requires rust nightly.

### git lfs

Binary files are stored with [git lfs](https://github.com/git-lfs/git-lfs) to keep the repository size small. 

## Tools


### gpick

If you have a shader output values per pixel in the r, g, b and/or a components
for debugging, a color picker can be helpful when determining the values for
each component from the color.

### lldb/gdb

Install and then run `rust-gdb` or `rust-lldb`.

```
file target/debug/opengl-experiment
break rust_panic
run
```

### rustfmt

Format rust code. Recommended way to install for nightly as of jan 2018 is through rustup.

```
rustup component add rustfmt-preview
```

## Changelog

You can find the [changelog here](CHANGELOG.md).
