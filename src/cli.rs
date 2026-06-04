use std::path::PathBuf;

#[derive(Debug, Default)]
pub enum LaunchArg {
    #[default]
    None,
    File(PathBuf),
    Dir(PathBuf),
    Error(String),
}

/// Parse `std::env::args`, skipping the binary name.
/// Resolves relative paths against the current working directory.
pub fn parse_args() -> LaunchArg {
    let mut args = std::env::args().skip(1);

    let Some(first) = args.next() else {
        return LaunchArg::None;
    };

    // Consume remaining args to detect unexpected extras
    let extra = args.next();

    if first == "--help" || first == "-h" {
        print_help();
        std::process::exit(0);
    }
    if first == "--version" || first == "-V" {
        println!("md-reader {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    if extra.is_some() {
        return LaunchArg::Error("Too many arguments. Usage: md-reader [FILE|DIR]".to_string());
    }

    // Resolve against cwd so that `md-reader ./docs` works from any directory
    let path = cwd_join(&first);

    if path.is_file() {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "md" | "markdown" | "txt") {
            LaunchArg::File(path)
        } else {
            LaunchArg::Error(format!("Not a Markdown file: {}", path.display()))
        }
    } else if path.is_dir() {
        LaunchArg::Dir(path)
    } else {
        LaunchArg::Error(format!("Path not found: {}", path.display()))
    }
}

fn cwd_join(s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map(|d| d.join(&p)).unwrap_or(p)
    }
}

fn print_help() {
    let ver = env!("CARGO_PKG_VERSION");
    println!("md-reader {ver}");
    println!();
    println!("USAGE:");
    println!("  md-reader [FILE|DIR]");
    println!();
    println!("ARGUMENTS:");
    println!("  FILE   Open a Markdown file (.md / .markdown / .txt)");
    println!("  DIR    Open a directory as the file tree (auto-opens README.md if present)");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help     Print this help");
    println!("  -V, --version  Print version");
    println!();
    println!("KEYBOARD SHORTCUTS:");
    println!("  Ctrl+O          Open file");
    println!("  Ctrl+F          Search");
    println!("  Ctrl+Shift+P    Command palette");
    println!("  Ctrl++/-/0      Zoom in / out / reset");
}
