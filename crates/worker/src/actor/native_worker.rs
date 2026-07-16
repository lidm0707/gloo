use crate::codec::Codec;
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
pub(crate) use web_sys::Worker as DedicatedWorker;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

pub(crate) trait WorkerSelf {
    type GlobalScope;

    fn worker_self() -> Self::GlobalScope;
}

impl WorkerSelf for DedicatedWorker {
    type GlobalScope = DedicatedWorkerGlobalScope;

    fn worker_self() -> Self::GlobalScope {
        JsValue::from(js_sys::global()).into()
    }
}

pub(crate) trait NativeWorkerExt {
    fn set_on_packed_message<T, CODEC, F>(&self, handler: F)
    where
        T: Serialize + for<'de> Deserialize<'de>,
        CODEC: Codec,
        F: 'static + Fn(T);

    fn post_packed_message<T, CODEC>(&self, data: T)
    where
        T: Serialize + for<'de> Deserialize<'de>,
        CODEC: Codec;
}

macro_rules! worker_ext_impl {
    ($($type:path),+) => {$(
        impl NativeWorkerExt for $type {
            fn set_on_packed_message<T, CODEC, F>(&self, handler: F)
            where
                T: Serialize + for<'de> Deserialize<'de>,
                CODEC: Codec,
                F: 'static + Fn(T)
            {
                let handler = move |message: MessageEvent| {
                    let msg = CODEC::decode(message.data());
                    handler(msg);
                };
                // wasm-bindgen 0.2.117+ requires `MaybeUnwindSafe` on closure
                // inputs under `panic = "unwind"`; the `Box<F> as Box<dyn Fn>`
                // coercion erases that bound. The internal callers (spawner,
                // registrar) supply handlers that touch `Rc<RefCell<...>>` of
                // worker state via `borrow_mut().take()` patterns that release
                // borrows before invoking nested user code, so a panic across
                // the `catch_unwind` boundary leaves no observable invariant
                // violation. Asserting unwind safety here is sound. On other
                // panic strategies this is plain `Closure::wrap`.
                let inner = Box::new(handler) as Box<dyn Fn(MessageEvent)>;
                #[cfg(all(target_arch = "wasm32", panic = "unwind"))]
                let closure = Closure::wrap_assert_unwind_safe(inner).into_js_value();
                #[cfg(not(all(target_arch = "wasm32", panic = "unwind")))]
                let closure = Closure::wrap(inner).into_js_value();
                self.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            }

            fn post_packed_message<T, CODEC>(&self, data: T)
            where
                T: Serialize + for<'de> Deserialize<'de>,
                CODEC: Codec
            {
                self.post_message(&CODEC::encode(data))
                    .expect_throw("failed to post message");
            }
        }
    )+};
}

worker_ext_impl! {
    DedicatedWorker, DedicatedWorkerGlobalScope
}
