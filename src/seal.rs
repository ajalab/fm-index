// A way to have trait methods that are effectively private.
//
// They can only be used from this crate, but not outside, as long
// as we don't publish anything in this module to the outside world.
//
// https://jack.wrenn.fyi/blog/private-trait-methods/

// An effectively private type to encode locality. We make it uninhabited
// so we *cannot* accidentally leak it.
pub enum Local {}

// We pair it with a 'sealed' trait that is *only* implemented for `Local`.
pub trait IsLocal {}

impl IsLocal for Local {}
