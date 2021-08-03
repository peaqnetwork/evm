//! Allows to listen to runtime events.

use crate::Context;
use evm_runtime::{CreateScheme, Transfer};
use primitive_types::{H160, U256};
use once_cell::unsync::OnceCell;

environmental::environmental!(listener: dyn EventListener + 'static);
struct LocalKey {
    pub inner: OnceCell<bool>,
}
// Just copy environmental in
// https://github.com/paritytech/environmental/blob/master/src/local_key.rs#L26
// This is safe as long there is no threads in wasm32.
unsafe impl ::core::marker::Sync for LocalKey {}
static IS_ACTIVE: LocalKey = LocalKey {
    inner: OnceCell::new(),
};

pub trait EventListener {
	fn event(&mut self, event: Event);
}

#[derive(Debug, Copy, Clone)]
pub enum Event<'a> {
	Call {
		code_address: H160,
		transfer: &'a Option<Transfer>,
		input: &'a [u8],
		target_gas: Option<u64>,
		is_static: bool,
		context: &'a Context,
	},
	Create {
		caller: H160,
		address: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: &'a [u8],
		target_gas: Option<u64>,
	},
	Suicide {
		address: H160,
		target: H160,
		balance: U256,
	},
}

impl<'a> Event<'a> {
	pub(crate) fn emit(self) {
		listener::with(|listener| listener.event(self));
	}
}

/// Cache and return whether or not we are in an environmental `using` call.
#[inline]
pub fn is_environmental() -> bool {
    if let Some(active) = IS_ACTIVE.inner.get() {
        *active
    } else if let Some(_) = listener::with(|_listener| ())  {
        // `with` returns whether there is `Some` environmental context for the Event or `None`.
        IS_ACTIVE.inner.set(true).unwrap();
        true
    } else {
        IS_ACTIVE.inner.set(false).unwrap();
        false
    }
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
	listener::using(new, f)
}
