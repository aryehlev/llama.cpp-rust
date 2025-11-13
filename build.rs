use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Get llama.cpp version from environment or use default
    let version = get_llama_version();
    let version_num = parse_version(&version);
    println!(
        "cargo:warning=Using llama.cpp version: {} ({})",
        version, version_num
    );

    // Emit version cfg flags for conditional compilation
    emit_version_cfgs(version_num);

    // Get platform information
    let (os, arch) = get_platform_info();
    println!("cargo:warning=Platform: {} {}", os, arch);

    // Download llama.cpp headers
    let llama_include_root = out_dir.join("llama_include");
    download_llama_headers(&llama_include_root, &version, version_num)
        .expect("Failed to download llama.cpp headers");

    // Download and extract pre-built binaries
    let libs_dir = out_dir.join("libs");
    fs::create_dir_all(&libs_dir).expect("Failed to create libs directory");

    download_and_extract_binary(&libs_dir, &version, &os, &arch)
        .expect("Failed to download llama.cpp binary");

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", llama_include_root.display()))
        .clang_arg(format!(
            "-I{}",
            llama_include_root.join("include").display()
        ))
        .allowlist_function("llama_.*")
        .allowlist_function("ggml_.*")
        .allowlist_type("llama_.*")
        .allowlist_type("ggml_.*")
        .allowlist_var("LLAMA_.*")
        .allowlist_var("GGML_.*")
        .derive_debug(true)
        .derive_default(true)
        .generate_comments(false)
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings");

    // Set up linking
    configure_linking(&libs_dir, &os);
}

fn emit_version_cfgs(version_num: u32) {
    // Declare all possible cfg flags for Cargo check-cfg lint
    println!("cargo:rustc-check-cfg=cfg(llama_vocab_api)");

    // Emit cfg flags for version-dependent features
    // Only emit flags that are actually used in the code

    // Vocab API separated from model in b6739
    if version_num >= 6739 {
        println!("cargo:rustc-cfg=llama_vocab_api");
    }

    // Note: We can add more version-specific cfg flags here as needed when
    // new breaking changes occur in llama.cpp
}

fn get_llama_version() -> String {
    env::var("LLAMA_CPP_VERSION").unwrap_or_else(|_| "b7035".to_string())
}

fn parse_version(version: &str) -> u32 {
    // Extract numeric part from version string like "b7035" -> 7035
    version
        .trim_start_matches('b')
        .trim_start_matches('v')
        .parse::<u32>()
        .unwrap_or(7035)
}

fn get_platform_info() -> (String, String) {
    let target = env::var("TARGET").unwrap();
    let parts: Vec<&str> = target.split('-').collect();

    let arch = parts[0];
    let os = if target.contains("darwin") {
        "macos"
    } else if target.contains("linux") {
        "ubuntu"
    } else if target.contains("windows") {
        "windows"
    } else {
        panic!("Unsupported platform: {}", target);
    };

    (os.to_string(), arch.to_string())
}

