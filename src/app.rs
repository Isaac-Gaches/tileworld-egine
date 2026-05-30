use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::engine::render::Renderer;
use crate::game::game::Game;

pub struct App{
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    assets: Option<AssetRegistry>,
    file_manager: Arc<FileManager>,
    input_manager: InputManager,
    game: Game,
    last_update_time: Instant,
    light_update_timer: Instant,
}

impl App{
    pub fn new()->Self{
        Self{
            window: None,
            renderer: None,
            assets: None,
            file_manager: Arc::new(FileManager::new()),
            input_manager: Default::default(),
            game: Game::new(),
            last_update_time: Instant::now(),
            light_update_timer: Instant::now(),
        }
    }
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window"));

        let mut renderer = Renderer::new(window.clone());
        let assets = AssetRegistry::new(&mut renderer);
        self.game.item_registry.load(&assets);

        self.game.chunk_manager.set_mesh_materials(vec![
            renderer.mesh_engine.bg_mesh_material.clone(),
            renderer.mesh_engine.fg_mesh_material.clone()
        ]);

        self.game.generate_terrain(&mut renderer.egpu, &self.file_manager);

        self.game.spawn_player(&mut renderer);

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.assets = Some(assets);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent)  {
        let renderer = self.renderer.as_mut().unwrap();
        let assets = self.assets.as_mut().unwrap();

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_manager.handle_keyboard(&event);
            }
            WindowEvent::MouseInput { button,state,..} =>{
                self.input_manager.handle_mouse_buttons(&button,&state);
            }
            WindowEvent::CursorMoved {position,..} =>{
                self.input_manager.handle_mouse_move(&position,renderer.egpu.width(),renderer.egpu.height());
                self.input_manager.update_mouse_world_pos(&renderer.camera);
            }
            WindowEvent::CloseRequested => {
                self.game.chunk_manager.save_chunks(&self.file_manager);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.egpu.resize_surface(size);
            }

            WindowEvent::RedrawRequested => {
                let dt = self.last_update_time.elapsed().as_secs_f32();
                self.last_update_time = Instant::now();

                self.game.update(&mut renderer.egpu,&self.file_manager,&mut self.input_manager,assets,dt);
                renderer.update(&self.input_manager, self.game.player_position,dt);

                if self.game.chunk_manager.dirty || self.light_update_timer.elapsed().as_secs_f32() > 0.0{
                    self.game.extract_lights(&mut renderer.lighting_engine);
                    renderer.lighting_engine.update(&mut renderer.egpu, self.game.extract_tiles(), self.game.player_position);
                }

                let frame = renderer.egpu.begin_frame();

                if self.game.chunk_manager.dirty || self.light_update_timer.elapsed().as_secs_f32() > 0.0{
                    renderer.lighting_engine.compute(frame);
                    self.game.chunk_manager.dirty = false;
                    self.light_update_timer = Instant::now();
                }

                renderer.sky.draw(frame);
                renderer.sprite_batch_engine.draw_sprites(frame,&self.game.world);
                self.game.draw(frame);

                frame.sort_by_material();
                renderer.egpu.render();

            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}