use ggez::{
    graphics::{Color, DrawMode, Mesh},
    Context, GameResult,
};

/// Handles to all our loaded assets, loaded up onces are re-used
pub struct Assets {
    pub arena: Mesh,
    pub boi: Mesh,
    pub vision: Mesh,
}

impl Assets {
    pub fn load(ctx: &mut Context, mesh_raster_scale: f32, arena_radius: f32) -> GameResult<Self> {
        // Debug markers
        let arena = Mesh::new_circle(
            ctx,
            DrawMode::stroke(1. * mesh_raster_scale),
            [0., 0.],
            arena_radius * mesh_raster_scale,
            2.,
            Color::BLACK,
        )?;
        let boi = Mesh::new_polygon(
            ctx,
            DrawMode::fill(),
            &[
                [1. * mesh_raster_scale, 0. * mesh_raster_scale], // Tip of the triangle (points forward)
                [-1. * mesh_raster_scale, 3. / 5. * mesh_raster_scale], // Left base of the triangle
                [-1. * mesh_raster_scale, -3. / 5. * mesh_raster_scale], // Right base of the triangle
            ],
            Color::RED,
        )?;

        let vision = Mesh::new_circle(
            ctx,
            DrawMode::stroke(0.1 * mesh_raster_scale),
            [0., 0.],
            1. * mesh_raster_scale,
            2.,
            Color::new(0., 0., 0., 0.2),
        )?;

        Ok(Assets { arena, boi, vision })
    }
}

/// A single structure to hold all the info about rendering
pub struct RenderState {
    pub assets: Assets,
    // Meshes are drawn at a higher resolution first so they
    // don't look blocky
    pub mesh_raster_scale: f32,
    pub screen_scale: f32, // difference between world scale and draw scale
    pub padding: f32,      // padding around edge of world in pixels
    pub fps: u32,
    pub needs_render: bool,
}

impl RenderState {
    /// Base rendering scale used for Meshes
    pub fn base_scale(&self) -> f32 {
        self.screen_scale / self.mesh_raster_scale
    }
}
