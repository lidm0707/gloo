//! Crate that provides wrapper for
//! [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)

#![deny(missing_docs, missing_debug_implementations)]

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Bound carrying the unwind-safety requirement on [`request_animation_frame`]
/// callbacks.
///
/// Under `panic = "unwind"` on wasm the callback is invoked across a
/// `catch_unwind` boundary inside `wasm_bindgen`, so this resolves to
/// [`std::panic::UnwindSafe`]. Under any other panic strategy it is a no-op
/// blanket. Wrap non-`UnwindSafe` captures in [`std::panic::AssertUnwindSafe`]
/// at the call site.
#[cfg(all(target_arch = "wasm32", panic = "unwind"))]
pub trait CallbackUnwindSafe: std::panic::UnwindSafe {}
#[cfg(all(target_arch = "wasm32", panic = "unwind"))]
impl<T: std::panic::UnwindSafe> CallbackUnwindSafe for T {}

#[doc(hidden)]
#[cfg(not(all(target_arch = "wasm32", panic = "unwind")))]
pub trait CallbackUnwindSafe {}
#[cfg(not(all(target_arch = "wasm32", panic = "unwind")))]
impl<T> CallbackUnwindSafe for T {}

/// Handle for [`request_animation_frame`].
#[derive(Debug)]
pub struct AnimationFrame {
    render_id: i32,
    _closure: Closure<dyn Fn(JsValue)>,
    callback_wrapper: Rc<RefCell<Option<CallbackWrapper>>>,
}

struct CallbackWrapper(Box<dyn FnOnce(f64) + 'static>);
impl fmt::Debug for CallbackWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallbackWrapper").finish_non_exhaustive()
    }
}

impl Drop for AnimationFrame {
    fn drop(&mut self) {
        if self.callback_wrapper.borrow_mut().is_some() {
            web_sys::window()
                .unwrap_throw()
                .cancel_animation_frame(self.render_id)
                .unwrap_throw()
        }
    }
}

/// Calls browser's `requestAnimationFrame`. It is cancelled when the handler is dropped.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)
pub fn request_animation_frame<F>(callback_once: F) -> AnimationFrame
where
    F: FnOnce(f64) + 'static + CallbackUnwindSafe,
{
    let callback_wrapper = Rc::new(RefCell::new(Some(CallbackWrapper(Box::new(callback_once)))));
    // The internal trampoline captures `Rc<RefCell<Option<CallbackWrapper>>>`
    // which is not `UnwindSafe`. Asserting unwind safety here is sound: the
    // `borrow_mut().take()` releases the borrow before invoking the user
    // closure, and a panic inside the user closure leaves the cell holding
    // `None` — a valid post-fire state. `Drop` checks `is_some()` and skips
    // cancellation when the slot is already empty. The `CallbackUnwindSafe`
    // bound on the public API enforces unwind safety at the call site.
    let callback: Closure<dyn Fn(JsValue)> = {
        let callback_wrapper = Rc::clone(&callback_wrapper);
        let inner = Box::new(move |v: JsValue| {
            let time: f64 = v.as_f64().unwrap_or(0.0);
            let callback = callback_wrapper.borrow_mut().take().unwrap().0;
            callback(time);
        }) as Box<dyn Fn(JsValue)>;
        #[cfg(all(target_arch = "wasm32", panic = "unwind"))]
        {
            Closure::wrap_assert_unwind_safe(inner)
        }
        #[cfg(not(all(target_arch = "wasm32", panic = "unwind")))]
        {
            Closure::wrap(inner)
        }
    };

    let render_id = web_sys::window()
        .unwrap_throw()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .unwrap_throw();

    AnimationFrame {
        render_id,
        _closure: callback,
        callback_wrapper,
    }
}
