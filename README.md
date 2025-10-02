# Blur Package Manager (bpm)

**bpm** is a simple, local, binary-based package manager written in **Rust** with support for dependency resolution.  
It uses `.mri` (**Magnet Remote Installer**) files in TOML format to describe available packages and their dependencies.  

---

## 📦 Features
- Local **binary-based** package management.
- `.mri` TOML repositories for easy package definitions.
- Dependency resolution (installs required packages first).
- Uses [`calcbits`](https://crates.io/crates/calcbits) for progress bars and storage. 
- Installed packages are tracked in `installed.json`.
- Supports multiple repositories.
- Works completely **offline** (local paths as repos). for now...

---

## 📂 Directory Layout
By default, bpm lives in:

```

C:/Users/User/Bpm-Store/
├── bins/               # Installed binaries go here
├── main/               # Example repo
│   └── packages.mri    # Repo index (TOML)
├── installed.json      # Installed packages DB
└── packages.db         # Binary storage (via calcbits)

````

---

## 📑 Example `.mri` File
Example `packages.mri`:

```toml
[neovim."0.9.0"]
path = "C:/Users/User/Bpm-Store/main/neovim-0.9.0"
binaries = ["nvim.exe"]
dependencies = ["libuv:1.0.0"]

[libuv."1.0.0"]
path = "C:/Users/User/Bpm-Store/main/libuv-1.0.0"
binaries = ["libuv.dll"]
````

This means:

* Installing `main:neovim` will automatically install `libuv:1.0.0` first.

---

## ⚡ Usage

```
bpm /i <repo:package[:version]>   # Install a package
bpm /r <package>                  # Remove a package
bpm /u <repo:package>             # Update package to latest version
bpm /l                            # List installed packages
bpm /v                            # Show bpm version
bpm /h                            # Help menu
```

### Examples

```sh
bpm /i main:neovim         # Install latest version of neovim from 'main' repo
bpm /i main:neovim:0.9.0   # Install specific version
bpm /r neovim              # Remove package
bpm /u main:neovim         # Update neovim
bpm /l                     # List installed packages
```

---

## 🛠 Development

To build bpm from source:

```sh
git clone https://github.com/Kazan20/bpm.git
cd bpm
cargo build --release
```

Binary will be at:

```
target/release/bpm.exe
```

---

## 🔮 Roadmap

* ✅ Dependency resolution
* ⏳ Remote repo fetching (git/http)
* ⏳ Better error messages
* ⏳ Search command for packages

---

## 📜 License

MIT License.

check out [`calcbits`](https://github.com/Kazan20/calcbits) made by me