# WebJPEG

A small utility that can be used to create compressed circle avatars.
Image data outside of the circle will be removed without creating additional JPEG artifacts.

## Installation

```bash
cargo build --release
cargo install --path .
```

## Usage

```bash
# Show arguments
webjpeg --help

# Create a circle with a diameter of 200 pixels and iterate JFIF quality down
# until filesize is <=6000
webjpeg -- -c -s 200 -m 6000 <input> <output>
```
