#!/bin/fish
cd (status dirname)
cargo build
if test $status -eq 0
  env WINEPATH="/usr/x86_64-w64-mingw32/bin" wine ../../arcdps_mock.exe ./target/x86_64-pc-windows-gnu/debug/clears.dll
end
