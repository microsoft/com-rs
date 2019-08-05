# COM Example

A COM example in Rust

# Install

First build the server by running the following in the server folder:

```bash
cargo build
```

To "install" the server dll, you need to add the following keys to your Windows registry:

```
[Computer\HKEY_CLASSES_ROOT\CLSID\{C5F45CBC-4439-418C-A9F9-05AC67525E43}]
@="Cat Component"
[Computer\HKEY_CLASSES_ROOT\CLSID\{C5F45CBC-4439-418C-A9F9-05AC67525E43}\InprocServer32]
@="C:\path\to\the\server\dll\file\in\your\target\folder"
```

# Run

To run the client which talks to the server, simply run the following from the client folder:

```bash
cargo run 
```