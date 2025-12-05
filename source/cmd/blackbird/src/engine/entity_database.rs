pub struct EntityDatabase {
    entities: Vec<Box<dyn std::any::Any>>,
}

impl EntityDatabase {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    //=========================================================================
    // Queries
    //=========================================================================

    pub fn must_select_mut<T: 'static>(&mut self) -> &mut T {
        self.select_mut::<T>().unwrap_or_else(|| {
            panic!(
                "EntityDatabase: must_select_mut failed for type {}",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn select<T: 'static>(&self) -> Option<&T> {
        for entity in &self.entities {
            if let Some(e) = entity.downcast_ref::<T>() {
                return Some(e);
            }
        }
        None
    }
    pub fn select_mut<T: 'static>(&mut self) -> Option<&mut T> {
        for entity in &mut self.entities {
            if let Some(e) = entity.downcast_mut::<T>() {
                return Some(e);
            }
        }
        None
    }

    pub fn query<T: 'static>(&self) -> Vec<&T> {
        let mut results = Vec::new();
        for entity in &self.entities {
            if let Some(e) = entity.downcast_ref::<T>() {
                results.push(e);
            }
        }
        results
    }

    //=========================================================================
    // Engine internals
    //=========================================================================

    pub fn append(&mut self, other: &mut Vec<Box<dyn std::any::Any>>) {
        self.entities.append(other);
    }

    pub fn swap_entities(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.entities, &mut other.entities);
    }
}
