mod constants;
use constants::*;

use std::sync::mpsc::{
    Sender,
    Receiver,
};


pub trait NeuralNet {
    fn response(&self, player_pos_sorted: [[f32;2]; PLAYER_COUNT]) -> ([f32;2], u32);
}


pub struct Characteristics {
    pub lifetime: u32,
    
    pub speed: f32,
    pub force: f32,

    pub tether_length: f32,
    pub tether_speed: f32,
    pub tether_drain: u32,

    pub kill_boost: u32,
}

pub struct Player<N: NeuralNet> {
    pub id: u32,
    pub characteristics: Characteristics,
    pub neural_net: N,
}

#[derive(Default, Clone, Copy)]
pub struct Position{
    pub id: u32,
    pub pos: [f32;2],
}

pub struct Score {
    pub id: u32,
    pub score: usize,
}

fn run_game<N: NeuralNet>(
    incoming_players: Receiver<Player<N>>,
    current_pos: Option<Sender<[Position; PLAYER_COUNT]>>,
    result: Sender<Score>,
    mid_update: bool,
) {
    let mut players = Vec::new();
    let mut positions = [[0.0;2];PLAYER_COUNT];
    let mut life: [isize; PLAYER_COUNT] = [0;PLAYER_COUNT];
    let mut tethers = [0;PLAYER_COUNT];
    let mut scores = [0;PLAYER_COUNT];

    for p in 0..PLAYER_COUNT {
        players.push(incoming_players.recv().expect("unable to initialise players"));
        positions[p] = spawn_loc(&positions);
        life[p] = players[p].characteristics.lifetime as isize;
    }

    let mut connected = true;
    
    while connected {
        if mid_update {
            for new_player in incoming_players.try_iter() {
                if let Some(idx) = (0..PLAYER_COUNT).find(|i| new_player.id == players[*i].id) {
                    players[idx].neural_net = new_player.neural_net;
                }
            }
        }

        let mut new_pos = positions;
        for player in 0..PLAYER_COUNT {
            if life[player] > 0 {
                let mut sorted_positions = positions;
                sorted_positions.sort_by(|a,b|{
                    distance_2(a, &positions[player]).partial_cmp(&distance_2(b, &positions[player])).expect("error sorting distances")
                });
                let (directions, tether) = players[player].neural_net.response(sorted_positions);

                let mut tether = (0..PLAYER_COUNT).find(|i| positions[*i] == sorted_positions[tether as usize]).expect("error correcting tether");

                if life[tether] <= 0 {
                    tether = player
                }

                tethers[player] = tether;

                let speed = if tether == player {
                    players[player].characteristics.speed
                } else {
                    players[player].characteristics.tether_speed
                };

                let normalise_scale = speed / (directions[0].powi(2) + directions[1].powi(2));

                new_pos[player][0] = positions[player][0] + (normalise_scale * directions[0]);
                new_pos[player][1] = positions[player][1] + (normalise_scale * directions[1]);
            }
        }

        let mut tether_pos = positions;
        for player in 0..PLAYER_COUNT {
            if life[player] > 0 {
                let tethered = tethers[player];
                if tethered != player {
                    life[tethered] -= players[player].characteristics.tether_drain as isize;
                    life[player] += players[player].characteristics.tether_drain as isize;

                    let last_vector = [
                        positions[player][0] - positions[tethered][0],
                        positions[player][1] - positions[tethered][1],
                    ];
                    let new_vector = [
                        new_pos[player][0] - new_pos[tethered][0],
                        new_pos[player][1] - new_pos[tethered][1],
                    ];

                    let extension = (new_vector[0].powi(2) + new_vector[1].powi(2)) - (last_vector[0].powi(2) + last_vector[1].powi(2));

                    if extension > 0.0 {
                        let total_force = players[player].characteristics.force + players[tethered].characteristics.force;
                        tether_pos[player][0] -= extension * new_vector[0] * (players[tethered].characteristics.force / total_force);
                        tether_pos[player][1] -= extension * new_vector[1] * (players[tethered].characteristics.force / total_force);

                        tether_pos[tethered][0] += extension * new_vector[0] * (players[player].characteristics.force / total_force);
                        tether_pos[tethered][1] += extension * new_vector[1] * (players[player].characteristics.force / total_force);
                    }
                }
            }
        }

        for player in 0..PLAYER_COUNT {
            if life[player] > 0 {
                positions[player] = tether_pos[player];
                for second in (player+1)..PLAYER_COUNT {
                    if life[second] > 0 {
                        let vector = [
                            tether_pos[player][0] - tether_pos[second][0],
                            tether_pos[player][1] - tether_pos[second][1],
                        ];
                        let overlap = (2.0 * PLAYER_RADIUS).powi(2) - (vector[0].powi(2) + vector[1].powi(2));
                        if overlap > 0.0 {
                            let total_force = players[player].characteristics.force + players[second].characteristics.force;
                            positions[player][0] += overlap * vector[0] * (players[second].characteristics.force / total_force);
                            positions[player][1] += overlap * vector[1] * (players[second].characteristics.force / total_force);

                            positions[second][0] -= overlap * vector[0] * (players[player].characteristics.force / total_force);
                            positions[second][1] -= overlap * vector[1] * (players[player].characteristics.force / total_force);
                        }
                    }
                    
                }
            }
        }

        for player in 0..PLAYER_COUNT {
            scores[player] += 1;
            life[player] -= 1;
            if life[player] > 0 {
                if positions[player][0].powi(2) + positions[player][0].powi(2) >= FIELD_RADIUS.powi(2) {
                    life[player] = 0;
                }

                let mut pit = 0;
                while (pit < PIT_COUNT) && life[player] > 0 { 
                    if distance_2(&PIT_LOCATIONS_RADIUS[pit].0, &positions[player]) <= (PIT_LOCATIONS_RADIUS[pit].1 + PLAYER_RADIUS).powi(2) {
                        life[player] = 0;
                    }
                    pit += 1;
                }
            }

            if life[player] < -DEATH_TIMEOUT {
                let score = Score{
                    id: players[player].id,
                    score: scores[player],
                };
                result.send(score).ok();
                scores[player] = 0;

                if let Ok(new_player) = incoming_players.recv() {
                    players[player] = new_player;
                    positions[player] = spawn_loc(&positions);
                    life[player] = players[player].characteristics.lifetime as isize;
                } else {
                    connected = false;
                }
            }
        }

        if let Some(pos_update) = current_pos.as_ref() {
            let mut return_pos = [Default::default(); PLAYER_COUNT];

            for player in 0..PLAYER_COUNT {
                return_pos[player] = Position{
                    id: players[player].id,
                    pos: positions[player]
                };
            }

            pos_update.send(return_pos).ok();
        }
    }
}

// distance squared
fn distance_2(a: &[f32;2], b: &[f32;2]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}

fn spawn_loc(current_pos: &[[f32;2];PLAYER_COUNT]) -> [f32;2] {
    use rand::random;

    let mut free = false;
    let mut pos = [0.0f32;2];

    while !free {
        pos[0] = (random::<u8>() as f32 / u8::MAX as f32) * FIELD_RADIUS;
        pos[1] = (random::<u8>() as f32 / u8::MAX as f32) * FIELD_RADIUS;

        free = pos[0].powi(2) + pos[1].powi(2) < FIELD_RADIUS.powi(2);

        let mut player = 0;
        while (player < PLAYER_COUNT) && free { 
            free = distance_2(&current_pos[player], &pos) > (2.0 * PLAYER_RADIUS).powi(2);
            player += 1;
        }

        let mut pit = 0;
        while (pit < PIT_COUNT) && free { 
            free = distance_2(&PIT_LOCATIONS_RADIUS[pit].0, &pos) > (PIT_LOCATIONS_RADIUS[pit].1 + PLAYER_RADIUS).powi(2);
            pit += 1;
        }
    }

    pos
}