fn download_llama_headers(out_dir: &Path, version: &str, version_num: u32) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;
    let include_dir = out_dir.join("include");
    fs::create_dir_all(&include_dir)?;

    // Core headers present in all versions
    let mut headers = vec![
        ("include/llama.h", "llama.h", 0),
        ("ggml/include/ggml.h", "ggml.h", 0),
    ];

    // Version-specific headers
    if version_num >= 5000 {
        headers.push(("ggml/include/ggml-cpu.h", "ggml-cpu.h", 5000));
        headers.push(("ggml/include/ggml-backend.h", "ggml-backend.h", 5000));
        headers.push(("ggml/include/ggml-alloc.h", "ggml-alloc.h", 5000));
    }

    if version_num >= 6000 {
        headers.push(("ggml/include/ggml-opt.h", "ggml-opt.h", 6000));
    }

    for (remote_path, local_name, min_version) in headers {
        let url = format!(
            "https://raw.githubusercontent.com/ggml-org/llama.cpp/{}/{}",
            version, remote_path
        );
        let dest = include_dir.join(local_name);

        // Skip if already downloaded
        if dest.exists() {
            println!("cargo:warning=Header already exists: {}", dest.display());
            continue;
        }

        // Try to download, but don't fail for optional newer headers
        match download_file(&url, &dest) {
            Ok(_) => {}
            Err(e) if min_version > 0 => {
                println!(
                    "cargo:warning=Optional header {} not available for version {} (requires {}+): {}",
                    local_name, version_num, min_version, e
                );
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

fn download_file(url: &str, dest: &Path) -> io::Result<()> {
    println!("cargo:warning=Downloading: {}", url);

    let response = ureq::get(url)
        .call()
        .map_err(|e| io::Error::other(format!("HTTP request failed: {}", e)))?;

    let mut content = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut content)
        .map_err(|e| io::Error::other(format!("Failed to read response: {}", e)))?;

    let mut file = fs::File::create(dest)?;
    file.write_all(&content)?;

    println!("cargo:warning=Downloaded to: {}", dest.display());
    Ok(())
}

fn download_and_extract_binary(
    libs_dir: &Path,
    version: &str,
    os: &str,
    arch: &str,
) -> io::Result<()> {
    // Determine the release file name based on platform
    let (release_name, lib_name) = match (os, arch) {
        ("macos", "aarch64") => (
            format!("llama-{}-bin-macos-arm64.zip", version),
            "libllama.dylib",
        ),
        ("macos", "x86_64") => (
            format!("llama-{}-bin-macos-x64.zip", version),
            "libllama.dylib",
        ),
        ("ubuntu", "x86_64") => (
            format!("llama-{}-bin-ubuntu-x64.zip", version),
            "libllama.so",
        ),
        ("windows", "x86_64") => (format!("llama-{}-bin-win-cpu-x64.zip", version), "llama.dll"),
        ("windows", "aarch64") => (format!("llama-{}-bin-win-cpu-arm64.zip", version), "llama.dll"),
        _ => panic!("Unsupported platform combination: {} {}. Only x86_64 ubuntu/macos/windows and aarch64 macos/windows are supported.", os, arch),
    };

    // Check if library already exists
    let lib_path = libs_dir.join(lib_name);
    if lib_path.exists() {
        println!(
            "cargo:warning=Library already exists: {}",
            lib_path.display()
        );
        return Ok(());
    }

    // Download release from GitHub
    let url = format!(
        "https://github.com/ggml-org/llama.cpp/releases/download/{}/{}",
        version, release_name
    );

    println!("cargo:warning=Downloading binary from: {}", url);

    let response = ureq::get(&url)
        .call()
        .map_err(|e| io::Error::other(format!("Failed to download: {}", e)))?;

    let mut zip_data = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut zip_data)
        .map_err(|e| io::Error::other(format!("Failed to read: {}", e)))?;

    // Extract the library from the zip
    let cursor = io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| io::Error::other(format!("Invalid zip: {}", e)))?;

    // Find and extract all library files (libllama + libggml)
    let mut found_main_lib = false;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| io::Error::other(format!("Zip error: {}", e)))?;

        let file_name = file.name().to_string();

        // Check if this is a library file we need
        let is_target = if os == "windows" {
            file_name.ends_with(".dll") || file_name.ends_with(".lib")
        } else if os == "macos" {
            file_name.ends_with(".dylib")
        } else {
            file_name.ends_with(".so") || file_name.contains(".so.")
        };

        if is_target && (file_name.contains("llama") || file_name.contains("ggml")) {
            let lib_filename = Path::new(&file_name).file_name().unwrap().to_str().unwrap();
            let dest_path = libs_dir.join(lib_filename);

            println!(
                "cargo:warning=Extracting: {} -> {}",
                file_name,
                dest_path.display()
            );

            let mut lib_content = Vec::new();
            file.read_to_end(&mut lib_content)?;

            let mut out_file = fs::File::create(&dest_path)?;
            out_file.write_all(&lib_content)?;

            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms)?;
            }

            println!(
                "cargo:warning=Extracted library to: {}",
                dest_path.display()
            );

            if file_name.contains("llama") {
                found_main_lib = true;
            }
        }
    }

    if !found_main_lib {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Main library {} not found in archive", lib_name),
        ));
    }

    // On Windows, generate .lib import libraries from .dll files
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = generate_windows_import_libs(libs_dir) {
            eprintln!("\n=======================================================================");
            eprintln!("ERROR: Failed to generate Windows import libraries");
            eprintln!("=======================================================================");
            eprintln!("{}", e);
            eprintln!("\nThis requires Windows SDK tools (dumpbin.exe and lib.exe).");
            eprintln!("\nTo fix this, install one of:");
            eprintln!("  1. Visual Studio (with C++ development tools)");
            eprintln!("  2. Build Tools for Visual Studio");
            eprintln!("     Download: https://visualstudio.microsoft.com/downloads/");
            eprintln!("\nMake sure the tools are in your PATH, or run from a");
            eprintln!("'Developer Command Prompt for VS' / 'x64 Native Tools Command Prompt'");
            eprintln!("=======================================================================\n");
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn generate_windows_import_libs(libs_dir: &Path) -> io::Result<()> {
    use std::process::Command;

    let dll_path = libs_dir.join("llama.dll");
    let lib_path = libs_dir.join("llama.lib");
    let def_path = libs_dir.join("llama.def");

    if !dll_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "llama.dll not found",
        ));
    }

    // Skip if .lib already exists
    if lib_path.exists() {
        return Ok(());
    }

    println!("cargo:warning=Generating llama.lib import library from llama.dll");

    // Step 1: Use dumpbin to get exports
    let dumpbin_output = Command::new("dumpbin")
        .args(&["/EXPORTS", dll_path.to_str().unwrap()])
        .output();

    match dumpbin_output {
        Ok(output) if output.status.success() => {
            // Step 2: Parse dumpbin output and create .def file
            let exports_str = String::from_utf8_lossy(&output.stdout);
            let mut exports = Vec::new();

            let mut in_exports = false;
            for line in exports_str.lines() {
                if line.contains("ordinal hint") {
                    in_exports = true;
                    continue;
                }
                if in_exports {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        exports.push(parts[3].to_string());
                    }
                }
            }

            if exports.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No exports found in DLL",
                ));
            }

            // Step 3: Write .def file
            let mut def_content = String::from("EXPORTS\n");
            for export in exports {
                def_content.push_str(&format!("    {}\n", export));
            }
            fs::write(&def_path, def_content)?;

            println!("cargo:warning=Created {}", def_path.display());

            // Step 4: Use lib.exe to create import library
            let lib_output = Command::new("lib")
                .args(&[
                    &format!("/DEF:{}", def_path.display()),
                    &format!("/OUT:{}", lib_path.display()),
                    "/MACHINE:X64",
                ])
                .current_dir(libs_dir)
                .output();

            match lib_output {
                Ok(result) if result.status.success() => {
                    println!("cargo:warning=Successfully generated {}", lib_path.display());
                    // Clean up .def file
                    let _ = fs::remove_file(&def_path);
                    Ok(())
                }
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("lib.exe failed: {}", stderr),
                    ))
                }
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("lib.exe not found: {}", e),
                )),
            }
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("dumpbin failed: {}", stderr),
            ))
        }
        Err(e) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("dumpbin not found: {}", e),
        )),
    }
}

