use dispatch::Queue;
use fd_lock::*;
use objc2_app_kit::NSTextStorageEditActions;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::{fs::*, os::unix::net::UnixStream};

mod search;
mod window;

use objc2_foundation::{
    NSSearchPathDirectory, NSSearchPathDomainMask, NSSearchPathForDirectoriesInDomains,
};

fn app_support_path() -> String {
    let paths = NSSearchPathForDirectoriesInDomains(
        NSSearchPathDirectory::ApplicationSupportDirectory,
        NSSearchPathDomainMask::UserDomainMask,
        true,
    );
    let path = paths.firstObject().unwrap();
    let path_str = path.to_string();
    let bundlename = "runner";
    format!("{}/{}", path_str, bundlename)
}
fn check_for_appsupport_dir(path: &String) -> String {
    let file_path = format!("{}/lockfile.lock", path);
    if !Path::new(path.as_str()).exists() {
        let _ = create_dir(&path); // Create the application directory if it doesn't exist
    }
    file_path
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_path = app_support_path();
    let lock_path = check_for_appsupport_dir(&app_path);
    let socket_path = format!("{}/ipc.sock", &app_path);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)?;
    let mut f = RwLock::new(file);

    let _guard = match f.try_write() {
        Ok(guard) => {
            println!("First process...");
            guard
        }
        Err(_) => {
            println!("Second process...");
            let mut stream = UnixStream::connect(&socket_path)?;
            //stream.write(b"");
            std::process::exit(0);
        }
    };

    let mtm = objc2_foundation::MainThreadMarker::new().unwrap();

    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    let delegate = window::Delegate::new(mtm);
    app.setDelegate(Some(objc2::runtime::ProtocolObject::from_ref(&*delegate)));

    let main_queue = Queue::main();
    let _ = remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path)?;
    std::thread::spawn(move || {
        loop {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        main_queue
                            .exec_async({ move || println!("A wild second instance appears!") });
                    }
                    Err(err) => {
                        break;
                    }
                }
            }
        }
    });

    app.run();

    Ok(())
}
