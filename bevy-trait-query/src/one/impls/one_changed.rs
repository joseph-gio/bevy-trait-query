use bevy_ecs::ptr::UnsafeCellDeref;
use std::marker::PhantomData;

use bevy_ecs::{
    archetype::Archetype,
    component::{ComponentId, Components, Tick},
    prelude::{Entity, World},
    query::{FilteredAccess, QueryData, QueryFilter, ReadOnlyQueryData, WorldQuery},
    storage::{Table, TableRow},
    world::unsafe_world_cell::UnsafeWorldCell,
};

use crate::{TraitQuery, TraitQueryState, debug_unreachable};

use crate::{ChangeDetectionFetch, ChangeDetectionStorage};

/// [`WorldQuery`] filter for entities with exactly [one](crate::One) component
/// implementing a trait, which was added since the last time the system ran.
pub struct OneChanged<Trait: ?Sized + TraitQuery> {
    marker: PhantomData<&'static Trait>,
}

unsafe impl<Trait: ?Sized + TraitQuery> QueryData for OneChanged<Trait> {
    type ReadOnly = Self;

    /// SAFETY: read-only access
    const IS_READ_ONLY: bool = true;

    type Item<'w> = bool;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        item
    }

    #[inline(always)]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Self::Item<'w> {
        unsafe {
            let ticks_ptr = match fetch.storage {
                ChangeDetectionStorage::Uninit => {
                    // set_archetype must have been called already
                    debug_unreachable()
                }
                ChangeDetectionStorage::Table { ticks } => ticks.get(table_row.as_usize()),
                ChangeDetectionStorage::SparseSet { components } => components
                    .get_changed_tick(entity)
                    .unwrap_or_else(|| debug_unreachable()),
            };

            (*ticks_ptr)
                .deref()
                .is_newer_than(fetch.last_run, fetch.this_run)
        }
    }
}

unsafe impl<Trait: ?Sized + TraitQuery> WorldQuery for OneChanged<Trait> {
    type Fetch<'w> = ChangeDetectionFetch<'w>;
    type State = TraitQueryState<Trait>;

    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        _state: &Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        unsafe {
            Self::Fetch::<'w> {
                storage: ChangeDetectionStorage::Uninit,
                sparse_sets: &world.storages().sparse_sets,
                last_run,
                this_run,
            }
        }
    }

    // This will always be false for us, as we (so far) do not know at compile time whether the
    // components our trait has been impl'd for are stored in table or in sparse set
    const IS_DENSE: bool = false;

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        _archetype: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe {
            // Search for a registered trait impl that is present in the archetype.
            // We check the table components first since it is faster to retrieve data of this type.
            for &component in &*state.components {
                if let Some(changed) = table.get_changed_ticks_slice_for(component) {
                    fetch.storage = ChangeDetectionStorage::Table {
                        ticks: changed.into(),
                    };
                    return;
                }
            }
            for &component in &*state.components {
                if let Some(components) = fetch.sparse_sets.get(component) {
                    fetch.storage = ChangeDetectionStorage::SparseSet { components };
                    return;
                }
            }
            // At least one of the components must be present in the table/sparse set.
            debug_unreachable()
        }
    }

    #[inline]
    unsafe fn set_table<'w>(_fetch: &mut Self::Fetch<'w>, _state: &Self::State, _table: &'w Table) {
        unsafe {
            // only gets called if IS_DENSE == true, which does not hold for us
            debug_unreachable()
        }
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        let mut new_access = access.clone();
        let mut not_first = false;
        for &component in &*state.components {
            assert!(
                !access.access().has_component_write(component),
                "&{} conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<Trait>(),
            );
            if not_first {
                let mut intermediate = access.clone();
                intermediate.add_component_read(component);
                new_access.append_or(&intermediate);
                new_access.extend_access(&intermediate);
            } else {
                new_access.and_with(component);
                new_access.access_mut().add_component_read(component);
                not_first = true;
            }
        }
        *access = new_access;
    }

    #[inline]
    fn init_state(world: &mut World) -> Self::State {
        TraitQueryState::init(world)
    }

    #[inline]
    fn get_state(_: &Components) -> Option<Self::State> {
        // TODO: fix this https://github.com/bevyengine/bevy/issues/13798
        panic!(
            "transmuting and any other operations concerning the state of a query are currently broken and shouldn't be used. See https://github.com/JoJoJet/bevy-trait-query/issues/59"
        );
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        state.matches_component_set_one(set_contains_id)
    }

    #[inline]
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        fetch
    }
}

unsafe impl<Trait: ?Sized + TraitQuery> ReadOnlyQueryData for OneChanged<Trait> {}
unsafe impl<Trait: ?Sized + TraitQuery> QueryFilter for OneChanged<Trait> {
    const IS_ARCHETYPAL: bool = false;
    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        table_row: TableRow,
    ) -> bool {
        unsafe { <Self as QueryData>::fetch(fetch, entity, table_row) }
    }
}
