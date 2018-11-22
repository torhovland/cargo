use std::env;
use support::{basic_manifest, project};

#[test]
fn collision_dylib() {
    // Path dependencies don't include metadata hash in filename for dylibs.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "1.0.0"

            [lib]
            crate-type = ["dylib"]
            "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
            [package]
            name = "b"
            version = "1.0.0"

            [lib]
            crate-type = ["dylib"]
            name = "a"
            "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    // j=1 is required because on windows you'll get an error because
    // two processes will be writing to the file at the same time.
    p.cargo("build -j=1")
        .with_stderr_contains(&format!("\
[WARNING] output filename collision.
The lib target `a` in package `b v1.0.0 ([..]/foo/b)` has the same output filename as the lib target `a` in package `a v1.0.0 ([..]/foo/a)`.
Colliding filename is: [..]/foo/target/debug/deps/{}a{}
The targets should have unique names.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future, see https://github.com/rust-lang/cargo/issues/6313
", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX))
        .run();
}

#[test]
fn collision_example() {
    // Examples in a workspace can easily collide.
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["a", "b"]
            "#,
        )
        .file("a/Cargo.toml", &basic_manifest("a", "1.0.0"))
        .file("a/examples/ex1.rs", "fn main() {}")
        .file("b/Cargo.toml", &basic_manifest("b", "1.0.0"))
        .file("b/examples/ex1.rs", "fn main() {}")
        .build();

    p.cargo("build --examples")
        .with_stderr_contains("\
[WARNING] output filename collision.
The example target `ex1` in package `b v1.0.0 ([..]/foo/b)` has the same output filename as the example target `ex1` in package `a v1.0.0 ([..]/foo/a)`.
Colliding filename is: [..]/foo/target/debug/examples/ex1[EXE]
The targets should have unique names.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future, see https://github.com/rust-lang/cargo/issues/6313
")
        .run();
}

#[test]
fn collision_export() {
    // --out-dir combines some things which can cause conflicts.
    let p = project()
        .file("Cargo.toml", &basic_manifest("foo", "1.0.0"))
        .file("examples/foo.rs", "fn main() {}")
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build --out-dir=out -Z unstable-options --bins --examples")
        .masquerade_as_nightly_cargo()
        .with_stderr_contains("\
[WARNING] `--out-dir` filename collision.
The example target `foo` in package `foo v1.0.0 ([..]/foo)` has the same output filename as the bin target `foo` in package `foo v1.0.0 ([..]/foo)`.
Colliding filename is: [..]/foo/out/foo[EXE]
The exported filenames should be unique.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future, see https://github.com/rust-lang/cargo/issues/6313
")
        .run();
}