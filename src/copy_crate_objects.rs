use crate::log;
use crate::oops;
use crate::opt_str;
use crate::opt_bool;
use crate::opt_str_vec;
use crate::try_create_dir;

use std::io::Error;
use std::io::ErrorKind;
use std::io::Result as IoResult;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::canonicalize;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::copy;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use toml::Value;

use walkdir::WalkDir;

const PRINT_SORTED: bool = true;

pub fn process(config: &Value) {
    let stage = "copy-crate-objects";

    let root = opt_str(config, &["theseus-root"]);
    let build_dir = opt_str(config, &["build-dir"]);

    let kernel_prefix = opt_str(config, &["prefixes", "kernel"]);
    let apps_prefix = opt_str(config, &["prefixes", "applications"]);

    let target = opt_str(config, &["build-cells", "cargo-target-name"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);

    let extra_target_dirs = opt_str_vec(config, &["copy-crate-objects", "extra-target-dirs"]);
    let extra_apps = opt_str_vec(config, &["copy-crate-objects", "extra-apps"]);
    let debug_crates_objects = opt_bool(config, &["copy-crate-objects", "debug-crate-objects"]);

    log!(stage, "discovering crates");

    let objects_dir = format!("{}/isofiles/modules", build_dir);
    let deps_dir = format!("{}/deps", build_dir);
    let sysroot_dir = format!("{}/sysroot/lib/rustlib/{}/lib", deps_dir, target);

    let mut input_dirs = extra_target_dirs;
    input_dirs.push(format!("{}/target", build_dir));

    for target_dir in &mut input_dirs {
        target_dir.push_str(&format!("/{}/{}/deps", target, build_mode));
    }

    let kernel_path = format!("{}/kernel", root);
    let kernel_path_buf = match canonicalize(&kernel_path) {
        Ok(path_buf) => path_buf,
        _ => oops!(stage, "couldn't access {}", &kernel_path),
    };

    let kernel_crates_set = match kernel_path_buf.is_file() {
        true => populate_crates_from_file(kernel_path_buf),
        _    => populate_crates_from_dir(kernel_path_buf),
    }.unwrap_or_else(|e| oops!(stage, "couldn't access {}: {}", &kernel_path, e));

    let apps_path = format!("{}/applications", root);
    let apps_path_buf = match canonicalize(&apps_path) {
        Ok(path_buf) => path_buf,
        _ => oops!(stage, "couldn't access {}", &apps_path),
    };

    let mut apps_crates_set = match apps_path_buf.is_file() {
        true => populate_crates_from_file(apps_path_buf),
        _    => populate_crates_from_dir(apps_path_buf),
    }.unwrap_or_else(|e| oops!(stage, "couldn't access {}: {}", &apps_path, e));

    apps_crates_set.extend(extra_apps);

    let (
        app_object_files,
        kernel_objects_and_deps_files,
        other_objects_and_deps_files,
    ) = parse_input_dir(
        apps_crates_set,
        kernel_crates_set,
        input_dirs,
        debug_crates_objects,
    ).unwrap();

    log!(stage, "copying crate objects");

    // Now that we have obtained the lists of kernel, app, and other crates,
    // we copy their crate object files into the output object directory with the proper prefix.
    copy_files(
        &objects_dir,
        app_object_files.values().map(|d| d.path()),
        &apps_prefix,
        debug_crates_objects,
    ).unwrap();
    copy_files(
        &objects_dir,
        kernel_objects_and_deps_files.values().map(|(obj_direnty, _)| obj_direnty.path()),
        &kernel_prefix,
        debug_crates_objects,
    ).unwrap();
    copy_files(
        &objects_dir,
        other_objects_and_deps_files.values().map(|(obj_direnty, _)| obj_direnty.path()),
        &kernel_prefix,
        debug_crates_objects,
    ).unwrap();

    // Now we do the same kind of copy operation of crate dependency files, namely the .rlib and .rmeta files,
    // into the output deps directory.
    copy_files(
        &deps_dir,
        kernel_objects_and_deps_files.values().flat_map(|(_, deps)| deps.iter()),
        "",
        debug_crates_objects,
    ).unwrap();
    // Currently we also copy non-kernel dependency files just for efficiency in future out-of-tree builds.
    copy_files(
        &deps_dir,
        other_objects_and_deps_files.values().flat_map(|(_, deps)| deps.iter()),
        "",
        debug_crates_objects,
    ).unwrap();

    // Here, if requested, we create the sysroot directory, containing the fundamental Rust libraries 
    // that we ask cargo to build for us for Theseus's custom platform target
    // Currently this comprises core, alloc, compiler_builtins, and rustc_std_workspace_core.
    try_create_dir(&sysroot_dir, true);

    let sysroot_files = other_objects_and_deps_files.iter()
        .filter(|(crate_name, _val)| {
            crate_name.starts_with("core-") || 
            crate_name.starts_with("compiler_builtins-") || 
            crate_name.starts_with("rustc_std_workspace_core-") || 
            crate_name.starts_with("alloc-")
        })
        .flat_map(|(_key, (_, deps))| deps.iter());
    copy_files(
        &sysroot_dir,
        sysroot_files,
        "",
        debug_crates_objects,
    ).unwrap();
}

/// Parses the file as a list of crate names, one per line.
/// 
/// Returns the set of unique crate names. 
fn populate_crates_from_file<P: AsRef<Path>>(file_path: P) -> Result<HashSet<String>, Error> {
    let file = File::open(file_path)?;
    let mut crates: HashSet<String> = HashSet::new();
    for line in BufReader::new(file).lines() {
        if let Some(crate_name) = line?.split("-").next() {
            crates.insert(crate_name.to_string());
        }
    }

    Ok(crates)
}

/// Iterates over the contents of the given directory to find crates within it. 
/// 
/// Crates are discovered by looking for a directory that contains a `Cargo.toml` file. 
/// 
/// Returns the set of unique crate names. 
fn populate_crates_from_dir<P: AsRef<Path>>(dir_path: P) -> Result<HashSet<String>, Error> {
    let mut crates: HashSet<String> = HashSet::new();
    
    let dir_iter = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|res| res.ok());
        // .filter(|entry| entry.path().is_file() && entry.path().extension() == Some(object_file_extension))
        // .filter_map(|entry| entry.path().file_name()
        //     .map(|fname| {
        //         (
        //             fname.to_string_lossy().to_string().into(), 
        //             entry.path().to_path_buf()
        //         )
        //     })
        // );

    for dir_entry in dir_iter {
        if dir_entry.file_type().is_file() && dir_entry.file_name() == "Cargo.toml" {
            // the parent of this dir_entry is a crate directory
            let parent_crate_dir = dir_entry.path().parent().ok_or_else(|| {
                let err_str = format!("Error getting the containing (parent) crate directory of a Cargo.toml file: {:?}", dir_entry.path());
                Error::new(ErrorKind::NotFound, err_str)
            })?;
            let parent_crate_name = parent_crate_dir.file_name().ok_or_else(|| {
                let err_str = format!("Error getting the name of crate directory {:?}", parent_crate_dir);
                Error::new(ErrorKind::NotFound, err_str)
            })?;
            crates.insert(parent_crate_name.to_str().unwrap().to_string());
        }

    }
    Ok(crates)
}


