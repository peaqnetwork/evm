//! Allows to listen to runtime events.

use crate::{Capture, Context, ExitReason, Memory, Opcode, Stack, Trap};
use once_cell::unsync::OnceCell;
use primitive_types::{H160, H256};

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
	Step {
		context: &'a Context,
		opcode: Opcode,
		position: &'a Result<usize, ExitReason>,
		stack: &'a Stack,
		memory: &'a Memory,
	},
	StepResult {
		result: &'a Result<(), Capture<ExitReason, Trap>>,
		return_value: &'a [u8],
	},
	SLoad {
		address: H160,
		index: H256,
		value: H256,
	},
	SStore {
		address: H160,
		index: H256,
		value: H256,
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
	} else if let Some(_) = listener::with(|_listener| ()) {
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
