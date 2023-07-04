# rs3270
This library abstracts over interacting with the x3270 client. Simply provide the mainframe address and the scripting port for the client in order to immediately interact with the x3270 client programmatically.

## Features

- A `ClientSpawner` implementation provides the means to spawn a `Client`.
  - The current version of this crate provides the x3270 implementation.
- A `CommandExecutor` implementation provides the means to run commands against the connected client.
- Each `CommandBuilder` implementation utilizes a custom `command!` macro to simplify and reduce duplicate code.
- The `MainframeProvider` struct provides functions that utilize one or more lower-level calls to the `CommandExecutor`, allowing for more complex operations.
- The `StreamCommandExecutor` uses the `CommandExecutor` trait, so implementing your own and providing an instance to the `MainframeProvider` allows you to work with your own terminal emulator.
  - Create custom `CommandBuilder` implementations via the `command!` macro as needed

## Usage

There are currently two levels of abstraction implemented in this library.

### TerminalConfiguration, Client, and CommandExecutor

To use this lower-level abstraction, simply create an instance of `TerminalConfiguration`, spawn a `Client` with an implementation of `ClientSpawner`, and create a `CommandExecutor`. After which you will be able to `execute` `CommandBuilder` instances on the `CommandExecutor` that interact with the spawned client.

### MainframeProvider

To use this higher-level abstraction, simply create an instance of a `CommandExecutor` as describe above and supply it to the `new` function of the `MainframeProvider`. With this struct you will be able to call convenient functions for interacting with the attached `Client`.

## Examples

**Example coming soon**

## Inspiration

This crate was inspired by two existing libraries from two different languages.
- j3270
  - https://github.com/filipesimoes/j3270
- py3270
  - https://github.com/py3270/py3270

## Future work

- Headless operation
- Windows support
- Higher automation processing layer