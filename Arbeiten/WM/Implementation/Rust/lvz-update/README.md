# lvz-update

Renumber bibliography links in XML files using their sorted filename position.
All source code, comments, prompts and error messages are in English.

## Open and run in RustRover

1. Extract this archive into your project folder.
2. Use **File > Open** and select the folder containing `Cargo.toml`.
3. Wait for Cargo to download the dependencies.
4. Run `cargo run` in the project terminal.
5. Enter the folder containing your XML files. Surrounding double quotes are accepted.
   Enter `exit` to quit without processing files.

A working Rust/Cargo toolchain is required. Cargo generates `Cargo.lock` on the
first build; keep that file in version control. If updating the earlier starter
project, replace `Cargo.toml`, `src/main.rs` and `README.md`, and add `tests`.
Cargo refreshes the starter project's old lockfile automatically.

## Behaviour

- Includes regular `.xml` files (extension is case-insensitive) in this folder only.
- Ignores subdirectories, symbolic links and other file types.
- Sorts filenames lexicographically after Unicode lowercase conversion.
  Original spelling breaks ties. This is not locale-aware or natural numeric sorting:
  `10.xml` comes before `2.xml`. Zero-padded names sort as expected.
- Numbers all included XML files starting at 1.
- Requires exactly one `link` with `styleclass="Body Text"` and an `anchor`
  consisting of `EL-LVZ-` followed by decimal digits.
- Replaces only the literal digits inside that link. Surrounding whitespace remains.
- Preserves filenames and all other file-content bytes, including UTF-8 BOM and CRLF.
- Requires UTF-8 input (with or without BOM) and well-formed XML.
- Rejects missing/ambiguous targets, non-numeric content, nested markup and CDATA.
- Validates and prepares the entire batch in memory before writing any originals.
- Writes each changed file through a temporary file in the same directory.
- Stops on the first error and exits with a nonzero status.
- If a write fails, previously completed changes remain. The error reports their count.
  This is not a transaction across all files and does not create backups.
- File metadata such as timestamps may change. Do not edit files concurrently.

## Example

If `EL-SNIP-LVZ-0300.xml` is the fifth XML file in sorted order:

```xml
<link anchor="EL-LVZ-0300" styleclass="Body Text">3</link>
```

becomes:

```xml
<link anchor="EL-LVZ-0300" styleclass="Body Text">5</link>
```

## Development checks

```shell
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

The tests cover exact content preservation using the supplied XML fixture,
invalid/ambiguous links, whitespace, filename sorting, ignored subdirectories,
writing replacements, and validation before any modification.

The delivery environment has no Rust/Cargo installation. These Rust tests and
compilation have not been executed there; run the commands above locally.

## Design

`roxmltree` validates XML and locates the source byte range. The program never
serializes the parsed XML. `tempfile` stages complete output before replacement.
The CLI uses standard-library stdin/stdout and Rust's `Result` error propagation.

References:
- https://doc.rust-lang.org/std/fs/fn.read_dir.html
- https://docs.rs/roxmltree/0.21.1/roxmltree/struct.Node.html
- https://docs.rs/tempfile/3.27.0/tempfile/struct.NamedTempFile.html
