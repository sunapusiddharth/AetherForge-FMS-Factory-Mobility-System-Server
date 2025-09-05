use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use chrono::{DateTime, Utc, TimeDelta};
use rand::prelude::*;
use uuid::Uuid;
use tokio::time::sleep;

// === DATA STRUCTURES ===

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rotation {
    pub pitch: f64,
    pub yaw: f64,
    pub roll: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Calibration {
    pub intrinsic: IntrinsicParams,
    pub extrinsic: ExtrinsicParams,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntrinsicParams {
    pub focal_length: f64,
    pub distortion: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtrinsicParams {
    pub translation: [f64; 3],
    pub rotation: [f64; 3],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CameraHealth {
    pub fps: u32,
    pub bandwidth: f64, // Mbps
    pub temperature: f64, // Celsius
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Camera {
    pub id: String,
    pub position: Position,
    pub orientation: Rotation,
    pub fov_horizontal: f64,
    pub fov_vertical: f64,
    #[serde(rename = "type")]
    pub camera_type: String,
    pub status: String,
    pub calibration: Calibration,
    pub health: CameraHealth,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Robot {
    pub id: String,
    #[serde(rename = "type")]
    pub robot_type: String, // "AGV", "AMR", "Forklift"
    pub position: Position,
    pub orientation: f64,
    pub velocity: f64,
    pub battery: u8,
    pub task: String,
    pub task_id: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Human {
    pub id: String,
    pub position: Position,
    pub path_pattern: String,
    pub current_path_index: usize,
    pub workstation: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Obstacle {
    pub id: String,
    #[serde(rename = "type")]
    pub obstacle_type: String,
    pub position: Position,
    pub size: Size,
    pub is_static: bool,
    pub lifespan: Option<u32>, // in simulation steps
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SemanticCell {
    pub cell_id: String,
    pub position: Position,
    pub size: Size,
    #[serde(rename = "type")]
    pub cell_type: String, // "pathway", "workstation", "forbidden_zone"
    pub risk_level: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Detection {
    pub id: String,
    #[serde(rename = "type")]
    pub detection_type: String,
    pub subtype: Option<String>,
    pub position: Position,
    pub confidence: f64,
    pub source_cameras: Vec<String>,
    pub is_static: bool,
    pub lifespan: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorldModel {
    pub timestamp: DateTime<Utc>,
    pub zone_id: String,
    pub detections: Vec<Detection>,
    pub semantic_map: Vec<SemanticCell>,
    pub active_cameras: Vec<String>,
    pub fusion_confidence: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NavigationCommand {
    pub robot_id: String,
    pub command: String,
    pub target: Option<Position>,
    pub velocity: f64,
    pub hazard_alert: Option<String>,
    pub hazard_distance: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemAlert {
    pub id: String,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperatorEvent {
    pub event_type: String,
    pub status: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

// === SIMULATOR ===

pub struct WarehouseSimulator {
    warehouse_size: (usize, usize),
    robots: Vec<Robot>,
    humans: Vec<Human>,
    static_obstacles: Vec<Obstacle>,
    dynamic_obstacles: Vec<Obstacle>,
    camera_network: Vec<Camera>,
    current_time: DateTime<Utc>,
    model_confidence: f64,
    traffic_heatmap: Vec<Vec<u32>>,
    rng: ThreadRng,
    step_count: u64,
}

impl WarehouseSimulator {
    pub fn new(warehouse_size: (usize, usize)) -> Self {
        let mut rng = thread_rng();
        let mut simulator = Self {
            warehouse_size,
            robots: Vec::new(),
            humans: Vec::new(),
            static_obstacles: Vec::new(),
            dynamic_obstacles: Vec::new(),
            camera_network: Vec::new(),
            current_time: Utc::now(),
            model_confidence: 0.95,
            traffic_heatmap: vec![vec![0; warehouse_size.1]; warehouse_size.0],
            rng,
            step_count: 0,
        };

        simulator.robots = simulator.generate_robots(15);
        simulator.humans = simulator.generate_humans(25);
        simulator.static_obstacles = simulator.generate_static_obstacles(30);
        simulator.camera_network = simulator.initialize_camera_network();

        simulator
    }

    fn generate_robots(&mut self, count: usize) -> Vec<Robot> {
        (0..count)
            .map(|i| {
                let id = format!("ATR-{:03}", i + 1);
                let x = self.rng.gen_range(10.0..(self.warehouse_size.0 as f64 - 10.0));
                let y = self.rng.gen_range(10.0..(self.warehouse_size.1 as f64 - 10.0));

                Robot {
                    id,
                    robot_type: ["AGV", "AMR", "Forklift"]
                        .choose(&mut self.rng)
                        .unwrap()
                        .to_string(),
                    position: Position { x, y, z: 0.0 },
                    orientation: self.rng.gen_range(0.0..360.0),
                    velocity: self.rng.gen_range(0.5..1.5),
                    battery: self.rng.gen_range(60..=100),
                    task: ["idle", "moving", "loading", "unloading"]
                        .choose(&mut self.rng)
                        .unwrap()
                        .to_string(),
                    task_id: format!("TASK-{}", Uuid::new_v4().to_string()[..6].to_uppercase()),
                    last_seen: self.current_time,
                }
            })
            .collect()
    }

    fn generate_humans(&mut self, count: usize) -> Vec<Human> {
        (0..count)
            .map(|i| Human {
                id: format!("WORKER-{:03}", i + 1),
                position: Position {
                    x: self.rng.gen_range(5.0..(self.warehouse_size.0 as f64 - 5.0)),
                    y: self.rng.gen_range(5.0..(self.warehouse_size.1 as f64 - 5.0)),
                    z: 0.0,
                },
                path_pattern: ["patrol", "random", "workstation"]
                    .choose(&mut self.rng)
                    .unwrap()
                    .to_string(),
                current_path_index: 0,
                workstation: if self.rng.gen_bool(0.7) {
                    Some(self.rng.gen_range(1..=10))
                } else {
                    None
                },
            })
            .collect()
    }

    fn generate_static_obstacles(&mut self, count: usize) -> Vec<Obstacle> {
        (0..count)
            .map(|_| {
                let x = self.rng.gen_range(5.0..(self.warehouse_size.0 as f64 - 5.0));
                let y = self.rng.gen_range(5.0..(self.warehouse_size.1 as f64 - 5.0));

                Obstacle {
                    id: format!("OBST-{}", Uuid::new_v4().to_string()[..6]),
                    obstacle_type: ["pallet", "tool", "cabinet", "debris"]
                        .choose(&mut self.rng)
                        .unwrap()
                        .to_string(),
                    position: Position { x, y, z: 0.0 },
                    size: Size {
                        width: self.rng.gen_range(0.5..2.0),
                        height: self.rng.gen_range(0.5..1.5),
                    },
                    is_static: true,
                    lifespan: None,
                }
            })
            .collect()
    }

    fn initialize_camera_network(&mut self) -> Vec<Camera> {
        let mut cameras = Vec::new();
        let spacing = 20;

        for i in (0..self.warehouse_size.0).step_by(spacing) {
            for j in (0..self.warehouse_size.1).step_by(spacing) {
                if self.rng.gen_bool(0.8) {
                    let cam_id = format!("CAM-{:02}-{:02}", i / spacing, j / spacing);
                    cameras.push(Camera {
                        id: cam_id,
                        position: Position {
                            x: (i + 10) as f64,
                            y: (j + 10) as f64,
                            z: 5.0,
                        },
                        orientation: Rotation {
                            pitch: -30.0,
                            yaw: 0.0,
                            roll: 0.0,
                        },
                        fov_horizontal: 90.0,
                        fov_vertical: 60.0,
                        camera_type: "industrial".to_string(),
                        status: "online".to_string(),
                        calibration: Calibration {
                            intrinsic: IntrinsicParams {
                                focal_length: 2.8,
                                distortion: 0.05,
                            },
                            extrinsic: ExtrinsicParams {
                                translation: [(i + 10) as f64, (j + 10) as f64, 5.0],
                                rotation: [0.0, 0.0, 0.0],
                            },
                        },
                        health: CameraHealth {
                            fps: 25,
                            bandwidth: self.rng.gen_range(10.5..=15.0),
                            temperature: self.rng.gen_range(35.0..=45.0),
                        },
                    });
                }
            }
        }
        cameras
    }

    fn update_robot_positions(&mut self) {
        for robot in &mut self.robots {
            if robot.task == "moving" {
                let dx = self.rng.gen_range(-0.5..0.5);
                let dy = self.rng.gen_range(-0.5..0.5);

                robot.position.x = (robot.position.x + dx).max(0.0).min(self.warehouse_size.0 as f64);
                robot.position.y = (robot.position.y + dy).max(0.0).min(self.warehouse_size.1 as f64);

                if self.rng.gen_bool(0.1) {
                    robot.orientation = (robot.orientation + self.rng.gen_range(-30.0..30.0)) % 360.0;
                }
            }

            if robot.task != "idle" {
                robot.battery = robot.battery.saturating_sub(1);
            }

            if self.rng.gen_bool(0.05) {
                robot.task = ["idle", "moving", "loading", "unloading"]
                    .choose(&mut self.rng)
                    .unwrap()
                    .to_string();
                robot.task_id = format!("TASK-{}", Uuid::new_v4().to_string()[..6].to_uppercase());
            }

            robot.last_seen = self.current_time;
        }
    }

    fn update_human_positions(&mut self) {
        use std::f64::consts::PI;

        for human in &mut self.humans {
            match human.path_pattern.as_str() {
                "patrol" => {
                    human.current_path_index = (human.current_path_index + 1) % 10;
                    let angle = (human.current_path_index as f64 / 10.0) * 2.0 * PI;
                    human.position.x = (self.warehouse_size.0 / 2) as f64 + 15.0 * angle.cos();
                    human.position.y = (self.warehouse_size.1 / 2) as f64 + 15.0 * angle.sin();
                }
                "workstation" => {
                    if let Some(ws) = human.workstation {
                        let ws_x = ((ws % 5) * 20 + 10) as f64;
                        let ws_y = ((ws / 5) * 15 + 10) as f64;
                        human.position.x += 0.3 * (ws_x - human.position.x);
                        human.position.y += 0.3 * (ws_y - human.position.y);
                    }
                }
                _ => {
                    human.position.x += self.rng.gen_range(-0.8..0.8);
                    human.position.y += self.rng.gen_range(-0.8..0.8);
                    human.position.x = human.position.x.max(5.0).min(self.warehouse_size.0 as f64 - 5.0);
                    human.position.y = human.position.y.max(5.0).min(self.warehouse_size.1 as f64 - 5.0);
                }
            }
        }
    }

    fn generate_dynamic_obstacles(&mut self) -> Vec<Obstacle> {
        let mut obstacles = Vec::new();
        if self.rng.gen_bool(0.3) {
            let x = self.rng.gen_range(10.0..(self.warehouse_size.0 as f64 - 10.0));
            let y = self.rng.gen_range(10.0..(self.warehouse_size.1 as f64 - 10.0));

            obstacles.push(Obstacle {
                id: format!("DYN-{}", Uuid::new_v4().to_string()[..6]),
                obstacle_type: ["fallen_pallet", "tool", "box", "unknown_debris"]
                    .choose(&mut self.rng)
                    .unwrap()
                    .to_string(),
                position: Position { x, y, z: 0.0 },
                size: Size {
                    width: self.rng.gen_range(0.3..1.2),
                    height: self.rng.gen_range(0.3..0.8),
                },
                is_static: false,
                lifespan: Some(self.rng.gen_range(5..=30)),
            });
        }
        obstacles
    }

    fn update_dynamic_obstacles(&mut self) {
        // Remove expired obstacles
        self.dynamic_obstacles.retain(|obstacle| {
            if let Some(lifespan) = obstacle.lifespan {
                lifespan > 0
            } else {
                true
            }
        });

        // Decrement lifespan of remaining obstacles
        for obstacle in &mut self.dynamic_obstacles {
            if let Some(lifespan) = &mut obstacle.lifespan {
                *lifespan -= 1;
            }
        }

        // Generate new obstacles
        let new_obstacles = self.generate_dynamic_obstacles();
        self.dynamic_obstacles.extend(new_obstacles);
    }

    fn simulate_camera_failures(&mut self) {
        for cam in &mut self.camera_network {
            if cam.status == "offline" {
                if self.rng.gen_bool(0.1) {
                    cam.status = "online".to_string();
                }
                continue;
            }

            if self.rng.gen_bool(0.05) {
                cam.calibration.extrinsic.rotation[2] += self.rng.gen_range(-0.5..0.5);
            }

            if self.rng.gen_bool(0.02) {
                cam.status = "offline".to_string();
            }

            cam.health.temperature = (cam.health.temperature + self.rng.gen_range(-0.5..0.5))
                .max(30.0)
                .min(50.0);
            cam.health.fps = ((cam.health.fps as i32) + self.rng.gen_range(-1..=1))
                .max(15)
                .min(30) as u32;
        }
    }

    fn generate_semantic_map(&self) -> Vec<SemanticCell> {
        let grid_size = 5;
        let rows = self.warehouse_size.0 / grid_size;
        let cols = self.warehouse_size.1 / grid_size;

        (0..rows)
            .flat_map(|i| {
                (0..cols).map(move |j| {
                    let cell_type = if (i + j) % 3 == 0 {
                        "workstation"
                    } else if (i + j) % 5 == 0 {
                        "forbidden_zone"
                    } else {
                        "pathway"
                    };

                    SemanticCell {
                        cell_id: format!("CELL-{}-{}", i, j),
                        position: Position {
                            x: (i * grid_size) as f64,
                            y: (j * grid_size) as f64,
                            z: 0.0,
                        },
                        size: Size {
                            width: grid_size as f64,
                            height: grid_size as f64,
                        },
                        cell_type: cell_type.to_string(),
                        risk_level: match cell_type {
                            "pathway" => 1,
                            "workstation" => 2,
                            _ => 3,
                        },
                    }
                })
            })
            .collect()
    }

    fn generate_fused_world_model(&mut self) -> WorldModel {
        let active_cameras: Vec<&Camera> = self
            .camera_network
            .iter()
            .filter(|c| c.status == "online")
            .collect();

        let mut detections = Vec::new();

        // Robots
        for robot in &self.robots {
            if self.rng.gen_bool(0.9) {
                let confidence = (self.model_confidence - self.rng.gen_range(0.0..0.2)).max(0.6);
                detections.push(Detection {
                    id: robot.id.clone(),
                    detection_type: "robot".to_string(),
                    subtype: Some(robot.robot_type.clone()),
                    position: robot.position.clone(),
                    confidence,
                    source_cameras: active_cameras
                        .choose_multiple(&mut self.rng, 2.min(active_cameras.len()))
                        .map(|c| c.id.clone())
                        .collect(),
                    is_static: false,
                    lifespan: None,
                });
            }
        }

        // Humans
        for human in &self.humans {
            if self.rng.gen_bool(0.8) {
                let confidence = (self.model_confidence - self.rng.gen_range(0.1..0.3)).max(0.5);
                detections.push(Detection {
                    id: human.id.clone(),
                    detection_type: "human".to_string(),
                    subtype: None,
                    position: human.position.clone(),
                    confidence,
                    source_cameras: active_cameras
                        .choose_multiple(&mut self.rng, 2.min(active_cameras.len()))
                        .map(|c| c.id.clone())
                        .collect(),
                    is_static: false,
                    lifespan: None,
                });
            }
        }

        // Static obstacles
        for obstacle in &self.static_obstacles {
            detections.push(Detection {
                id: obstacle.id.clone(),
                detection_type: "obstacle".to_string(),
                subtype: Some(obstacle.obstacle_type.clone()),
                position: obstacle.position.clone(),
                confidence: 0.98,
                source_cameras: Vec::new(),
                is_static: true,
                lifespan: None,
            });
        }

        // Dynamic obstacles
        for obstacle in &self.dynamic_obstacles {
            detections.push(Detection {
                id: obstacle.id.clone(),
                detection_type: "obstacle".to_string(),
                subtype: Some(obstacle.obstacle_type.clone()),
                position: obstacle.position.clone(),
                confidence: 0.85,
                source_cameras: Vec::new(),
                is_static: false,
                lifespan: obstacle.lifespan,
            });
        }

        WorldModel {
            timestamp: self.current_time,
            zone_id: "MAIN_WAREHOUSE".to_string(),
            detections,
            semantic_map: self.generate_semantic_map(),
            active_cameras: active_cameras.iter().map(|c| c.id.clone()).collect(),
            fusion_confidence: self.model_confidence,
        }
    }

    fn generate_navigation_commands(&self) -> Vec<NavigationCommand> {
        self.robots
            .iter()
            .filter(|r| r.task == "moving")
            .map(|robot| {
                let hazard = self.rng.gen_bool(0.15);
                NavigationCommand {
                    robot_id: robot.id.clone(),
                    command: if hazard { "stop" } else { "move_to" }.to_string(),
                    target: if !hazard {
                        Some(Position {
                            x: self.rng.gen_range(0.0..self.warehouse_size.0 as f64),
                            y: self.rng.gen_range(0.0..self.warehouse_size.1 as f64),
                            z: 0.0,
                        })
                    } else {
                        None
                    },
                    velocity: if hazard { robot.velocity * 0.7 } else { robot.velocity },
                    hazard_alert: if hazard {
                        if self.rng.gen_bool(0.5) {
                            Some("person_nearby".to_string())
                        } else {
                            Some("obstacle".to_string())
                        }
                    } else {
                        None
                    },
                    hazard_distance: if hazard {
                        Some(self.rng.gen_range(0.5..2.0))
                    } else {
                        None
                    },
                    timestamp: self.current_time,
                }
            })
            .collect()
    }

    fn generate_system_alerts(&self) -> Vec<SystemAlert> {
        let mut alerts = Vec::new();

        for cam in &self.camera_network {
            if cam.status == "offline" && self.rng.gen_bool(0.3) {
                alerts.push(SystemAlert {
                    id: format!("ALERT-{}", Uuid::new_v4().to_string()[..6]),
                    alert_type: "camera_failure".to_string(),
                    severity: "critical".to_string(),
                    message: format!("Camera {} offline", cam.id),
                    timestamp: self.current_time,
                    data: json!({ "camera_id": cam.id }),
                });
            }
        }

        if self.model_confidence < 0.75 && self.rng.gen_bool(0.2) {
            alerts.push(SystemAlert {
                id: format!("ALERT-{}", Uuid::new_v4().to_string()[..6]),
                alert_type: "model_degradation".to_string(),
                severity: "warning".to_string(),
                message: format!("AI model confidence degraded to {:.2}", self.model_confidence),
                timestamp: self.current_time,
                data: json!({ "confidence": self.model_confidence }),
            });
        }

        for robot in &self.robots {
            if robot.battery < 20 && self.rng.gen_bool(0.1) {
                alerts.push(SystemAlert {
                    id: format!("ALERT-{}", Uuid::new_v4().to_string()[..6]),
                    alert_type: "low_battery".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "Robot {} battery critical ({}%)",
                        robot.id, robot.battery
                    ),
                    timestamp: self.current_time,
                    data: json!({ "robot_id": robot.id, "battery": robot.battery }),
                });
            }
        }

        if self.rng.gen_bool(0.05) {
            if self.robots.len() >= 2 {
                let robot1 = self.robots.choose(&mut self.rng.clone()).unwrap();
                let robot2 = self.robots.choose(&mut self.rng.clone()).unwrap();
                
                if robot1.id != robot2.id {
                    let dx = robot1.position.x - robot2.position.x;
                    let dy = robot1.position.y - robot2.position.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < 2.0 {
                        alerts.push(SystemAlert {
                            id: format!("ALERT-{}", Uuid::new_v4().to_string()[..6]),
                            alert_type: "collision_risk".to_string(),
                            severity: "critical".to_string(),
                            message: format!(
                                "Collision risk between {} and {}",
                                robot1.id, robot2.id
                            ),
                            timestamp: self.current_time,
                            data: json!({
                                "robot1": robot1.id,
                                "robot2": robot2.id,
                                "distance": format!("{:.2}", distance)
                            }),
                        });
                    }
                }
            }
        }

        alerts
    }

    fn update_traffic_heatmap(&mut self) {
        for robot in &self.robots {
            if robot.task != "idle" {
                let x = robot.position.x as usize;
                let y = robot.position.y as usize;
                if x < self.warehouse_size.0 && y < self.warehouse_size.1 {
                    self.traffic_heatmap[x][y] += 1;
                }
            }
        }

        for human in &self.humans {
            let x = human.position.x as usize;
            let y = human.position.y as usize;
            if x < self.warehouse_size.0 && y < self.warehouse_size.1 {
                self.traffic_heatmap[x][y] += 1;
            }
        }
    }

    fn simulate_model_drift(&mut self) {
        self.model_confidence -= self.rng.gen_range(0.001..=0.005);
        self.model_confidence = self.model_confidence.max(0.5);
    }

    fn generate_operator_platform_events(&mut self) -> Vec<OperatorEvent> {
        let mut events = Vec::new();

        if self.rng.gen_bool(0.1) {
            if let Some(cam) = self.camera_network.choose(&mut self.rng) {
                events.push(OperatorEvent {
                    event_type: "camera_calibration".to_string(),
                    status: Some("completed".to_string()),
                    timestamp: self.current_time,
                    data: json!({
                        "action": "intrinsic_calibration",
                        "camera_id": cam.id,
                        "metrics": {
                            "before_error": format!("{:.2}", self.rng.gen_range(2.5..5.0)),
                            "after_error": format!("{:.2}", self.rng.gen_range(0.5..1.0))
                        }
                    }),
                });
            }
        }

        if self.model_confidence < 0.7 && self.rng.gen_bool(0.3) {
            events.push(OperatorEvent {
                event_type: "model_retraining".to_string(),
                status: Some("started".to_string()),
                timestamp: self.current_time,
                data: json!({
                    "reason": "performance_degradation",
                    "confidence_threshold": 0.7,
                    "current_confidence": format!("{:.2}", self.model_confidence)
                }),
            });
        }

        if self.rng.gen_bool(0.05) {
            events.push(OperatorEvent {
                event_type: "low_confidence_data".to_string(),
                status: None,
                timestamp: self.current_time,
                data: json!({
                    "frame_id": format!("FRAME-{}", Uuid::new_v4().to_string()[..8]),
                    "confidence": format!("{:.2}", self.rng.gen_range(0.4..0.6)),
                    "detections": [{
                        "type": ["person", "forklift", "debris"].choose(&mut self.rng).unwrap(),
                        "bounding_box": [
                            self.rng.gen_range(100..500),
                            self.rng.gen_range(100..500),
                            self.rng.gen_range(50..200),
                            self.rng.gen_range(50..200)
                        ]
                    }]
                }),
            });
        }

        events
    }

    pub async fn run_simulation(&mut self, duration_seconds: u64, output_dir: &str) {
        let output_path = Path::new(output_dir);
        fs::create_dir_all(output_path).expect("Failed to create output directory");

        println!(
            "Starting {} second simulation...",
            duration_seconds
        );

        // Create a single file for each type of data instead of individual files per timestamp
        let mut world_models = Vec::new();
        let mut navigation_commands = Vec::new();
        let mut alerts = Vec::new();
        let mut operator_events = Vec::new();

        for step in 0..duration_seconds {
            self.step_count += 1;
            
            // Update simulation state
            self.update_robot_positions();
            self.update_human_positions();
            self.update_dynamic_obstacles();
            self.simulate_camera_failures();
            self.simulate_model_drift();
            self.update_traffic_heatmap();
            self.current_time = self.current_time + TimeDelta::seconds(1);

            // Generate data
            let world_model = self.generate_fused_world_model();
            let nav_cmds = self.generate_navigation_commands();
            let system_alerts = self.generate_system_alerts();
            let op_events = self.generate_operator_platform_events();

            // Collect data
            world_models.push(world_model);
            navigation_commands.extend(nav_cmds);
            alerts.extend(system_alerts);
            operator_events.extend(op_events);

            // Save data periodically to avoid memory issues
            if step % 300 == 0 && step > 0 {
                self.save_data_chunk(
                    &output_path,
                    &world_models,
                    &navigation_commands,
                    &alerts,
                    &operator_events,
                    step,
                );
                
                // Clear the vectors for the next chunk
                world_models.clear();
                navigation_commands.clear();
                alerts.clear();
                operator_events.clear();
            }

            if step % 60 == 0 {
                println!(
                    "Simulated {}/{} seconds | Model confidence: {:.2}",
                    step, duration_seconds, self.model_confidence
                );
            }

            sleep(Duration::from_millis(10)).await;
        }

        // Save any remaining data
        if !world_models.is_empty() {
            self.save_data_chunk(
                &output_path,
                &world_models,
                &navigation_commands,
                &alerts,
                &operator_events,
                duration_seconds,
            );
        }

        println!("Simulation completed! Data saved to {:?}", output_path);
    }

    fn save_data_chunk(
        &self,
        output_path: &Path,
        world_models: &[WorldModel],
        navigation_commands: &[NavigationCommand],
        alerts: &[SystemAlert],
        operator_events: &[OperatorEvent],
        chunk_id: u64,
    ) {
        if !world_models.is_empty() {
            fs::write(
                output_path.join(format!("world_models_{}.json", chunk_id)),
                serde_json::to_string_pretty(world_models).unwrap(),
            )
            .unwrap();
        }

        if !navigation_commands.is_empty() {
            fs::write(
                output_path.join(format!("navigation_commands_{}.json", chunk_id)),
                serde_json::to_string_pretty(navigation_commands).unwrap(),
            )
            .unwrap();
        }

        if !alerts.is_empty() {
            fs::write(
                output_path.join(format!("alerts_{}.json", chunk_id)),
                serde_json::to_string_pretty(alerts).unwrap(),
            )
            .unwrap();
        }

        if !operator_events.is_empty() {
            fs::write(
                output_path.join(format!("operator_events_{}.json", chunk_id)),
                serde_json::to_string_pretty(operator_events).unwrap(),
            )
            .unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let mut simulator = WarehouseSimulator::new((120, 100));
    simulator.run_simulation(1800, "simulation_data").await;
}