### A simple "VM" based calculator thing
#### Features:
- Number systems: Decimal, Binary, Octal, Hexadecimal
    - Note that output is only in the decimal number system
- Strings with basic escape sequences parsed
    - Strings can be conactenated with the `+` operator
- Basic math operations: Add (`+`), Subtract (`-`), Divide (`/`), Multiply (`*`), Exponent (`**`)
- Binary operations: AND (`&`), OR (`|`), XOR (`^`), Left Shift (`<<`), Right Shift (`>>`)
    - Note that these operations will truncate the floating point of both sides before proceeding
- Variables: Null values or floating point values (64 bit precision)
- Assignment + Operations on variables, ie. Add + Assign (`+=`), Subtract + Assign (`-=`), so on and so forth. This applies to all operators previously discussed
- Null values cannot have any operation performed on them
- Basic function support: each function allows only a single expression to compute
    - Also note that you cannot override built in functions, but you can your own functions. You *can* also have a variable that has the same name as a built in function or the same name as a function you declared.
- Deletion of variables and functions
    - Note that if you have a variable and a function of the same name (what's the point?), this will delete both the function and variable
    - Also, you are not allowed to delete built in functions. Again, why would you want to? 
- REPL: Kinda buggy but works, no need for semicolons or colons, but you can use them
- Command line arguments:
    - `repl` starts the repl
    - `-rf` | `--run-file` reads a file and executes it
    - `-t` | `--text` runs the text provided to the command line
    - `-rb` | `--run-binary` runs the binary file provided by the next argument
    - `-wb` | `--write-binary` reads a file provided by the next argument and generates the bytecode to stores it as binary file. This file is in the same location with the extension `.bin` if another argument is not provided, otherwise, it stores it to the path provided by that other argument.
    - `-rfs` | `--run-store` | `--run-and-store-binary` runs the file provided by the next argument, and stores the bytecode produced in a new file. This file is in the same location with the extension `.bin` if another argument is not provided, otherwise, it stores it to the path provided by that other argument.

Here is a bit of an example of the syntax and the working:
Try to run it
```cs
// Currently using C# syntax highlighting. Which other language offers syntax highlighting that better suits this?

// : at the end to display output
// ; at the end to compute the result but not display it
// use the `print(...)` function to display outputs as well

print(1, 2, 3, 4); // 1 2 3 4

// Basic math
5 + 7:   // 12
5 * 7:   // 35
5 / 7:   // 0.7142...
5 ** 7:  // 78125
5 - 7:   // -2
5 ** -7: // 0.0000128

// of course, any type of well known number system is supported:
10:    // 10
0b111: // 7
0o777: // 511
0xfff: // 4095
// The outputs are all in the decimal system and cannot be changed.

// Bitwise operations
// Note that any bitwise operation will truncate the fraction of both sides before proceeding since floating point bitwise operations don't make sense

1 & 2:  // 0
1 | 2:  // 3
1 >> 2: // 0
1 << 2: // 4
1 ^ 2:  // 3

// Declare variables
let variable_name = 1.5;
variable_name:

// Other operations that also assign to the variable:
variable_name *= 1.5;
variable_name /= 1.5;
variable_name **= 1.5;
variable_name += 1.5;
variable_name -= 1.5;

variable_name: // 1.837...

// Bitwise operations as well:
// The same condition as above applies to this as well

variable_name &= 2;
variable_name |= 2;
variable_name ^= 2;
variable_name >>= 2;
variable_name <<= 2;

variable_name: // 0

// Delete variables if you want to
// Delete functions the same way as well
// You cannot delete built in functions

delete variable_name;

// variable_name: // Will throw an error. Uncomment to try

// Strings
"Hello": // Hello
// String concatenation
"Hello" + " " + "World": // Hello World
// Printing achieves the same effect
print("Hello", "World"); // Hello World

// Functions

let no_args _ = sin(to_radians(90)); // Just a `_` implies no arguments
no_args(): // 1

let args _ a = a / _; // But `_` can be used as an argument when more than one argument is expected
args(5, 2): // 0.4
// The number of arguments are fixed and are not dynamic

// You cannot access variables declared after the function
// let access_outside _ = c + d;

let c = 5;
let d = 10;

// This does not work
// access_outside():

// Just to be clear, recursion is *not* allowed. It does not make sense with a single expression function anyway.
// let sum a b = sum(a, b + 1); // Produces an error

// An interesting workaround I found for recursion (which I fixed now)
// let a _ = 1; // Something arbitrary
// let b _ = a();
// Now, redefine a such that recursion is produced
// let a _ = b(); // Doing this will produce an error
```
Pretty simple, I'd say

Some things are still buggy, and some syntax does not allow you to do what you'd expect, but this is pretty much it.
