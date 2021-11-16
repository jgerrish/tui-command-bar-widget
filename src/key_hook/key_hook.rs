///
/// KeyHook is a trait for registering key commands for views and other UI
/// elements.
///
use std::collections::HashMap;

/// The KeyDatabase stores command keys and the functions they invoke
#[derive(Clone)]
pub struct KeyDatabase<'a, T> {
    /// keys is the actual key database, implemented as a HashMap
    /// mappings characters to functions that accept a generic object
    /// and a character
    pub keys: HashMap<char, &'a dyn Fn(&mut T, char) -> ()>,
}

impl<'a, T> Default for KeyDatabase<'a, T> {
    fn default() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }
}

/// Register keys to listen for
/// Each View can register to listen for certain key presses in the main
/// app event loop
///
/// Cursive handles events by starting at the root object and then descending
/// the tree to the view currently in focus.
/// Views can choose to consume events or ignore them.
/// If no view consumes the event, the global callback table is checked
///
/// Info from the doucmentation for cursive::event
///
/// The command view may not be focused or even visible, so handling is done
/// on the global hook.
pub trait KeyHook<'a, T> {
    /// Register a key listener
    fn register_key(&mut self, key: char, f: &'a dyn Fn(&mut T, char) -> ());

    /// Unregister a key listener
    fn unregister_key(&mut self, key: char);
}
