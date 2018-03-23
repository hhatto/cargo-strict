# cargo-strict

check `unwrap` strict code.

## Instllation

```
$ cargo install --git https://github.com/hhatto/cargo-strict.git
```

## Usage

```
$ cargo strict
```

input:
```rust
$ cat examples/simple.rs
fn main() {
    let s = "Hello world";  /* .unwrap() */
    let _ = s.find("wo").unwrap();
    // let _ = s.find("wo").unwrap();
    let ignore = "s.unwrap();";
}
```

### as linter
```
$ cargo strict
./examples/simple.rs:2:24:     let _ = s.find("wo").unwrap();
./src/main.rs:96:36:     let input = File::open(filename).unwrap();
```

```
$ cargo strict example/simple.rs
./examples/simple.rs:2:24:     let _ = s.find("wo").unwrap();
```

### as formatter
```diff
$ cargo strict --fix
$ git diff
diff --git a/examples/simple.rs b/examples/simple.rs
index e066898..dbe188b 100644
--- a/examples/simple.rs
+++ b/examples/simple.rs
@@ -1,6 +1,6 @@
 fn main() {
     let s = "Hello world";  /* .unwrap() */
-    let _ = s.find("wo").unwrap();
+    let _ = s.find("wo").expect("error-id:5cb0410ba34b040dbbde09dc1991685d");
     // let _ = s.find("wo").unwrap();
     let ignore = "s.unwrap();";
 }
```
