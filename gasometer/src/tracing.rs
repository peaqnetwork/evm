//! Allows to listen to gasometer events.

use super::Snapshot;
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

impl Snapshot {
	pub fn gas(&self) -> u64 {
		self.gas_limit - self.used_gas - self.memory_gas
	}
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
	RecordCost {
		cost: u64,
		snapshot: Snapshot,
	},
	RecordRefund {
		refund: i64,
		snapshot: Snapshot,
	},
	RecordStipend {
		stipend: u64,
		snapshot: Snapshot,
	},
	RecordDynamicCost {
		gas_cost: u64,
		memory_gas: u64,
		gas_refund: i64,
		snapshot: Snapshot,
	},
	RecordTransaction {
		cost: u64,
		snapshot: Snapshot,
	},
}

impl Event {
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
