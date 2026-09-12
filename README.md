# brainrust

A Brainfuck interpreter written in Rust, built as a learning project.

## Build

```sh
cargo build --release
```

## Usage

```
brainrust [OPTIONS]
```

Exactly one of `--file` / `--inline-program` must be provided.

### Examples

Run a program from a file:

```sh
brainrust --file src/programs/large_fib.b
```

Run code inline (prints `A`):

```sh
brainrust --inline-program '++++++[>++++++++++<-]>+++++.'
```

Read input from stdin, then dump the tape afterwards:

```sh
brainrust --inline-program ',[.,]' --memdump <<< "hello"
```

Step through a program interactively:

```sh
brainrust --inline-program '++++++[>++++++++++<-]>+++++.' -d
```

```
Program: 
++++++[>++++++++++<-]>+++++.
                   ^
Memory: [6, 10]
s: step / r: run the program until halt / p: print current memory value / q: quit
```


### Options

| Flag | Description |
| --- | --- |
| `-f, --file <FILE>` | Path to a Brainfuck source file |
| `-i, --inline-program <CODE>` | Run BF code passed directly on the command line |
| `-d, --interactive-mode` | Step-by-step debug mode (shows state between steps) |
| `--memsize <MEMSIZE>` | Memory tape size (default: 500) |
| `--memdump` | Dump memory at the end of execution |

## Project layout

```
src/
├── main.rs         # entry point, wires the CLI runner
├── cli.rs          # argument parsing, runner, interactive mode
├── interpreter.rs  # execution engine
├── program.rs      # compiled program (tokens + jump table)
├── tokenizer.rs    # source -> tokens, bracket matching
├── token.rs        # token definitions
└── error.rs        # error types
```
