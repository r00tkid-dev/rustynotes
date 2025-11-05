```
 ________  ___  ___  ________  _________    ___    ___   
|\   __  \|\  \|\  \|\   ____\|\___   ___\ |\  \  /  /|  
\ \  \|\  \ \  \\\  \ \  \___|\|___ \  \_| \ \  \/  / /  
 \ \   _  _\ \  \\\  \ \_____  \   \ \  \   \ \    / /   
  \ \  \\  \\ \  \\\  \|____|\  \   \ \  \   \/  /  /    
   \ \__\\ _\\ \_______\____\_\  \   \ \__\__/  / /      
    \|__|\|__|\|_______|\_________\   \|__|\___/ /       
                       \|_________|       \|___|/        
                                                          
                                                          
 ________   ________  _________  _______   ________      
|\   ___  \|\   __  \|\___   ___\\  ___ \ |\   ____\     
\ \  \\ \  \ \  \|\  \|___ \  \_\ \   __/|\ \  \___|_    
 \ \  \\ \  \ \  \\\  \   \ \  \ \ \  \_|/_\ \_____  \   
  \ \  \\ \  \ \  \\\  \   \ \  \ \ \  \_|\ \|____|\  \  
   \ \__\\ \__\ \_______\   \ \__\ \ \_______\____\_\  \ 
    \|__| \|__|\|_______|    \|__|  \|_______|\_________\
                                             \|_________|
                                              - by r00tkid

A minimalist approach to note-taking necessities in the command-line. 
```

## Installation

### Pre-built Binaries (Recommended)

Navigate to the [Releases](https://github.com/r00tkid-dev/rustynotes/releases) page and download the appropriate binary for your operating system:

- **Linux**: `rustynotes-linux-x86_64.tar.gz`
- **macOS**: `rustynotes-macos-x86_64.tar.gz` or `rustynotes-macos-arm64.tar.gz`
- **Windows**: `rustynotes-windows-x86_64.zip`

### Quick Install

```bash
# Linux/macOS
curl -L https://github.com/r00tkid-dev/rustynotes/releases/latest/download/rustynotes-linux-x86_64.tar.gz | tar xz
sudo mv rustynotes /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest https://github.com/r00tkid-dev/rustynotes/releases/latest/download/rustynotes-windows-x86_64.zip -OutFile rustynotes.zip
Expand-Archive rustynotes.zip
Move-Item rustynotes.exe $env:USERPROFILE\AppData\Local\Microsoft\WindowsApps\
```

### Build from Source

#### Prerequisites
- [Rust 1.70+](https://rustup.rs/)
- Git

```bash
git clone https://github.com/r00tkid-dev/rustynotes.git
cd rustynotes
cargo build --release
```

The binary will be located at `target/release/rustynotes`.

## Usage

### Running the Application

```bash
# If installed system-wide
rustynotes

# Or if running from local directory
./rustynotes
```

### First Time Setup
On first run, rustynotes creates a `~/.notes` directory to store your notes.

## 📋 Commands

### Core Operations
```bash
:quit               ► exit rustynotes
:n  / :n!           ► new note (with/without warning)
:save [name]        ► save note (with optional name)
:load [name]        ► load note
:ls                 ► list saved notes
:list               ► show current note
```

### Organization & Search
```bash
:tag [name]         ► add tag to current note
:tags               ► list all tags
:tagged [tag]       ► list notes with specific tag
:search [keyword]   ► search for keyword
```

### Editing
```bash
:ml                 ► start/end multi-line input
:edit               ► start edit mode
  :line N           ► select line to edit
  :save             ► save changes
  :cancel           ► discard changes
```

### Analytics
```bash
:stats              ► show note statistics with cool banner
```

### Help
```bash
:help               ► show this help message
```

## Development & Building

### Local Development
```bash
# Clone the repository
git clone https://github.com/r00tkid-dev/rustynotes.git
cd rustynotes

# Install dependencies and build
cargo build

# Run tests
cargo test

# Run with debug output
cargo run
```

### Cross-Platform Compilation

#### Using Cargo (Simple)
```bash
# Build for current platform
cargo build --release

# Build with optimizations (smaller binary)
cargo build --release
```

#### Using Cross (Recommended for releases)
First install [cross-rs](https://github.com/cross-rs/cross):

```bash
cargo install cross
```

Then build for different targets:

```bash
# Linux x86_64
cross build --target x86_64-unknown-linux-gnu --release

# Windows x86_64
cross build --target x86_64-pc-windows-gnu --release

# macOS x86_64 (requires macOS SDK)
cross build --target x86_64-apple-darwin --release

# macOS ARM64 (Apple Silicon)
cross build --target aarch64-apple-darwin --release
```

### Creating Release Packages

#### Linux
```bash
#!/bin/bash
TARGET=x86_64-unknown-linux-gnu
cross build --target $TARGET --release
mkdir -p release
cp target/$TARGET/release/rustynotes release/
tar -czf rustynotes-linux-x86_64.tar.gz -C release rustynotes
```

#### Windows
```bash
#!/bin/bash
TARGET=x86_64-pc-windows-gnu
cross build --target $TARGET --release
mkdir -p release
cp target/$TARGET/release/rustynotes.exe release/
cd release
zip rustynotes-windows-x86_64.zip rustynotes.exe
```

#### macOS
```bash
#!/bin/bash
# Intel Macs
TARGET=x86_64-apple-darwin
cross build --target $TARGET --release
mkdir -p release
cp target/$TARGET/release/rustynotes release/
tar -czf rustynotes-macos-x86_64.tar.gz -C release rustynotes

# Apple Silicon
TARGET=aarch64-apple-darwin
cross build --target $TARGET --release
cp target/$TARGET/release/rustynotes release/rustynotes-arm64
tar -czf rustynotes-macos-arm64.tar.gz -C release rustynotes-arm64
```

## Performance

- **Binary Size**: ~800KB (optimized release build)
- **Memory Usage**: <5MB during normal operation
- **Startup Time**: <50ms on modern hardware
- **Dependencies**: Only 3 minimal crates (dirs, chrono, rustyline)

## File Structure

```
~/.notes/                    # Default notes directory
├── note_20241104_143022.md   # Auto-generated timestamp notes
├── my-note.md               # Named notes
└── project-ideas.md          # Your custom notes
```

### Note Format
Notes are stored as Markdown files with optional YAML frontmatter for tags:

```markdown
---
tags: rust, cli, tools
---

# My Note Title

This is the content of my note.

- Bullet point 1
- Bullet point 2
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Roadmap

- [ ] Persistent command history
- [ ] Find and replace functionality
- [ ] Note templates
- [ ] Fuzzy search
- [ ] Export to different formats
- [ ] Quick capture mode

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Built with Rust  
Lightning fast, minimalist, and actually useful