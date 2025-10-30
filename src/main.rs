mod search;
mod window;

fn main() {
    let mtm = objc2_foundation::MainThreadMarker::new().unwrap();

    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    let delegate = window::Delegate::new(mtm);
    app.setDelegate(Some(objc2::runtime::ProtocolObject::from_ref(&*delegate)));

    app.run();
}
