# COM Example

A COM example in Rust

# Install

To "install" this app, you need to add the following keys to your Windows registry:

```
[Computer\HKEY_CLASSES_ROOT\CLSID\{C5F45CBC-4439-418C-A9F9-05AC67525E43}]
@="Cat Component"
[Computer\HKEY_CLASSES_ROOT\CLSID\{C5F45CBC-4439-418C-A9F9-05AC67525E43}\InprocServer32]
@="C:\path\to\the\dll\file\in\your\target\folder"
```

# Run

To run, simply run:

```bash
cargo run --release
```