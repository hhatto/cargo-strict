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
```
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

### as formatter (WIP)
```
$ cargo strict --fix
./examples/simple.rs:2:24:     let _ = s.find("wo").expect("error-id1");
./src/main.rs:96:36:     let input = File::open(filename).expect("error-id2");
```