/// A key-value set of crate dependency files, in which 
/// the key is the crate name, and the value is the crate's object file.
type CrateObjectFiles = HashMap<String, DirEntry>;
/// A key-value set of crate dependency files, in which 
/// the key is the crate name, and 
/// the value is a tuple of the crate's `(object file, [.rmeta file, .rlib file])`. 
type CrateObjectAndDepsFiles = HashMap<String, (DirEntry, [PathBuf; 2])>;


const DEPS_PREFIX:     &str = "lib";
const RMETA_EXTENSION: &str = "rmeta";
const RLIB_EXTENSION:  &str = "rlib";


/// Parses the given input directory, which should be the directory of object files built by Rust, 
/// to determine the latest versions of kernel crates, application crates, and other crates.
/// 
/// See the top of this file for more details. 
/// 
/// Upon success, returns a tuple of:
/// * application crate object files,
/// * kernel crate object files,
/// * all other crate object files,
/// * kernel dependency files (.rmeta and .rlib),
/// * all other non-application dependency files (.rmeta and .rlib).
/// 
fn parse_input_dir(
    app_crates: HashSet<String>,
    kernel_crates: HashSet<String>,
    input_dirs: Vec<String>,
    debug_crates_objects: bool,
) -> IoResult<(
    CrateObjectFiles,
    CrateObjectAndDepsFiles,
    CrateObjectAndDepsFiles,
)> {

    let mut app_objects = CrateObjectFiles::new();
    let mut kernel_files = CrateObjectAndDepsFiles::new();
    let mut other_files = CrateObjectAndDepsFiles::new();

    for input_dir in &input_dirs {
        for dir_entry in read_dir(input_dir)? {
            let dir_entry = dir_entry?;
            let metadata = dir_entry.metadata()?;
            if !metadata.is_file() { continue; }
            let file_name = dir_entry.file_name().into_string().unwrap();
            if !file_name.ends_with(".o") { continue; }
            let file_stem = file_name.split(".o").next().expect("object file name didn't have the .o extension");
            let prefix = file_name.split("-").next().expect("object file name didn't have the crate/hash '-' delimiter");
            let modified_time = metadata.modified()?;

            // A closure for calculating paths for .rmeta and .rlib files in the same directory as the given object file.
            let generate_deps_paths = |obj_file: DirEntry| {
                let mut rmeta_path = obj_file.path();
                rmeta_path.set_file_name(format!("{}{}.{}", DEPS_PREFIX, file_stem, RMETA_EXTENSION));
                let mut rlib_path = rmeta_path.clone();
                rlib_path.set_extension(RLIB_EXTENSION);
                (obj_file, [rmeta_path, rlib_path])
            };

            // Check whether the object file is for a crate designated as an application, kernel, or other crate.
            if app_crates.contains(prefix) {
                match app_objects.entry(prefix.to_string()) {
                    Entry::Occupied(mut occupied) => {
                        if occupied.get().metadata()?.modified()? < modified_time {
                            occupied.insert(dir_entry);
                        }
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(dir_entry);
                    }
                }
            } else if kernel_crates.contains(prefix) {
                match kernel_files.entry(prefix.to_string()) {
                    Entry::Occupied(mut occupied) => {
                        if occupied.get().0.metadata()?.modified()? < modified_time {
                            occupied.insert(generate_deps_paths(dir_entry));
                        }
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(generate_deps_paths(dir_entry));
                    }
                }
            } else {
                other_files.insert(file_stem.to_string(), generate_deps_paths(dir_entry));
            }

        }
    }

    // optional debug output
    if debug_crates_objects {
        println!("APPLICATION OBJECT FILES:");
        print_crates_objects(&app_objects, PRINT_SORTED);
        println!("KERNEL OBJECT FILES AND DEPS FILES:");
        print_crates_objects_and_deps(&kernel_files, PRINT_SORTED);
        println!("OTHER OBJECT FILES AND DEPS FILES:");
        print_crates_objects_and_deps(&other_files, PRINT_SORTED);
    }
    
    Ok((
        app_objects,
        kernel_files,
        other_files,
    ))
}


