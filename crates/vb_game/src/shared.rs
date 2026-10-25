use bevy::prelude::*;

/// Player wallet. Persists across a run (prototype: one run).
#[derive(Resource, Default)]
pub struct Inventory {
    pub ore: u32,
}

/// How many destructible blocks are still standing.
#[derive(Resource, Default)]
pub struct BricksLeft(pub i32);

/// Shared material handles for mine blocks, indexed by remaining HP tier.
#[derive(Resource)]
pub struct BrickMaterials {
    /// `tiers[0]` = 1 HP (cracked), `tiers[2]` = 3 HP (fresh).
    pub tiers: [Handle<StandardMaterial>; 3],
    pub bedrock: Handle<StandardMaterial>,
}

/// Handles reused by every ore pickup (avoids per-pickup allocations).
#[derive(Resource)]
pub struct FxAssets {
    pub ore_mesh: Handle<Mesh>,
    pub ore_material: Handle<StandardMaterial>,
}
