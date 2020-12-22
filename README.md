# Crocolang

![ci](https://github.com/truelossless/crocolang/workflows/ci/badge.svg)

Croco is a small and fun-to-use language written in Rust.

You can see some examples of the syntax below :)  
Other simple examples can be seen under the `tests` folder.
For the partial spec and even more examples, see [here](SPEC.md).

PULL REQUESTS ARE WELCOME SO YOU CAN IMPROVE MY MESS !

The lexer and parser are backend-agnostic, which means it should be easy to add all types of backends.  
Currently there is a interpreter backend (crocoi), and an llvm backend (crocol, WIP)

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

You probably want to put the crocoi/crocol executable in your path.  
Once it's done, you can use do `crocoi myfile.croco` in your favorite shell to run your myfile with the croco interpreter !

```
$ crocoi --help
Usage: crocoi [OPTIONS]

Positional arguments:
  input          the .croco file to execute

Optional arguments:
  -h, --help     show help message
  -v, --version  show croco version
```

```
$ crocol --help
Usage: crocol [OPTIONS]

Positional arguments:
  input             the .croco file to execute

Optional arguments:
  -O OPTIMIZATION   optimization level (O0, O1, O2, O3)
  --verbose         verbose output
  --no-llvm-checks  ignore llvm ir checks
  -h, --help        show help message
  -v, --version     show crocol version
  -S                emit assembly only
  -c                emit object files only
  --emit-llvm       emit llvm ir only
  -o OUTPUT         output file path
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

Arrays

```
let arr = [[2, 1], [3, 4, 5]]
println(arr[1][2])
```

```
5
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

### Built-in functions

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
_Processor: i7 6700HQ, released in September 2015_

```
$ time node bench_name.js
$ time python bench_name.py
$ time croco bench_name.croco
```

| benchmark name      | node  | python | crocoi |
| ------------------- | ----- | ------ | ------ |
| rec fibonacci, n=30 | 200ms | 400ms  | 13s    |
| loop, n=1000000     | 230ms | 236ms  | 850ms  |

We're getting there :D  
Croco is fully interpreted, so it's normal that it's way slower than Node, which is basically a VM.  
However, it should be closer to python performance, but it's clear that there's still a long way to go !
Apparently Python doesn't do any tail call optimization for recursive functions, so it's weird that croco is THAT slow with fibonacci. Actually Python is jitted so that's probably why.

### Where are the performance culprits ?

- A lot of clone calls :S I plan to further profile performance later.

## IDE support

I made a Visual Studio code extension that adds basic syntax highlighting. It is available under the `croco-0.0.3.vsix` file. Follow [these instructions for the installation](https://marketplace.visualstudio.com/items?itemName=fabiospampinato.vscode-install-vsix).
