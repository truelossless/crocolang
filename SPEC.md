# Croco lang

Here is the spec of Croco, an interpreted language. The implementation percentage is shown for each point discussed.  
Croco is designed to be a fun, fast, and easy to use programming language.

## Variables [70%]

### Primitives [90%]

- `num` represents a number (integer or floating point, with positive or negative values).  
Its default value is `0`.
- `str` represents a string of characters. Note that there isn't a char type.  
Its default value is `""`.
- `bool` represents a boolean, either `false` or `true`.  
Its default value is `false`.

### Declaration

Variables can be declared with the `let` keyword.  
Variables must be declared.
If the variable is assigned immediatly, you don't need to annotate its type.  
Variables should use camel_case.

```croco
let text = "hello world"

let text2 str 
// or let text2 = ""
```

```croco
let data = 0
```

### Strong typing

Variables cannot change type.
```croco
let foo = "beep boop i'm a robot"
foo = 0101011010

// ERROR !
```

### Casting [70%]

Primitive types can all be casted from one to another with the `as` operator.  
The only cast that can fail is `str` -> `num`  .
The specific behavior of the casts is described in the examples below.

```croco
// num to str
assert(3 as str == "3")

// num to bool
assert(0 as bool == false) // 0 is the only falsy value
assert(-34 as bool == true)

// str to num
let failed_cast = "yo !" as num // ERROR !
assert("-12" as num == -12)

// str to bool
assert("ahahhldsf" as bool == false)
assert("true" as bool == true)

// bool to str
assert(false as str == "false")
assert(true as str == "true")

// bool to num
assert(false as num == 0)
assert(true as num == 1)
```

### Arrays [50%]

Arrays don't have a fixed length.  
All array elements must be of the same type.  
You can use the array indexing syntax to get the value of a field.

```croco
let arr = [5, 3, 2, 8.8]

// empty array of an array of strings 
let arr2 [[str]]

println(arr[0])
```
```
5
```

### Maps [0%]

Maps are associative dictionnaries.  
All map keys and values must be of the same type.  
You can use the array indexing syntax to get the value of a field.

```croco
let map = [
    "hello": 10
    "goodbye": 100
]

println(arr["hello"])
```
```
10
```

### Structs [80%]

Structs must be defined with the `struct` before they are created.  
There is no anonymous objects.  
Structs should use PascalCase.

```croco
struct Character {
    name str
    hp num
    is_alive bool
}

let char = Character {
    hp: 100
    name: "xXWarriorXx"
}

println(char.name)
println(char.is_alive) // will default to false
```

### References [50%]

References point to another object.  
When calling a struct field or method, dereferencing is automatically performed.  
Else, you can dereference a reference using the * operator. NOTE: not yet implemented  
You can't declare dangling references.


```croco
let txt = "lorem ipsum"

// let ref &str 
// ref = &txt
// this won't work

let ref = &txt
*ref = "lorum ipsem"

println(txt)
```
```
lorum ipsem
```

## Lööps [20%]

Loops are very similar to what other languages offers.  
Currently only `while` is implemented.

### While loops [100%]

```croco
let a = 0
while a < 5 {
    println("a is indeed" + a as str)
}
```

### For loops [0%]

You should use ranges when iterating with for loops
```croco
for let i in 0..5 {
   println("a is indeed" + a)
}
```

### Early returns from loops [100%]

```croco
let a = 0
while true {

    if a == 1 {
        println("skipping 1")
        continue
    }

    println(a as str)

    if a == 2 {
        break
    }

    a += 1
}
```
```
1
2
3
4
skipping 5
6
7
```

## Functions [80%]

Functions are declared with the `fn` keyword. When they return a value the type should be annotated. A value is returned with the `return` keyword. You can exit early a function without any return value with the `return` keyword as well.

```croco
fn greet() {
    println("Hello, neighbourhood !")
    return

    // this will never be executed
    println("I hate this place but nobody is going to see this !")
}


fn books_sold() num {
    return 42
}

greet()
let books_sold_today = books_sold()
```

### Methods [80%]
Structs can also have functions.
```croco
struct Character {

    name str

    fn hi() {
        println(self.name)
    }
}

let bobby = Character {
    name: "Bobby"
}

bobby.hi()
```

## Control flow [60%]

`if`, `elif` and `else` can be used for conditionnal matching.  

```croco
let croco_state = "good"

if croco_state == "bad" {
    println("croco is trash on wheels")
} elif croco_state == "good" {
    println("croco rocks")
} else {
    println("croco is neither good nor bad")
}
```

## Imports [40%]

You can import other files by specifying their path with the `import` keyword.

math.croco
```croco
let pi = 3.14159
```

main.croco
```croco
import "./math"
println(pi)
```

you can also use built-in librairies. In this case, you don't specify a path but a name.  
When importing built-in librairies, you must use the library name before calling any variable or function.
NOTE: this is broken right now

```croco
import "math"
println(math.e)
```

### Known issues
Importing one file will import all files imported by this file.

## Traits [0%]
Traits are used for polymorphism. They are similar to Go interfaces. A struct implementing all the functions of a trait automatically implements this trait. The function definitions must match. A trait is entirely considered as a type.

```croco

trait speak {
    yell() str
    say() str
}

struct Dog {
    fn yell() str {
        return "WOOF"
    }

    fn say() str {
        return "woof"
    }
}

struct Cat {
    fn yell() str {
        return "MIAAAA"
    }

    fn say() str {
        return "miou"
    }
}

let animal speak = Dog {}
println(animal.speak()); 

animal = Cat {}
println(animal.yell());
```
```
woof
MIAAAA
```

## Built-in librairies

Croco aims to have a really complete standard library.  
Here are the first modules implemented:
- `fs`
- `http`
- `math`
- `os`

## Built-in test framework [0%]

### Writing a test

Tests are similar to what Zig offers.
```croco

fn choose() bool {
    return true
}

test "division" {
    assert(6/3 == 2)
}

test "basic assert" {
    assert(choose())
}
```

### Running tests

In your main directory,

```
croco test
```

## Comments
The only valid comments are started with `//`
```croco
let actual_code; // this is a comment
// this won't be read by the interpreter !!
```

## Line endings
LF line endings should be used, but CRLF are also supported. Each instruction is supposed to go on a new line, to prevent bad programming practices with multiple instructions on the same line.
Hence you don't need semicolons or parenthesis around control keywords.

## Naming convention
Variables should use the snake_case convention.
```croco
// DO !!
bool long_ass_and_yet_readable_variable_name = true

// DON'T !!
bool unreadableStringGivingJavaVietnamFlashbacks = false
```

## Appendix

### Operator precedence
Higher value means higher priority.

|operator             |precedence|
|---------------------|----------|
|`\|\|`               |1         |
|`&&`                 |2         |
|`==` `!=`            |3         |
|`>` `>=` `<` `<=`    |4         |
|`+` `-`              |5         |
|`*` `/`              |6         |
|`as`                 |7         |
|`-`(unary)           |8         |
|`^`                  |9         |
|`&` (ref) `*` (deref)|10        |
|`.` `[]`             |11        |