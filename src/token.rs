use crate::{gc, private};

pub trait GcToken: private::Sealed {}

pub struct ThreadToken {
    _non_send: std::marker::PhantomData<*const ()>,
}

impl private::Sealed for ThreadToken {}
impl GcToken for ThreadToken {}

impl ThreadToken {
    pub(crate) fn new() -> Self {
        Self {
            _non_send: std::marker::PhantomData,
        }
    }

    pub fn get() -> Self {
        Self::try_get()
            .expect("Thread is not registered with the GC. Please call init_thread() first.")
    }

    pub fn try_get() -> Option<Self> {
        if crate::GC_REGISTERED.get() {
            Some(ThreadToken {
                _non_send: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct ThreadGuard {
    _non_send: std::marker::PhantomData<*const ()>,
}

impl private::Sealed for ThreadGuard {}
impl GcToken for ThreadGuard {}

impl ThreadGuard {
    pub(crate) fn new() -> Self {
        Self {
            _non_send: std::marker::PhantomData,
        }
    }
}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        unsafe { gc::GC_unregister_my_thread() };
    }
}
