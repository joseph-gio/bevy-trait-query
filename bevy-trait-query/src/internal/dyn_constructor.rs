use bevy_ecs::ptr::{Ptr, PtrMut};

/// Turns an untyped pointer into a trait object pointer,
/// for a specific erased concrete type.
pub(crate) struct DynCtor<Trait: ?Sized> {
    pub(crate) cast: unsafe fn(*mut u8) -> *mut Trait,
}

impl<T: ?Sized> Copy for DynCtor<T> {}
impl<T: ?Sized> Clone for DynCtor<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Trait: ?Sized> DynCtor<Trait> {
    #[inline]
    pub(crate) unsafe fn cast(self, ptr: Ptr<'_>) -> &'_ Trait {
        unsafe { &*(self.cast)(ptr.as_ptr()) }
    }
    #[inline]
    pub(crate) unsafe fn cast_mut(self, ptr: PtrMut<'_>) -> &'_ mut Trait {
        unsafe { &mut *(self.cast)(ptr.as_ptr()) }
    }
}
