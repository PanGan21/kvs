# kvs

A multi-threaded, persistent key/value store server and client with asynchronous networking over a custom protocol.

## Features

- <b>Multi-threaded</b>: Efficiently handles concurrent requests through multi-threading.
- <b>Persistent Storage</b>: Persists data to storage for durability.
- <b>Custom Protocol</b>: Uses a custom networking protocol for communication between the server and client.
- <b>Supports Different Engines</b>: Supports different storage engines, such as kvs and sled.
- <b>Asynchronous Networking</b>: Leverages asynchronous networking for improved performance.

### Installation

To install kvs, follow these steps:

```
cargo install kvs
```

### Usage

#### Running the Server

To start the server:

```
kvs-server --engine <engine_name> --addr <address>
```

- `<engine_name>`: Specifies the storage engine to use (e.g., kvs or sled).

- `<address>`: Specifies the server address (e.g., 127.0.0.1:4000).

#### Running the Client

##### Get Command

To get a value from the key/value store:

```
kvs-client get <key> [--addr <address>]
```

- `<key>`: Specifies the key to retrieve.
- `--addr <address>`: Optional. Specifies the server address.

##### Set Command

To set a value in the key/value store:

```
kvs-client set <key> <value> [--addr <address>]
```

- `<key>`: Specifies the key to set.
- `<value>`: Specifies the value to associate with the key.
- `--addr <address>`: Optional. Specifies the server address.

##### Remove Command

To remove a key from the key/value store:

```
kvs-client rm <key> [--addr <address>]
```

- `<key>`: Specifies the key to remove.
- `--addr <address>`: Optional. Specifies the server address.

##### Run the tests

```
cargo test
```
