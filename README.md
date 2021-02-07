# Crocolang

![ci](https://github.com/truelossless/crocolang/workflows/ci/badge.svg)

Croco is a small and fun-to-use programming language written in Rust.

You can see some examples of the syntax below :)  
Other simple examples can be seen under the `tests` folder.
For the partial spec and even more examples, see [here](SPEC.md).

Feel free to fill issues and open pull requests!!

The lexer and parser are backend-agnostic, which means it should be easy to add all types of backends.  
Currently there is a interpreter backend (crocoi), and an LLVM backend (crocol).

## Downloading croco

Croco is automatically built for Windows, MacOS and Linux, for each Git commit.  
[You can download it here.](https://github.com/truelossless/crocolang/releases/latest)

## Building croco

### Building all backends

Building the crocol backend can be a little bit tough because it relies on LLVM and Clang.  

Download Clang and make sure it's available in your path.  
Build LLVM 11 from source and set the environment variable `LLVM_SYS_110_PREFIX` to your LLVM folder.  
Clone this repository and then run

```bash
cargo build --release
```

### Building the crocoi interpreter only

If you only want to use the crocoi backend, there is no setup.

```bash
cargo build --release --no-default-features --features crocoi 
```

If you still have some trouble, you can look at the CI file `.github/workflows/ci.yml` for a step by step walkthrough on MacOS, Ubuntu and Windows.

## Using croco

You probably want to put the crocoi/crocol executable in your path.  
Once it's done, you can use do `crocoi myfile.croco` in your favorite shell to run your myfile with the croco interpreter!

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

**Fibonacci**

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

### Built-in functions

NOTE: right now namespaces are broken: use directly the function name without the module name.

```croco
// import required modules
import "os"
import "math"
import "fs"
import "http"

// println and some other functions are imported by default
println("nice")
println(os.exec("git --version"))

if math.pi > 3. {
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

Assertion failed!
```

## Benchmarks

The code for the benchmarks can be found under `benchmarks/`

The interesting bits is the relative performance to other languages.  
_Processor: i7 6700HQ, released in September 2015_

### Crocoi

```
$ time node bench_name.js
$ time python bench_name.py
$ time crocoi bench_name.croco
```

| benchmark name      | node  | python | crocoi |
|---------------------|-------|--------|--------|
| rec fibonacci, n=30 | 200ms | 400ms  | 13s    |
| loop, n=1000000     | 230ms | 230ms  | 850ms  |

We're getting there :D  
Crocoi is fully interpreted, so it's normal that it's way slower than Node, which is basically a VM.  
However, it should be closer to python performance, but it's clear that there's still a long way to go!
Apparently Python doesn't do any tail call optimization for recursive functions, so it's weird that croco is THAT slow with fibonacci.

### Crocol

```
$ time clang bench_name.c -O3
$ time ./a.out
$ time crocol bench_name.croco -O3
$ time ./bench_name
```

**Execution speed**

| benchmark name      | gcc    | clang  | crocol |
|---------------------|--------|--------|--------|
| rec fibonacci, n=45 | 3800ms | 4100ms | 4300ms |

These results are pretty good !  
Now that croco has integer primitives, it is almost as fast as clang !! 

**Compilation time (after warmup)**

| benchmark name      | clang | crocol |
|---------------------|-------|--------|
| rec fibonacci, n=40 | 250ms | 400ms  |

**LLVM IR differences**

`clang fib.c -O3 -emit-llvm -S`
```
; Function Attrs: nounwind readnone uwtable
define dso_local i32 @fib(i32 %0) local_unnamed_addr #0 {
  %2 = icmp slt i32 %0, 2
  br i1 %2, label %11, label %3

3:                                                ; preds = %1, %3
  %4 = phi i32 [ %8, %3 ], [ %0, %1 ]
  %5 = phi i32 [ %9, %3 ], [ 0, %1 ]
  %6 = add nsw i32 %4, -1
  %7 = tail call i32 @fib(i32 %6)
  %8 = add nsw i32 %4, -2
  %9 = add nsw i32 %7, %5
  %10 = icmp slt i32 %4, 4
  br i1 %10, label %11, label %3

11:                                               ; preds = %3, %1
  %12 = phi i32 [ 0, %1 ], [ %9, %3 ]
  %13 = phi i32 [ %0, %1 ], [ %8, %3 ]
  %14 = add nsw i32 %13, %12
  ret i32 %14
}
```

`crocol fib.croco -O3 --emit-llvm`
```
define i32 @fib(i32 %0) local_unnamed_addr #7 {
entry:
  %cmpnum5 = icmp slt i32 %0, 2
  br i1 %cmpnum5, label %then, label %endif

then:                                             ; preds = %endif, %entry
  %accumulator.tr.lcssa = phi i32 [ 0, %entry ], [ %add, %endif ]
  %.tr.lcssa = phi i32 [ %0, %entry ], [ %sub3, %endif ]
  %accumulator.ret.tr = add i32 %.tr.lcssa, %accumulator.tr.lcssa
  ret i32 %accumulator.ret.tr

endif:                                            ; preds = %entry, %endif
  %.tr7 = phi i32 [ %sub3, %endif ], [ %0, %entry ]
  %accumulator.tr6 = phi i32 [ %add, %endif ], [ 0, %entry ]
  %sub = add nsw i32 %.tr7, -1
  %callfn = tail call i32 @fib(i32 %sub)
  %sub3 = add nsw i32 %.tr7, -2
  %add = add i32 %callfn, %accumulator.tr6
  %cmpnum = icmp slt i32 %.tr7, 4
  br i1 %cmpnum, label %then, label %endif
}
```

## IDE support

I made a Visual Studio Code extension that adds basic syntax highlighting. It is available under the `croco-0.0.3.vsix` file.