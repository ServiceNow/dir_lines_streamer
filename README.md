*ServiceNow completed its acquisition of Element AI on January 8, 2021. All references to Element AI in the materials that are part of this project should refer to ServiceNow.*

# dir_lines_streamer

`dir_lines_streamer` is a Rust crate allowing reading files inside a directory line-by-line,
one file after the other (in human-alphabetical order).

```toml
# Cargo.toml
[dependencies]
dir_lines_streamer = "0.1"
```

## Example

Let's say you have a directory containing files split by logrotate that you want to read line-by-line:

```sh
ls fixtures/non-empty-dir/
messages        messages.1      messages.10     messages.2      messages.20
```

Note how the alphabetical ordering sorts `messages.10` before `messages.2`.

This crate allows creating a structure that implements the trait `Iterator<Item = String>`
which returns the lines of the files.

```rust
use failure; // Crate failure 0.1
use dir_lines_streamer::DirectoryLinesStreamer;

let streamer_result: Result<DirectoryLinesStreamer, failure::Error> = DirectoryLinesStreamer::from_dir("fixtures/non-empty-dir");

// Read all lines of all files inside the directory and store them in a Vec<String>
let lines: Vec<String> = streamer.collect();

println!("lines: {:#?}", lines);
```

This will print:

```text
lines: [
    "line one from messages\n",
    "line two from messages\n",
    "line three from messages\n",
    "line one from messages.1\n",
    "line two from messages.1\n",
    "line three from messages.1\n",
    "line one from messages.2\n",
    "line two from messages.2\n",
    "line three from messages.2\n",
    "line one from messages.10\n",
    "line two from messages.10\n",
    "line three from messages.10\n",
    "line one from messages.20\n",
    "line two from messages.20\n",
    "line three from messages.20\n"
]
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Copyright (c) Element AI Inc., 2018, by Nicolas Bigaouette. All rights reserved.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
