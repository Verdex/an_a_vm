
pub mod common;

use an_a_vm::*;
use an_a_vm::data::*;


// TODO 
// should yield
// should resume
// should handle params
// call
// dyn call
// multiple coroutines interleaved inside of a coroutine
// finish set branch kills off finished coroutine
// finish set branch doesnt kill off active coroutine
// resuming coroutine pulls coroutine out of order
// yielding or finishing coroutine puts it in the end of the coroutine list