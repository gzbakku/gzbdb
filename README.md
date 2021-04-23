

this is a the base gzbdb written in rust.

##use openssl on windows

vcpkg installation

-cloned git
-run bootstrap-vcpkg.bat

vcpkg is accessible from cli
open ssl installation

-run vcpkg install openssl-windows:x64-windows
-run vcpkg install openssl:x64-windows-static
-run vcpkg integrate install
vcpkg environment vars

VCPKGRS_DYNAMIC=1 VCPKG_ROOT=vcpkg_dir_path