# rust_c_interop

## Dev setup

To install windows target run:

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

## References

- [rust_c_interop](https://github.com/kgrech/rust_c_interop)
- [rust-ffi-omnibus](https://github.com/shepmaster/rust-ffi-omnibus)


## How to (static linking)[Windows]

```c
// main.c

#include <stdio.h>
#include <stdbool.h>
#include "rust_c_interop.h"

int main(int argc, char **argv)
{
    client_t *client = init_connection("<Host>", 1433, "<Username>", "<Password>", "<Database>");
    bool ret = simple_query(client, "<Table>");
    printf("Execution was successfull? %s", ret ? "true" : "false");
    free_client(client);
    return 0;
}

```

- Compile dll with `cargo build --lib --release`
- Get [file header](./include/rust_c_interop.h)
- Compile `C` executable with `gcc -o main.exe main.c rust_c_interop`

