/* This is some more or less good boilerplate for my future projects, so i decided to put this here, for ease of finding and using. This is logic for lockfiles */

fn app_support_path() -> String {
    unsafe {
        let paths = NSSearchPathForDirectoriesInDomains(
            NSSearchPathDirectory::ApplicationSupportDirectory,
            NSSearchPathDomainMask::UserDomainMask,
            true,
        );
        let path = paths.firstObject().unwrap();
        let path_str = path.to_string();
        let bundlename = "runner";
        format!("{}/{}/", path_str, bundlename)
    }
}
fn check_for_appsupport_dir(path: String) -> String {
    let file_path = format!("{}/lockfile.lock", path);
    if !Path::new(path.as_str()).exists() {
        let _ = create_dir(&path); // Create the application directory if it doesn't exist
    }
    file_path
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_path = app_support_path();
    let lock_path = check_for_appsupport_dir(app_path);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)?;
    let mut f = RwLock::new(file);

    let guard = match f.try_write() {
        Ok(guard) => {
            println!("First process...");
            guard // keep it alive
        }
        Err(_) => {
            println!("Second process...");
            std::process::exit(0); // exit immediately
        }
    };
}
