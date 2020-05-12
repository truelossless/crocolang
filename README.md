# Croco lang

Just a smol interpreted language to experiment, and to learn rust.
My goal is to improve my language understanding, and to build my dream language.

You can read the partial spec [here](SPEC.md).

PULL REQUESTS ARE WELCOME SO YOU CAN IMPROVE MY MESS !

## Building croco

Make sure Rust is installed and run in the main directory
```bash
cargo build --release
```

## Using croco

### Unix
- Copy target/release/croco to your favourite directory
- Put a "main.croco" file with your croco code next to the croco executable
- Open a terminal and run
```bash
./croco
```

### Windows
- Copy target/release/croco.exe to your favourite directory
- Put a "main.croco" file with your croco code next to the croco executable
- Open cmd.exe and run
```bash
croco.exe
```

I'll add a CLI to specify the file path one day.

## Examples

```croco
fn fancy_disp(n num) {

    let fancy = "Your variable equals " + n
    println(fancy + ". that's nice uh ?")
}

let bruh = (54^2 + 7) / (12.3+24*7) *0.6
fancy_disp(bruh)
```

### Variable assignment

```croco
let this_is_12_squared = (5*3-3)^2
let operator_precedence = 12-4*2^8/8
println(this_is_12_squared)
println(operator_precedence)
```

#### Output
```
144
-116
```

### Builtin functions

The only built-in function is println right now.

```croco
println("nice")
```

#### Output
```
nice
```