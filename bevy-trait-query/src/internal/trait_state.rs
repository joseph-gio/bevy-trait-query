use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::World;

use crate::{
    TraitQuery,
    trait_registry::{TraitImplMeta, TraitImplRegistry},
};

#[doc(hidden)]
pub struct TraitQueryState<Trait: ?Sized> {
    pub(crate) components: Box<[ComponentId]>,
    pub(crate) meta: Box<[TraitImplMeta<Trait>]>,
}

impl<Trait: ?Sized + TraitQuery> TraitQueryState<Trait> {
    pub(crate) fn init(world: &World) -> Self {
        match world.get_resource::<TraitImplRegistry<Trait>>() {
            Some(registry) => {
                registry.seal();
                Self {
                    components: registry.components.clone().into_boxed_slice(),
                    meta: registry.meta.clone().into_boxed_slice(),
                }
            }
            None => {
                tracing::warn!(
                    "no components found matching `{}`, did you forget to register them?",
                    std::any::type_name::<Trait>()
                );
                Self {
                    components: Box::new([]),
                    meta: Box::new([]),
                }
            }
        }
    }

    #[inline]
    pub(crate) fn matches_component_set_any(
        &self,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        self.components.iter().copied().any(set_contains_id)
    }

    #[inline]
    pub(crate) fn matches_component_set_one(
        &self,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        let match_count = self
            .components
            .iter()
            .filter(|&&c| set_contains_id(c))
            .count();
        match_count == 1
    }
}
