use std::env;
use std::process::Command;

const ASM_DIR: &str = "src/impl/x86_64";

#[cfg(not(target_arch = "x86_64"))]
fn assemble_file(file: &str, output_dir: &str) {}

#[cfg(not(target_arch = "x86_64"))]
fn assemble_all(output_dir: &str) {}

#[cfg(target_arch = "x86_64")]
fn assemble_file(file: &str, output_dir: &str)
{
    let out_file = file.replace(".asm", ".o");
    let file = format!("{}/{}", ASM_DIR, file);
    println!("cargo:rerun-if-changed={}", file);

    let status = if env::var_os("DEBUG").is_some() && env::var("DEBUG").unwrap() == "true"
    {
        Command::new("nasm")
            .arg(file)
            .arg("-o")
            .arg(format!("{}/{}", output_dir, out_file))
            .arg("-g")
            .arg("-f")
            .arg("elf64")
            .arg("-F")
            .arg("Dwarf")
            .status()
            .expect("couldn't run nasm")
    }
    else
    {
        Command::new("nasm")
            .arg(file)
            .arg("-o")
            .arg(format!("{}/{}", output_dir, out_file))
            .arg("-f")
            .arg("elf64")
            .arg("-F")
            .arg("Dwarf")
            .status()
            .expect("couldn't run nasm")
    };

    if !status.success()
    {
        panic!("nasm failed with {}", status);
    }

    println!("cargo:rustc-link-lib=static={}", out_file);
}

#[cfg(target_arch = "x86_64")]
fn assemble_all(output_dir: &str)
{
    let files = vec!
    [
        "boot/header.asm",
        "boot/main.asm",
        "boot/main64.asm",
    ];

    for f in files.iter()
    {
        assemble_file(f, output_dir);
    }
}

fn main()
{
    let output_dir = "build/x86_64/";

    println!("cargo:rustc-link-search={}", output_dir);

    assemble_all(&output_dir);
}