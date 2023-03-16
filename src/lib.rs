///
/// CommandBar widget library
/// This library has a set of TUI UI widgets and examples for using a command bar
/// in your own program.
///

/// The key_hook module contains key handling code
#[warn(missing_docs)]
#[warn(unsafe_code)]
pub mod key_hook;

/// The widgets module contains a set of UI widgets to use a CommandBar in
/// your app.
#[warn(missing_docs)]
#[warn(unsafe_code)]
pub mod widgets;

#[cfg(test)]
use mockall_double::double;

// This exists to provide a mock context for read
// For some reason, this only works in the widgets module, not the integration
// tests.
pub mod crossterm {
    #[allow(unused_imports)]
    use mockall::automock;

    // Use conditional cfg_attr, it seems to decrease compilation time
    #[cfg_attr(test, automock)]
    pub mod event {
        pub fn read() -> ::crossterm::Result<::crossterm::event::Event> {
            ::crossterm::event::read()
        }

        pub use crossterm::event::Event;
        pub use crossterm::event::KeyCode;
    }
}

#[cfg(test)]
#[double]
pub use crate::crossterm::event;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