/// Copies each file in the `files` iterator into the given `output_dir`.
///
/// Prepends the given `prefix` onto the front of the output file names.
/// 
/// Ignores any source files in the `files` iterator that do not exist. 
/// This is a policy choice due to how we form paths for deps files, which may not actually exist. 
fn copy_files<'p, O, P, I>(
    output_dir: O,
    files: I,
    prefix: &str,
    debug_crates_objects: bool,
) -> IoResult<()> 
    where O: AsRef<Path>,
          P: AsRef<Path>,
          I: Iterator<Item = P>,
{
    for source_path_ref in files {
        let source_path = source_path_ref.as_ref();
        let mut dest_path = output_dir.as_ref().to_path_buf();
        dest_path.push(format!("{}{}", prefix, source_path.file_name().and_then(|osstr| osstr.to_str()).unwrap()));

        if debug_crates_objects {
            println!("Copying {} to {}", source_path.display(), dest_path.display());
        }
            
        match copy(source_path, dest_path) {
            Ok(_bytes_copied) => { }
            Err(e) if e.kind() == ErrorKind::NotFound => { }  // Ignore source files that don't exist
            Err(other_err) => return Err(other_err),
        }
    }
    Ok(())
}



fn print_crates_objects(objects: &CrateObjectFiles, sorted: bool) {
    if sorted {
        let mut sorted = objects.keys().collect::<Vec<&String>>();
        sorted.sort_unstable();
        for o in &sorted {
            println!("\t{}", o);
        }
    } else {
        for (k, v) in objects.iter() {
            println!("\t{} --> {}", k, v.path().display());
        }
    }
}


fn print_crates_objects_and_deps(files: &CrateObjectAndDepsFiles, sorted: bool) {
    if sorted {
        let mut sorted = files.keys().collect::<Vec<&String>>();
        sorted.sort_unstable();
        for o in &sorted {
            println!("\t{}", o);
        }
    } else {
        for (k, v) in files.iter() {
            println!("\t{} --> {}, {}, {}", k, v.0.path().display(), v.1[0].display(), v.1[1].display());
        }
    }
}
