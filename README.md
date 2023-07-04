# rs3270
This library abstracts over interacting with the x3270 client. Simply provide the mainframe address and the scripting port for the client in order to immediately interact with the x3270 client programmatically.

## Features

- The `ClientAddress` struct provides the means to spawn a `Client` x3270 and a `ClientInterface` for sending commands to that x3270 instance.
  - Each `Command` implementation utilizes a custom `command!` macro to simplify and reduce duplicate code
- The `MainframeProvider` struct provides functions that utilize one or more lower-level calls to the `ClientInterface`, allowing for more complex operations.

## Usage

There are currently two levels of abstraction implemented in this library.

### ClientAddress, Client, and ClientInterface

To use this lower-level abstraction, simply create an instance of `ClientAddress`, spawn a `Client`, and then a `ClientInterface`. After which you will be able to `execute` `Command` instances on the `ClientInterface` that interact with the spawned x3270 client.

### MainframeProvider

To use this higher-level abstraction, simply create an instance of the `ClientInterface` as describe above and supply it to the `new` function of the `MainframeProvider`. With this struct you will be able to call convenient functions for interacting with the attached `Client`.

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