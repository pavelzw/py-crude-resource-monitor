use std::process::Command;

fn main() {
    #[cfg(not(windows))]
    let pnpm_executable = "pnpm";
    #[cfg(windows)]
    let pnpm_executable = "pnpm.cmd";
    Command::new(pnpm_executable)
        .args(["install"])
        .current_dir("frontend")
        .status()
        .expect("Failed to run pnpm install");

    Command::new(pnpm_executable)
        .args(["build"])
        .current_dir("frontend")
        .status()
        .expect("Failed to run pnpm build");

    println!("cargo::rerun-if-changed=frontend/src");
    println!("cargo::rerun-if-changed=frontend/package.json");
    println!("cargo::rerun-if-changed=frontend/tsconfig.json");
    println!("cargo::rerun-if-changed=frontend/index.html");
    println!("cargo::rerun-if-changed=build.rs");
}