fn configure_linking(libs_dir: &Path, os: &str) {
    // Tell cargo where to find the library
    println!("cargo:rustc-link-search=native={}", libs_dir.display());

    // Link the library
    println!("cargo:rustc-link-lib=dylib=llama");

    // Copy ALL libraries to target directory for runtime
    let target_dir = get_target_dir();

    // Copy all library files from libs_dir to target_dir
    if let Ok(entries) = fs::read_dir(libs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_str().unwrap();

                // Check if it's a library file
                let is_lib = match os {
                    "macos" => filename_str.ends_with(".dylib"),
                    "ubuntu" => filename_str.ends_with(".so") || filename_str.contains(".so."),
                    "windows" => filename_str.ends_with(".dll"),
                    _ => false,
                };

                if is_lib {
                    let dest = target_dir.join(filename);
                    if fs::copy(&path, &dest).is_ok() {
                        println!(
                            "cargo:warning=Copied {} to {}",
                            path.display(),
                            dest.display()
                        );
                    }
                }
            }
        }
    }

    // Configure rpath for runtime library loading
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../..");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../..");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", libs_dir.display());

        // Try to modify install name for all dylibs
        if let Ok(entries) = fs::read_dir(libs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    if filename.to_str().unwrap().ends_with(".dylib") {
                        let _ = std::process::Command::new("install_name_tool")
                            .arg("-id")
                            .arg(format!("@loader_path/{}", filename.to_str().unwrap()))
                            .arg(&path)
                            .output();
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../..");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", libs_dir.display());
    }

    // Rerun if wrapper.h changes
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_VERSION");
}

fn get_target_dir() -> PathBuf {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Navigate up from OUT_DIR to find target/{profile}
    let mut current = out_path;
    while let Some(parent) = current.parent() {
        if parent.ends_with("target") {
            if let Some(profile_dir) = current.file_name() {
                return parent.join(profile_dir);
            }
        }
        current = parent;
    }

    // Fallback: try to find target/debug or target/release
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = Path::new(&manifest_dir).join("target");

    if env::var("PROFILE").unwrap() == "release" {
        target.join("release")
    } else {
        target.join("debug")
    }
}
