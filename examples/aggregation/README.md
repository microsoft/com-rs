# COM Example

A COM example in Rust

# Run

To install the server and run the client, simply run the following from the basic folder:

```bash
cargo run
```

Alternatively, you can choose to build/install/run the server and client seperately.

# Build & Install Server

You can build the server by running the following in the server folder:

```bash
cargo build
```

To "install" the server, you need to add the CLSIDs to your Windows registry. You can do that by running:

```bash
regsvr32 path/to/your/server/dll/file
```

# Run Client

To run the client which talks to the server, simply run the following from the client folder:

```bash
cargo run 
```