# MicroPerl

A subset Perl compiler targeting the Z80 processor. Compiles Perl-like source code to bytecode, then generates a native Z80 ROM image with an embedded bytecode interpreter.

## Features

- **Scalar variables** - `my $x = 42;`
- **Strings** - `my $s = "hello";`
- **Arithmetic** - `+`, `-`, `*`, `/`, `%`, `++`, `--`
- **Comparisons** - `==`, `!=`, `<`, `>`, `<=`, `>=`, `eq`, `ne`, `lt`, `gt`
- **Logical operators** - `&&`, `||`, `!`
- **Control flow** - `if`/`elsif`/`else`, `while`, `for`
- **Subroutines** - `sub name($arg) { ... }`
- **Pattern matching** - `$s =~ /pattern/`, `$s !~ /pattern/` with `.` wildcard
- **I/O** - `print`

## Building

```sh
cargo build --release
```

The compiler binary will be at `target/release/microperl`.

## Usage

Generate a Z80 ROM image:

```sh
./target/release/microperl program.pl --rom output.rom
```

Debug options:

```sh
# Show tokenized output
./target/release/microperl program.pl --tokens

# Show parsed AST
./target/release/microperl program.pl --ast

# Show compiled bytecode
./target/release/microperl program.pl --bytecode
```

## Example

```perl
# FizzBuzz in MicroPerl

my $i = 1;

while ($i <= 30) {
    if ($i % 15 == 0) {
        print "FizzBuzz\n";
    } elsif ($i % 3 == 0) {
        print "Fizz\n";
    } elsif ($i % 5 == 0) {
        print "Buzz\n";
    } else {
        print $i, "\n";
    }
    $i++;
}
```

## Architecture

```
Source (.pl) -> Lexer -> Parser -> Compiler -> Bytecode -> Z80 Code Generator -> ROM
```

The generated ROM contains:
- Native Z80 runtime (~4KB) with bytecode interpreter
- Compiled bytecode appended at 0x1000
- String table

## Testing

```sh
cargo test
```

## License

BSD 3-Clause License. See [LICENSE](LICENSE).
