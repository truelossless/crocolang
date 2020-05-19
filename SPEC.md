# Croco lang

I will talk here about my dream interpreted language; and not the current state of this repository.
Croco is designed to be a fun, fast, and easy to use programming language.

## Variables [20%]

### Primary types [60%]

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

```croco
let text = "hello world"

let text2 str 
// or let text2 = ""
```

```croco
let data = 0
```

### Strong typing

Variables cannot change type (as opposed to JS)
```croco
let foo = "beep boop i'm a robot"
foo = 0101011010

// ERROR !
```

## Functions [40%]

Functions are declared with the `fn` keyword. When they return a value the type should be annotated. A value is returned with the `return` keyword. You can exit early a function without any return value with the `return` keyword as well.

```croco
fn greet() {
    print("Hello, neighbourhood !")
    return

    // this will never be executed
    print("I hate this place but nobody is going to see this !")
}


fn books_sold() num {
    return 42
}

greet()
let books_sold_today = books_sold()
```


## Built-in test framework [0%]

### Writing a test

Tests are similar to what Zig offers.
```croco

fn choose() bool {
    return true
}

test "division" {
    assert_eq(6/3, 2)
}

test "basic assert" {
    assert(choose())
}
```

### Running tests

    croco test main.croco

## Comments
The only valid comments are started with `//`
```croco
let actual_code; // this is a comment
// this won't be read by the interpreter !!
```

## Line endings
LF line endings should be used, but CRLF are also supported. Each instruction is supposed to go on a new line, to prevent bad programming practices with multiple instructions on the same line.
Putting a semicolon at the end of a line (or anywhere really) will result in an error.

## Naming convention
Variables should use the snake_case convention.
```croco
// DO !!
bool long_ass_and_yet_readable_variable_name = true

// DON'T !!
bool unreadableStringGivingJavaVietnamFlashbacks = false
```