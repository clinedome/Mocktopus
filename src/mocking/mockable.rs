use crate::mocking::mock::*;
use crate::mocking::MockResult;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::transmute;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local!{
    static MOCK_STORE: RefCell<HashMap<TypeId, Rc<RefCell<Box<Mock<(), Output = ()>>>>>> = RefCell::new(HashMap::new());
}

// trait IntoMockContainer<Output = MockContainer>
//
// struct MockConainer<I, O> {
//     mock: Box<Mock<(), Output = ()>>,
//     _variance_guard: PhantomData<Fn(
// }
//
// trait Mock<I> {
//     type Output;
//
//     fn call_mock
// }

/// Trait for setting up mocks
///
/// The trait is implemented for all functions, so its methods can be called on any function.
///
/// Note: methods have any effect only if called on functions [annotated as mockable](https://docs.rs/mocktopus_macros).
pub trait Mockable<I> {
    type Output;

    unsafe fn mock_raw(&self, mock: impl Mock<I, Output = Self::Output>) {
        // register_mock(self, mock);
        let mock_id = get_mock_id(self);
        let mock_boxed = Box::new(mock) as Box<Mock<I, Output = Self::Output>>;
        let mock_shared = Rc::new(RefCell::new(mock_boxed));
        let mock_stored = transmute(mock_shared);
        MOCK_STORE.with(|mock_ref_cell| mock_ref_cell.borrow_mut().insert(mock_id, mock_stored));
    }

    fn mock_safe(&self, mock: impl Mock<I, Output = Self::Output> + 'static) {
        unsafe {
            self.mock_raw(mock)
        }
    }

    fn mock_extra_safe(&self, mut mock: MockContainer<fn(Self::Output) -> I>
            //  + SafeMock<I, Self::Output> + 'static
        ){
        unsafe {
            let mock_id = get_mock_id(self);
            MOCK_STORE.with(|mock_ref_cell| mock_ref_cell.borrow_mut().insert(mock_id, Rc::new(RefCell::new(mock.mock))));
        }
    }

    fn fake_call(&self, input: I, mock: &mut Mock<I, Output = Self::Output>) -> Self::Output {
        mock.fake_call(input)
    }
}

#[doc(hidden)]
pub fn call_mock<I, O, M: Mockable<I, Output = O>>(mockable: &M, input: I) -> MockResult<I, O> {
    unsafe {
        let id = get_mock_id(mockable);
        let rc_opt = MOCK_STORE.with(|mock_ref_cell|
            mock_ref_cell.borrow()
                .get(&id)
                .cloned()
        );
        let stored_opt = rc_opt.as_ref()
            .and_then(|rc| rc.try_borrow_mut().ok());
        match stored_opt {
            Some(mut stored) => {
                let real: &mut Box<Mock<I, Output = O>> = transmute(&mut*stored);
                real.call_mock(input)
            }
            None => MockResult::Continue(input),
        }
    }
}

fn get_mock_id<T>(_: T) -> TypeId {
    (||()).type_id()
}

impl<O, F: Fn() -> O> Mockable<()> for F {
    type Output = O;
}

impl<I1, O, F: Fn(I1) -> O> Mockable<(I1,)> for F {
    type Output = O;
}

impl<I1, I2, O, F: Fn(I1, I2) -> O> Mockable<(I1, I2)> for F {
    type Output = O;
}

impl<I1, I2, I3, O, F: Fn(I1, I2, I3) -> O> Mockable<(I1, I2, I3)> for F {
    type Output = O;
}


