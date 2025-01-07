use imgui::ImColor32;
use pubg::state::StateLocalPlayerInfo;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

pub struct ViewController {
    pub camera_position: nalgebra::Vector3<f32>,
    pub camera_fov: f32,
    pub vaxisx: nalgebra::Vector3<f32>,
    pub vaxisy: nalgebra::Vector3<f32>,
    pub vaxisz: nalgebra::Vector3<f32>,
    pub screen_bounds: mint::Vector2<f32>,
}

impl State for ViewController {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            camera_position: Default::default(),
            camera_fov: Default::default(),
            vaxisx: Default::default(),
            vaxisy: Default::default(),
            vaxisz: Default::default(),
            screen_bounds: mint::Vector2 { x: 0.0, y: 0.0 },
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let local_player = states.resolve::<StateLocalPlayerInfo>(())?;

        // Convert Euler angles to rotation matrix
        let (pitch, yaw, roll) = (
            local_player.rotation[0].to_radians(),
            local_player.rotation[1].to_radians(),
            local_player.rotation[2].to_radians(),
        );

        let sp = pitch.sin();
        let cp = pitch.cos();
        let sy = yaw.sin();
        let cy = yaw.cos();
        let sr = roll.sin();
        let cr = roll.cos();

        let rotation = nalgebra::Matrix3::new(
            cp * cy,
            cp * sy,
            sp,
            sr * sp * cy - cr * sy,
            sr * sp * sy + cr * cy,
            -sr * cp,
            -(cr * sp * cy + sr * sy),
            -(cr * sp * sy - sr * cy),
            cr * cp,
        );

        let view_matrix = nalgebra::Matrix4::new(
            rotation[(0, 0)],
            rotation[(0, 1)],
            rotation[(0, 2)],
            0.0,
            rotation[(1, 0)],
            rotation[(1, 1)],
            rotation[(1, 2)],
            0.0,
            rotation[(2, 0)],
            rotation[(2, 1)],
            rotation[(2, 2)],
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        self.camera_position = nalgebra::Vector3::new(
            local_player.location[0],
            local_player.location[1],
            local_player.location[2],
        );
        self.camera_fov = local_player.fov_angle;

        self.vaxisx = nalgebra::Vector3::new(
            view_matrix[(0, 0)],
            view_matrix[(0, 1)],
            view_matrix[(0, 2)],
        );
        self.vaxisy = nalgebra::Vector3::new(
            view_matrix[(1, 0)],
            view_matrix[(1, 1)],
            view_matrix[(1, 2)],
        );
        self.vaxisz = nalgebra::Vector3::new(
            view_matrix[(2, 0)],
            view_matrix[(2, 1)],
            view_matrix[(2, 2)],
        );

        Ok(())
    }
}

impl ViewController {
    pub fn update_screen_bounds(&mut self, bounds: mint::Vector2<f32>) {
        self.screen_bounds = bounds;
    }

    /// Returning an mint::Vector2<f32> as the result should be used via ImGui.
    pub fn world_to_screen(
        &self,
        world_position: &nalgebra::Vector3<f32>,
        allow_of_screen: bool,
    ) -> Option<mint::Vector2<f32>> {
        let vdelta = *world_position - self.camera_position;
        let vtransformed = nalgebra::Vector3::new(
            vdelta.dot(&self.vaxisy),
            vdelta.dot(&self.vaxisz),
            vdelta.dot(&self.vaxisx),
        );

        if vtransformed.z < 0.0001 {
            return None;
        }

        let fov_angle = self.camera_fov;
        let screen_center_x = self.screen_bounds.x / 2.0;
        let screen_center_y = self.screen_bounds.y / 2.0;
        let screen_location_x = screen_center_x
            + vtransformed.x * (screen_center_x / (fov_angle * std::f32::consts::PI / 360.0).tan())
                / vtransformed.z;
        let screen_location_y = screen_center_y
            - vtransformed.y * (screen_center_x / (fov_angle * std::f32::consts::PI / 360.0).tan())
                / vtransformed.z;

        if !allow_of_screen {
            if screen_location_x > self.screen_bounds.x
                || screen_location_y > self.screen_bounds.y
                || screen_location_x < 0.0
                || screen_location_y < 0.0
            {
                return None;
            }
        }

        Some(mint::Vector2 {
            x: screen_location_x,
            y: screen_location_y,
        })
    }

    pub fn calculate_box_2d(
        &self,
        vmin: &nalgebra::Vector3<f32>,
        vmax: &nalgebra::Vector3<f32>,
    ) -> Option<(nalgebra::Vector2<f32>, nalgebra::Vector2<f32>)> {
        type Vec3 = nalgebra::Vector3<f32>;
        type Vec2 = nalgebra::Vector2<f32>;

        let points = [
            /* bottom */
            Vec3::new(vmin.x, vmin.y, vmin.z),
            Vec3::new(vmax.x, vmin.y, vmin.z),
            Vec3::new(vmin.x, vmax.y, vmin.z),
            Vec3::new(vmax.x, vmax.y, vmin.z),
            /* top */
            Vec3::new(vmin.x, vmin.y, vmax.z),
            Vec3::new(vmax.x, vmin.y, vmax.z),
            Vec3::new(vmin.x, vmax.y, vmax.z),
            Vec3::new(vmax.x, vmax.y, vmax.z),
        ];

        let mut min2d = Vec2::new(f32::MAX, f32::MAX);
        let mut max2d = Vec2::new(-f32::MAX, -f32::MAX);

        for point in points {
            if let Some(point) = self.world_to_screen(&point, true) {
                min2d.x = min2d.x.min(point.x);
                min2d.y = min2d.y.min(point.y);

                max2d.x = max2d.x.max(point.x);
                max2d.y = max2d.y.max(point.y);
            }
        }

        if min2d.x >= max2d.x {
            return None;
        }

        if min2d.y >= max2d.y {
            return None;
        }

        Some((min2d, max2d))
    }

    pub fn draw_box_3d(
        &self,
        draw: &imgui::DrawListMut,
        vmin: &nalgebra::Vector3<f32>,
        vmax: &nalgebra::Vector3<f32>,
        color: ImColor32,
        thickness: f32,
    ) {
        type Vec3 = nalgebra::Vector3<f32>;

        let lines = [
            /* bottom */
            (
                Vec3::new(vmin.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmin.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmin.y, vmax.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmin.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmin.y, vmin.z),
            ),
            /* top */
            (
                Vec3::new(vmin.x, vmax.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmax.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmax.x, vmax.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmax.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmin.z),
            ),
            /* corners */
            (
                Vec3::new(vmin.x, vmin.y, vmin.z),
                Vec3::new(vmin.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmin.z),
                Vec3::new(vmax.x, vmax.y, vmin.z),
            ),
            (
                Vec3::new(vmax.x, vmin.y, vmax.z),
                Vec3::new(vmax.x, vmax.y, vmax.z),
            ),
            (
                Vec3::new(vmin.x, vmin.y, vmax.z),
                Vec3::new(vmin.x, vmax.y, vmax.z),
            ),
        ];

        for (start, end) in lines {
            if let (Some(start), Some(end)) = (
                self.world_to_screen(&start, true),
                self.world_to_screen(&end, true),
            ) {
                draw.add_line(start, end, color)
                    .thickness(thickness)
                    .build();
            }
        }
    }
}
