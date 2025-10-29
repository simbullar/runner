#![deny(unsafe_op_in_unsafe_fn)]
use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSAutoresizingMaskOptions,
    NSBackingStoreType, NSColor, NSFloatingWindowLevel, NSFont, NSTextAlignment, NSTextField,
    NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
    ns_string,
};

#[derive(Debug, Default)]
struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
}

/* this was the example code from https://docs.rs/objc2/latest/objc2/
 * i kinda used it as a starting point :D
*/

const WIDTH_DEFAULT: u16 = 600;
const HEIGHT_DEFAULT: u16 = 350;

define_class!(
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `Delegate` does not implement `Drop`.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    struct Delegate;

    // SAFETY: `NSObjectProtocol` has no safety requirements.
    unsafe impl NSObjectProtocol for Delegate {}

    // SAFETY: `NSApplicationDelegate` has no safety requirements.
    unsafe impl NSApplicationDelegate for Delegate {
        // SAFETY: The signature is correct.
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, notification: &NSNotification) {
            let mtm = self.mtm();

            let app = notification
                .object()
                .unwrap()
                .downcast::<NSApplication>()
                .unwrap();

            /* Text field */

            let text_field = NSTextField::labelWithString(ns_string!("Hello, World!"), mtm);
            text_field.setFrame(NSRect::new(
                NSPoint::new(5.0, 100.0),
                NSSize::new(290.0, 100.0),
            ));
            text_field.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
                0.0, 0.5, 0.0, 1.0,
            )));
            text_field.setAlignment(NSTextAlignment::Center);
            text_field.setFont(Some(&NSFont::systemFontOfSize(45.0)));
            text_field.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );

            // SAFETY: We disable releasing when closed below.
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    NSRect::new(
                        NSPoint::new(0.0, 0.0),
                        NSSize::new(WIDTH_DEFAULT as f64, HEIGHT_DEFAULT as f64),
                    ),
                    NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            // SAFETY: Disable auto-release when closing windows.
            // This is required when creating `NSWindow` outside a window
            // controller.
            unsafe { window.setReleasedWhenClosed(false) };

            // Set various window properties.
            let view = window.contentView().expect("window must have content view");
            view.addSubview(&text_field);
            window.center();
            window.setDelegate(Some(ProtocolObject::from_ref(self)));

            unsafe {
                let _: () = msg_send![&*window, setLevel: NSFloatingWindowLevel];
            }

            // Show the window.
            window.orderFront(None);

            // Store the window in the delegate.
            self.ivars().window.set(window).unwrap();

            app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

            // Activate the application.
            // Required when launching unbundled (as is done with Cargo).
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        }
    }

    // SAFETY: `NSWindowDelegate` has no safety requirements.
    unsafe impl NSWindowDelegate for Delegate {
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            // Quit the application when the window is closed.
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }
    }
);

impl Delegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        // SAFETY: The signature of `NSObject`'s `init` method is correct.
        unsafe { msg_send![super(this), init] }
    }
}

pub fn start_window() {
    let mtm = MainThreadMarker::new().unwrap();

    let app = NSApplication::sharedApplication(mtm);
    let delegate = Delegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    app.run();
}
