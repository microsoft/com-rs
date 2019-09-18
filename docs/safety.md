# Safety

COM specifies very little in the way of memory safety of COM based APIs. It is left up to the programmer to verify APIs and to make safe wrappers for them.

## The `unsafe` Keyword

While it is not a requirement for the methods of a `com_interface` to marked as `unsafe`, it is generally a good idea to do so unless there is no way for that method to be called in an unsafe way.

## `&self`, `&mut self`, and `self`

All methods of a `com_interface` are required to take an unexclusive reference to self (`&self`). This reflects the reality that COM interfaces do not have exclusive access to the underlying class,
and it does not take ownership (i.e., it is not responsible for the destruction) of the underlying class. As such if you're implementing a COM server, you will most likely need to use [interior
mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) if you would like to mutate state in method calls.
