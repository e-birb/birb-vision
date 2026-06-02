use std::panic::{catch_unwind, UnwindSafe};

// TODO maybe this is not possible to prove: #[no_panic::no_panic]
pub fn try_no_panic(f: impl FnOnce() + UnwindSafe) {
    let r = catch_unwind(f);

    if let Err(e) = r {
        let error: &str = if let Some(e) = e.downcast_ref::<String>() {
            e
        } else if let Some(e) = e.downcast_ref::<&str> () {
            e
        } else {
            // This should never happen as panics can only be strings
            "unknown error"
        };

        if let Err(_) = catch_unwind(move || {
            #[cfg(feature = "log")]
            log::error!("MVS callback panicked, callbacks shall never panic as exception cannot propagate in this context: {}", error);
            #[cfg(not(feature = "log"))]
            let _ = error;
            //drop(e);
        }) {
            eprintln!("MVS callback panicked, callbacks shall never panic as exception cannot propagate in this context. -- Also failed to log or dropping the error! APP STATE MIGHT BE INVALID!");
        }
    }
}