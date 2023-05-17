use std::sync::{Once, Mutex};
use crate::{Function, KnownFunctions};
use paste::paste;

#[macro_export]
macro_rules! function_constant {
    ($const_name:ident, $value:expr, $name:expr) => {
        paste! {
            pub const [<$const_name _VALUE>]: u64 = $value;
        }
        pub const $const_name: Function = Function::new_with_static_name($value, $name);
    };
}

function_constant!(ADD, 1, "add");
function_constant!(SUB, 2, "sub");
function_constant!(MUL, 3, "mul");
function_constant!(DIV, 4, "div");

pub struct LazyFunctions {
    init: Once,
    data: Mutex<Option<KnownFunctions>>,
}

impl LazyFunctions {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<KnownFunctions>> {
        self.init.call_once(|| {
            let m = KnownFunctions::new([
                ADD,
                SUB,
                MUL,
                DIV,
            ]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

pub static FUNCTIONS: LazyFunctions = LazyFunctions {
    init: Once::new(),
    data: Mutex::new(None),
};