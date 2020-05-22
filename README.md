# Croco lang

Just a smol interpreted language to experiment, and to learn rust.
My goal is to improve my language understanding, and to build my dream language.

You can see some examples of the syntax below :)  
For the partial spec and more examples, see [here](SPEC.md).

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

Fibonacci (quite slow but it works !!!)
```croco
fn fib(n num) num {

    if n <= 1 {
        return n
    }

    return fib(n - 1) + fib(n - 2)
}

println("" + fib(20))
```
```
6765
```

String interpolation and functions
```croco
fn fancy_disp(n num) {

    let fancy = "Your variable equals " + n
    println(fancy + ". that's nice uh ?")
}

let bruh = (54^2 + 7) / (12.3+24*7) *0.6
fancy_disp(bruh)
```

```
Your variable equals 1.0129104. that's nice uh ?
```

Function return
```croco
fn divide_by_6(n num) num {
    return n/6
}

println("" + divide_by_6(24))
```
```
4
```

### Variable assignment

```croco
let this_is_12_squared = (5*3-3)^2
let operator_precedence = 12-4*2^8/8
println(this_is_12_squared)
println(operator_precedence)
```
```
144
-116
```

### Builtin functions

The only built-in function is println right now.

```croco
println("nice")
```
```
nice
```

## Benchmarks

The code for the benchmarks can be found under `benchmarks/`

The interesting bits is the relative performance to other languages.  
*Processor: i7 6700HQ (released in September 2015)*

```
$ time node bench_name.js
$ time python bench_name.py
$ time ./croco.exe
```

|benchmark name     |  node    |python|croco|
|-------------------|----------|------|-----|
|rec fibonacci, n=30|     200ms| 400ms|  12s|
|loop, n=1000000    |     230ms| 236ms|376ms|

We're getting there :D  
Croco is fully interpreted, so it's normal that it's slower than node, which is basically a VM.  
However, it should be closer to python performance, but it's clear that there's still a long way to go !
Apparently python doesn't do any tail call optimization for recursive functions, so it's weird that croco is THAT slow with fibonacci.

### Where are the performance culprits ?

Everytime a function is called, the corresponding AST is also cloned, which is very expensive (30-50% of the time is spent cloning). This is why recursive functions are so slow. I've not found a workaround.  

If you can, prefer using regular loops that doesn't involve that much cloning.
