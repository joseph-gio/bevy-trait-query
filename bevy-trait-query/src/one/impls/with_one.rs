use core::marker::PhantomData;

use crate::{TraitQuery, TraitQueryState};
use bevy_ecs::query::FilteredAccessSet;
use bevy_ecs::{
    change_detection::Tick,
    component::{ComponentId, Components},
    prelude::{Entity, World},
    query::{ArchetypeFilter, QueryFilter, WorldQuery},
    storage::TableRow,
    world::unsafe_world_cell::UnsafeWorldCell,
};

/// [`WorldQuery`] filter for entities with exactly [one](crate::One) component
/// implementing a trait.
pub struct WithOne<Trait: ?Sized + TraitQuery>(PhantomData<&'static Trait>);

// this takes inspiration from `With` in bevy's main repo
unsafe impl<Trait: ?Sized + TraitQuery> WorldQuery for WithOne<Trait> {
    type Fetch<'w> = ();
    type State = TraitQueryState<Trait>;

    #[inline]
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(_fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
    }

    #[inline]
    unsafe fn init_fetch(
        _world: UnsafeWorldCell<'_>,
        _state: &Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) {
    }

    const IS_DENSE: bool = false;

    #[inline]
    unsafe fn set_archetype<'w>(
        _fetch: &mut (),
        _state: &Self::State,
        _archetype: &'w bevy_ecs::archetype::Archetype,
        _table: &'w bevy_ecs::storage::Table,
    ) {
    }

    #[inline]
    unsafe fn set_table(_fetch: &mut (), _state: &Self::State, _table: &bevy_ecs::storage::Table) {}

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut bevy_ecs::query::FilteredAccess) {
        let mut new_access = access.clone();
        for &component in state.components.iter() {
            let mut intermediate = access.clone();
            intermediate.and_with(component);
            new_access.append_or(&intermediate);
        }
        *access = new_access;
    }

    fn init_nested_access(
        state: &Self::State,
        system_name: Option<&str>,
        component_access_set: &mut FilteredAccessSet,
        world: UnsafeWorldCell,
    ) {
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

    #[inline]
    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        state.matches_component_set_one(set_contains_id)
    }

    fn update_archetypes(state: &mut Self::State, world: UnsafeWorldCell) {}
}

impl<Trait: ?Sized + TraitQuery> ArchetypeFilter for WithOne<Trait> {}

/// SAFETY: read-only access
unsafe impl<Trait: ?Sized + TraitQuery> QueryFilter for WithOne<Trait> {
    const IS_ARCHETYPAL: bool = false;
    unsafe fn filter_fetch(
        _state: &Self::State,
        _fetch: &mut Self::Fetch<'_>,
        _entity: Entity,
        _table_row: TableRow,
    ) -> bool {
        true
    }
}
