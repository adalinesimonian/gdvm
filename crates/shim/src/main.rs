use std::path::PathBuf;
use std::process::Command;

fn main() {
    let exe = std::env::current_exe().expect("failed to get current exe");

    let exe_stem = exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let gdvm_name = if cfg!(target_os = "windows") {
        "gdvm.exe"
    } else {
        "gdvm"
    };

    let gdvm_path = exe
        .parent()
        .unwrap_or(PathBuf::new().as_path())
        .join(gdvm_name);

    let mut cmd = Command::new(gdvm_path);

    cmd.env("GDVM_ALIAS", &exe_stem);
    cmd.args(std::env::args().skip(1));

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let err = cmd.exec();
        eprintln!("failed to exec gdvm: {err}");
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        let status = cmd.status().expect("failed to spawn gdvm");
        std::process::exit(status.code().unwrap_or(1));
    }
}
