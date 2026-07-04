//! Provides functionality for registering clocks to be used by the animation system.
//! 
//! All Bevy built-in clocks are already registered by default, custom
//! clocks need to be registered in the app.
//! 
//! ```no_run
//! app.add_tween_clock::<CustomClock>()
//! ```
//! 
//! Then, tween animations can use `CustomClock` as their clock type.
//! 
//! ```no_run
//! TweenAnim::new_timed::<CustomClock>(tweenable)
//! ```
//! 

use bevy::{platform::collections::HashMap, prelude::*};

use std::any::TypeId;
use std::time::Duration;


/// A map that stores the delta time for each registered clock indexed by their `TypeId`.
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct DeltaTime(pub HashMap<TypeId, Duration>);

impl From<HashMap<TypeId, Duration>> for DeltaTime {
    fn from(map: HashMap<TypeId, Duration>) -> Self {
        Self(map)
    }
}

impl From<Duration> for DeltaTime {
    fn from(duration: Duration) -> Self {
        let mut map = HashMap::new();
        map.insert(TypeId::of::<()>(), duration);
        Self(map)
    }
}


/// Resource that stores type-erased functions to retrieve delta time for different clocks.
#[derive(Resource, Clone)]
pub struct TweenTime {
    functions: HashMap<TypeId, fn(&mut World) -> Duration>,
}

impl Default for TweenTime {
    fn default() -> Self {
        let mut time = Self {
            functions: HashMap::new(),
        };
        time.register::<()>();
        time
    }
}

impl TweenTime {
    /// Retrieve the delta time for all registered clocks.
    pub fn get_all(world: &mut World) -> DeltaTime {
        let tween_time = world.get_resource::<TweenTime>().cloned().unwrap();

        let mut deltas = HashMap::new();

        for (type_id, func) in tween_time.functions {
            deltas.insert(type_id, func(world));
        }

        deltas.into()
    }

    /// Register a new clock to be used by animations.
    pub fn register<T: Send + Sync + 'static + Default>(&mut self) {
        self.functions
            .insert(TypeId::of::<T>(), |world: &mut World| {
                world
                    .get_resource::<Time<T>>()
                    .map(|t| t.delta())
                    .unwrap_or(Duration::ZERO)
            });
    }
}

/// Extension trait to easily register new clocks in the `App`.
pub trait TweenTimePluginExt {
    /// Adds a new tween clock
    fn add_tween_clock<T: Send + Sync + 'static + Default>(&mut self) -> &mut Self;
}

impl TweenTimePluginExt for App {
    fn add_tween_clock<T: Send + Sync + 'static + Default>(&mut self) -> &mut Self {
        self.world_mut().resource_mut::<TweenTime>().register::<T>();
        self
    }
}
