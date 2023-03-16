///
/// key_hook is a module for managing command keys
/// You can register keys and run actions when keys are matched
/// Right now this is a simple database of keys and hooks,
/// eventually it may evolve into an event system.
///
#[allow(clippy::module_inception)]
pub mod key_hook;
