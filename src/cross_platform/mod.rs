pub mod get;

struct TextEditor {
    name: String,
    support: Vec<String>,
}

pub static TERMINAL_EDITORS: &[&str] = &[
    // Classic/Vi-family
    "vi",
    "vim",
    "nvim", // "neovim" is the package, "nvim" is the binary
    "ex",
    "view",
    "rvim",
    "rview",
    "elvis",
    "nvi",
    "vile",
    "xvi",
    "levee",
    // Emacs-family
    "emacs",
    "jove",
    "mg",
    "zile",
    "uemacs",
    "qemacs",
    // Nano/Pico family
    "nano",
    "pico",
    // Modern terminal editors
    "micro",
    "kak",   // "kakoune" is the package, "kak" is the binary
    "helix", // valid on some distros
    "hx",    // valid on others (Arch, macOS)
    "amp",
    "vis",
    // Lightweight/Simple
    "ed",
    "red",
    "ee",
    "edit", // common symlink on Debian/Ubuntu
    "ae",
    "le",
    "ne",
    "teco",
    // Programmer's editors
    "joe",
    "jmacs",
    "jpico",
    "jstar",
    "rjoe",
    "mcedit",
    "mcview",
    "cooledit",
    "mined",
    // WordStar family
    "wordgrinder",
    // Other notable ones
    "slap",
    "tilde",
    "vy",
    "iota",
    // Rust-based specifically
    "kibi",
    "scrawl",
    "ox",
    "babi",
    // Special purpose (Hex editors)
    "hexedit",
    "hexer",
    "bvi",
    "dhex",
    // Historical/Obscure
    "qed",
    "edlin",
    // Distributions (that provide a unique binary)
    "lvim", // LunarVim binary
];
