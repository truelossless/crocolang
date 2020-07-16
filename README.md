# Croco lang

![build](https://github.com/truelossless/crocolang/workflows/build/badge.svg)
![tests](https://github.com/truelossless/crocolang/workflows/tests/badge.svg)

Croco is a small and fun-to-use interpreted language written in Rust.

You can see some examples of the syntax below :)  
Simple examples of the syntax can be seen under the `tests` folder.
For the partial spec and even more examples, see [here](SPEC.md).

PULL REQUESTS ARE WELCOME SO YOU CAN IMPROVE MY MESS !

## Downloading croco

Croco is automatically built for Windows, MacOS and Linux, for each Git commit.  
[You can download it here.](https://github.com/truelossless/crocolang/releases/latest)

## Building croco

You can also build Croco by yourself easely.  
Make sure Rust is installed and run in the main directory
```bash
cargo build --release
```

## Using croco

You probably want to put the croco executable in your path.  
Once it's done, you can use do `croco myfile.croco` in your favorite shell !

```
$ croco --help
Usage: croco [OPTIONS]

Positional arguments:
  input          the .croco file to execute

Optional arguments:
  -h, --help     show help message
  -v, --version  show croco version
```

## Examples

Fibonacci (quite slow but it works !!!)
```croco
fn fib(n num) num {

    if n <= 1 {
        return n
    }

    return fib(n - 1) + fib(n - 2)
}

println(fib(20) as str)
```
```
6765
```

String interpolation and functions
```croco
fn fancy_disp(n num) {
    let fancy = "Your variable equals " + n as str
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

println(divide_by_6(24) as str)
```
```
4
```

### Variable assignment

```croco
let this_is_12_squared = -(5*3-3)^2
let operator_precedence = 12+4*2^8/4
println(this_is_12_squared as str)
println(operator_precedence as str)
```
```
-144
268
```

### Builtin functions

NOTE: right now namespaces are broken: use directly the function name without the module name.

```croco

// imports required modules
import "os"
import "math"
import "fs"
import "http"

// println and some other functions are imported by default
println("nice")
println(os.exec("git --version"))

if math.pi > 3 {
  println("pi is a big number")
}

if fs.exists("UNICORN.exe") {
  println("I don't believe it")
}

println(http.get("https://www.twitter.com/robots.txt"))

assert(true == false)
```
```
nice
git version 2.25.0.windows.1

pi is a big number
User-agent: *
Disallow: /

Assertion failed !
```

## Benchmarks

The code for the benchmarks can be found under `benchmarks/`

The interesting bits is the relative performance to other languages.  
*Processor: i7 6700HQ, released in September 2015*

```
$ time node bench_name.js
$ time python bench_name.py
$ time croco bench_name.croco
```

|benchmark name     |  node    |python|croco|
|-------------------|----------|------|-----|
|rec fibonacci, n=30|     200ms| 400ms|13.5s|
|loop, n=1000000    |     230ms| 236ms|376ms|

We're getting there :D  
Croco is fully interpreted, so it's normal that it's way slower than Node, which is basically a VM.  
However, it should be closer to python performance, but it's clear that there's still a long way to go !
Apparently Python doesn't do any tail call optimization for recursive functions, so it's weird that croco is THAT slow with fibonacci. Actually Python is jitted so that's probably why. 

### Where are the performance culprits ?

- Everytime a function is called, the corresponding AST is also cloned, which is very expensive (30-50% of the time is spent cloning). This is why recursive functions are so slow. I've not found a workaround. If you can, prefer using regular loops that doesn't involve that much cloning.  

- Now that croco as pretty error messages, it has to keep track of a few more informations (the file name and line number mapped to each, instruction) and in the fibonacci test we lost 1.5s. I'll try to find a way to avoid some clone() calls on the file name strings.

## IDE support

I made a Visual Studio code extension that adds basic syntax highlighting. It is available under the `croco-0.0.3.vsix` file. Follow [these instructions for the installation](https://marketplace.visualstudio.com/items?itemName=fabiospampinato.vscode-install-vsix).
