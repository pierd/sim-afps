use ambient_api::{prelude::*, core::{player::components::local_user_id, messages::Frame, app::components::window_logical_size}};
use packages::{afps_schema::{components::{player_name, player_cam_ref}, messages::{StartGame, Input}}, this::{components::is_simulated, messages::StopSim}};

const FRAMES_BETWEEN_ACTION_CHANGE: i32 = 60;

fn random_input() -> Input {
    let jump = random::<usize>() % 4 == 0;
    Input {
        direction: Vec2::new((random::<usize>() % 3) as f32 - 1.0, (random::<usize>() % 3) as f32 - 1.0),    // -1, 0, 1 for both x and y
        mouse_delta: Vec2::ZERO,
        shoot: false,
        toggle_zoom: false,
        is_shooting: false,
        duck: !jump && random::<usize>() % 4 == 0,
        jump,
        running: random(),
        ray_origin: Vec3::ZERO,
        ray_dir: Vec3::ZERO,
    }
}

#[main]
pub fn main() {
    run_async(async {
        sleep(5.0).await;
        let player_id = player::get_local();
        let name = entity::get_component(player_id, local_user_id()).unwrap_or_else(|| format!("bot-{}", player_id));
        if !entity::has_component(player_id, player_name()) {
            StartGame::new(name).send_server_reliable()
        }
    });

    let mut frame = 0;
    let mut input_msg = random_input();
    Frame::subscribe(move |_| {
        let player_id = player::get_local();

        let (delta, _input) = input::get_delta();
        if delta.keys.contains(&KeyCode::Space) {
            StopSim::new(player_id).send_server_reliable();
        }

        if !entity::get_component(player_id, is_simulated()).unwrap_or_default() {
            return;
        }
        frame += 1;
        if frame % FRAMES_BETWEEN_ACTION_CHANGE == 0 {
            input_msg = random_input();
        }

        let cam = entity::get_component(player_id, player_cam_ref());
        if cam.is_none() {
            return;
        }

        let cam = cam.unwrap();
        let window_size =
            entity::get_component(entity::resources(), window_logical_size()).unwrap();
        let Ray { origin, dir } = camera::screen_position_to_world_ray(
            cam,
            vec2(window_size.x as f32 / 2., window_size.y as f32 / 2.),
        );
        input_msg.ray_origin = origin;
        input_msg.ray_dir = dir;

        input_msg.send_server_unreliable();
    });
}
