// C bridge for the Rust core. Imported by ios/Sources/Bridging-Header.h and
// bundled into RustCore.xcframework by build-rust.sh. Keep this in sync with
// the `#[unsafe(no_mangle)]` functions in src/lib.rs.
#ifndef RUST_BRIDGE_H
#define RUST_BRIDGE_H

#include <stdint.h>

int32_t rust_add(int32_t left, int32_t right);
char *rust_greeting(const char *name);
void rust_string_free(char *ptr);

#endif /* RUST_BRIDGE_H */
