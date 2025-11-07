/* This is some more or less good boilerplate for my future projects, it's just not used in this project. This is logic for lockfiles */

fn app_support_path() -> String {
    unsafe {
        let paths = NSSearchPathForDirectoriesInDomains(
            NSSearchPathDirectory::ApplicationSupportDirectory,
            NSSearchPathDomainMask::UserDomainMask,
            true,
        );
        let path = paths.firstObject().unwrap();
        let path_str = path.to_string();
        format!("{}/runner/", path_str)
    }
}
fn check_for_appsupport_dir(path: String) -> String {
    let file_path = format!("{}/lock.lock", path);
    if !Path::new(path.as_str()).exists() {
        let _ = fs::create_dir(&path); // Create the application directory if it doesn't exist
        let _ = fs::File::create(&file_path); // using create_new isn't necessarry, as if the file already exists right after creating the directory, the user might have other, more interesting problems...
    }
    file_path
}

fn lock(as_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let lock_path = check_for_appsupport_dir(as_path);
    let mut f = RwLock::new(File::open(lock_path)?);

    let result = f.try_write();
    match result {
        Ok(_i) => (),
        Err(_e) => {}
    }
    Ok(())
}
