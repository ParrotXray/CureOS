## Getting Started
First, read [docs](docs/Build.md) to learn how to set up the development environment.
## Installation and Execution
#### System Requirements
- **Ubuntu 22.04/24.04**: Can be compiled and run directly.
- **Windows 10/11**: Requires WSL to be enabled for compilation and execution.
### Build from Source
1. **Clone the Repository**
```bash
git clone https://github.com/ParrotXray/CureOS.git
cd CureOS
```
2. **Install Required Packages**
- Install Rust
```bash=
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- Install the nightly toolchain (required for OS development features)
```bash= 
rustup toolchain install nightly
rustup default nightly 
```
- Add i686 targets
```bash= 
rustup target add i686-unknown-linux-gnu
rustup target add i686-pc-windows-gnu
```
- Install required components
```bash= 
rustup component add rust-src
rustup component add llvm-tools-preview
cargo install bootimage
cargo install cargo-xbuild
```
3. **Run (Build / Debug / Run)**
- Build
> release
> ```bash=  
> make all
> ```

> debug
> ```bash=  
> make all-debug
> ```

- Debug
> qemu
> ```bash=
> make debug-qemu
> ```

> bochs
> ```bash=
> make debug-bochs
> ```

- Run
 ```bash=
make run
```
