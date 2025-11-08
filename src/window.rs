#![deny(unsafe_op_in_unsafe_fn)]
use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType,
    NSColor, NSFloatingWindowLevel, NSFont, NSTextAlignment, NSTextField, NSWindow,
    NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
};

#[derive(Debug, Default)]
pub struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
}

/* this was the example code from https://docs.rs/objc2/latest/objc2/
 * i kinda used it as a starting point :D
*/

/* Settings, will replace with a normal interface later on */
const WIDTH_DEFAULT: u16 = 600;
const HEIGHT_DEFAULT: u16 = 350;
const INPUT_HEIGHT: u16 = 40;
const WIN_CORNER_RAD: u8 = 4;
const SHADOW: bool = false;
const INPUT_ALIGNMENT: NSTextAlignment = NSTextAlignment::Center;
const INPUT_FONT_SIZE: f64 = 20.0;
const INPUT_FONT_COLOR: [f64; 4] = [1.0, 1.0, 1.0, 1.0];
const INPUT_BACKGROUND_COLOR: [f64; 4] = [0.2, 0.2, 0.2, 1.0];
const BACKGROUND_COLOR: [f64; 4] = [0.1, 0.1, 0.1, 1.0];
const PADDING_TOP_INPUT: f64 = 15.0;
const PADDING_SIDES_INPUT: f64 = 40.0;

/* text field */
fn text_field(mtm: MainThreadMarker) -> Retained<NSTextField> {
    let text_field = NSTextField::initWithFrame(
        NSTextField::alloc(mtm),
        NSRect::new(
            NSPoint::new(
                PADDING_SIDES_INPUT,
                HEIGHT_DEFAULT as f64 - INPUT_HEIGHT as f64 - PADDING_TOP_INPUT,
            ),
            NSSize::new(
                WIDTH_DEFAULT as f64 - 2.0 * PADDING_SIDES_INPUT,
                INPUT_HEIGHT as f64,
            ),
        ),
    );
    let [r, g, b, a] = INPUT_FONT_COLOR;

    text_field.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
        r, g, b, a,
    )));
    text_field.setAlignment(INPUT_ALIGNMENT);
    text_field.setFont(Some(&NSFont::systemFontOfSize(INPUT_FONT_SIZE)));
    text_field.setEditable(true);
    text_field.setSelectable(true);
    text_field.setMaximumNumberOfLines(0);
    text_field.acceptsFirstResponder();

    text_field.setBezeled(false);
    text_field.setBordered(false);

    text_field.setFocusRingType(objc2_app_kit::NSFocusRingType::None);

    let [r, g, b, a] = INPUT_BACKGROUND_COLOR;

    text_field.setBackgroundColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
        r, g, b, a,
    )));

    text_field.setPlaceholderString(Some(&NSString::from_str("Search...")));

    text_field
}

define_class!(
    #[unsafe(super = NSWindow)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    pub struct RunnerWindow;

    unsafe impl NSObjectProtocol for RunnerWindow {}

    impl RunnerWindow {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            true
        }
    }
);
define_class!(
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `Delegate` does not implement `Drop`.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    pub struct Delegate;

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

            /* UI */

            let text_field = text_field(mtm);

            // SAFETY: We disable releasing when closed below.
            let rect = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(WIDTH_DEFAULT as f64, HEIGHT_DEFAULT as f64),
            );
            let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
            let backing = NSBackingStoreType::Buffered;
            let defer = false;

            let window: Retained<RunnerWindow> = unsafe {
                let alloc = RunnerWindow::alloc(mtm);
                // Call the initializer on the allocated object itself — not via `super(...)`.
                msg_send![alloc, initWithContentRect: rect,
                                  styleMask: style,
                                    backing: backing,
                                      defer: defer]
            };

            window.setOpaque(false);
            window.setLevel(NSFloatingWindowLevel);
            window.setBackgroundColor(Some(&NSColor::clearColor()));
            window.setHasShadow(SHADOW);

            // SAFETY: Disable auto-release when closing windows.
            // This is required when creating `NSWindow` outside a window
            // controller.
            unsafe { window.setReleasedWhenClosed(false) };

            // Set various window properties.
            let view = window.contentView().expect("window must have content view");
            view.setWantsLayer(true);

            let [r, g, b, a] = BACKGROUND_COLOR;

            if let Some(layer) = view.layer() {
                layer.setCornerRadius(WIN_CORNER_RAD as f64);
                layer.setMasksToBounds(true);
                layer.setBackgroundColor(Some(
                    &NSColor::colorWithSRGBRed_green_blue_alpha(r, g, b, a).CGColor(),
                ));
            }

            view.addSubview(&text_field);
            window.center();
            window.setDelegate(Some(ProtocolObject::from_ref(self)));

            window.makeFirstResponder(Some(&text_field));

            // Show the window.
            window.orderFront(None);

            // Store the window in the delegate.
            self.ivars().window.set(window.into_super()).unwrap();

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
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        // SAFETY: The signature of `NSObject`'s `init` method is correct.
        unsafe { msg_send![super(this), init] }
    }
    pub fn toggle_window_visibility(&self) {
        if let Some(window) = self.ivars().window.get() {
            if window.isVisible() {
                window.orderOut(None);
            } else {
                window.makeKeyAndOrderFront(None);
            }
        }
    }
}
