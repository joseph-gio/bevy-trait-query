use core::sync::atomic::{AtomicBool, Ordering};

use crate::TraitQuery;
use crate::dyn_constructor::DynCtor;
use bevy_ecs::component::{Component, ComponentId, StorageType};
use bevy_ecs::prelude::Resource;
#[derive(Resource)]
pub(crate) struct TraitImplRegistry<Trait: ?Sized> {
    // Component IDs are stored contiguously so that we can search them quickly.
    pub(crate) components: Vec<ComponentId>,
    pub(crate) meta: Vec<TraitImplMeta<Trait>>,

    pub(crate) table_components: Vec<ComponentId>,
    pub(crate) table_meta: Vec<TraitImplMeta<Trait>>,

    pub(crate) sparse_components: Vec<ComponentId>,
    pub(crate) sparse_meta: Vec<TraitImplMeta<Trait>>,

    pub(crate) sealed: AtomicBool,
}

impl<T: ?Sized> Default for TraitImplRegistry<T> {
    #[inline]
    fn default() -> Self {
        Self {
            components: vec![],
            meta: vec![],
            table_components: vec![],
            table_meta: vec![],
            sparse_components: vec![],
            sparse_meta: vec![],
            sealed: AtomicBool::new(false),
        }
    }
}

impl<Trait: ?Sized + TraitQuery> TraitImplRegistry<Trait> {
    pub(crate) fn register<C: Component>(
        &mut self,
        component: ComponentId,
        meta: TraitImplMeta<Trait>,
    ) {
        // Don't register the same component multiple times.
        if self.components.contains(&component) {
            return;
        }

        if self.sealed.load(Ordering::Relaxed) {
            // It is not possible to update the `FetchState` for a given system after the game has started,
            // so for explicitness, let's panic instead of having a trait impl silently get forgotten.
            panic!("Cannot register new trait impls after the game has started");
        }

        self.components.push(component);
        self.meta.push(meta);

        match <C as Component>::STORAGE_TYPE {
            StorageType::Table => {
                self.table_components.push(component);
                self.table_meta.push(meta);
            }
            StorageType::SparseSet => {
                self.sparse_components.push(component);
                self.sparse_meta.push(meta);
            }
        }
    }

    pub(crate) fn seal(&self) {
        self.sealed.store(true, Ordering::Relaxed);
    }
}

/// Stores data about an impl of a trait
pub(crate) struct TraitImplMeta<Trait: ?Sized> {
    pub(crate) size_bytes: usize,
    pub(crate) dyn_ctor: DynCtor<Trait>,
}

impl<T: ?Sized> Copy for TraitImplMeta<T> {}
impl<T: ?Sized> Clone for TraitImplMeta<T> {
    fn clone(&self) -> Self {
        *self
    }
}
