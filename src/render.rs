use ggez::{
    graphics::{Color, DrawMode, Image, Mesh},
    Context, GameResult,
};

/// Handles to all our loaded assets, loaded up once and re-used
pub struct Assets {
    pub arena: Mesh,
    pub boi: Image,
    pub vision: Mesh,
}

impl Assets {
    pub fn load(ctx: &mut Context, mesh_raster_scale: f32, arena_radius: f32) -> GameResult<Self> {
        let arena = Mesh::new_circle(
            ctx,
            DrawMode::stroke(1. * mesh_raster_scale),
            [0., 0.],
            arena_radius * mesh_raster_scale,
            2.,
            Color::BLACK,
        )?;

        // There doesn't seem to be a great way to modify an image before loading it onto the GPU.
        // Ideally I want to normalise the size of sprites when I load them, but instead I'm
        // loading it as-is and specifically scaling it differently during rendering time
        let boi = Image::from_bytes(ctx, include_bytes!("assets/bird_no_bg_32.png"))?;

        let vision = Mesh::new_circle(
            ctx,
            DrawMode::stroke(0.1 * mesh_raster_scale),
            [0., 0.],
            1. * mesh_raster_scale,
            2.,
            Color::new(0., 0., 0., 0.2), // Grey
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
