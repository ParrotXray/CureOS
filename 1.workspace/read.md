## Preparing the development environment

### Task list
- Building a pure GCC cross-compilation toolchain (i686-elf), not bundled with the system.
- Building/Installing Bochs and QEMU.

### Compiling gcc-i686-elf
Open the terminal and execute the [gcc-build.sh](gcc-build.sh) script; this may take about half an hour.
```sh=
./gcc-build.sh
```
**Permanently add the newly compiled GCC to the PATH environment variable**

Permanently add the newly compiled GCC to the PATH environment variable. The configuration file depends on the shell you are using, such as `${HOME}/.zshrc` for zsh or `${HOME}/.bashrc` for bash. Alternatively, you can directly modify the global `/etc/profile` file (though it's not recommended). Afterward, reboot your system.
```sh=
vim ~/.bashrc
```
Add `export PATH="$HOME/cross-compiler/bin:$PATH"`

**Test the command**
```sh=
i686-elf-gcc --version
```
output
```sh=
i686-elf-gcc (GCC) 11.2.0
Copyright (C) 2021 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
```

