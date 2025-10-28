# raiment-shell

A library of practical convenience functions for writing short "shell" scripts with Deno.

The primary export from this library is the `sh` object. It is a singleton with a set of convenience methods for interacting with the shell. The general idea is a script can import this single object and handle many common needs with single method calls.

## Design principles

- Exception should be thrown on most errors (fail fast)
