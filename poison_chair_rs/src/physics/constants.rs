pub const FIELD_RADIUS: f32 = 16.0;

pub const PLAYER_COUNT: usize = 6;
pub const PLAYER_RADIUS: f32 = 0.5;

pub const PIT_COUNT: usize = 10;
pub const PIT_LOCATIONS_RADIUS: [([f32;2], f32); PIT_COUNT] = [
    ([  0.0 ,  0.0 ], 2.0),

    ([  8.0 ,  0.0 ], 1.0),
    ([ -4.0 ,  6.93], 1.0),
    ([ -4.0 , -6.93], 1.0),

    ([ 10.39,  6.0 ], 0.5),
    ([  0.0 , 12.0 ], 0.5),
    ([-10.39,  6.0 ], 0.5),
    ([-10.39, -6.0 ], 0.5),
    ([  0.0 ,-12.0 ], 0.5),
    ([ 10.39, -6.0 ], 0.5),
];

pub const DEATH_TIMEOUT: isize = 100;