use lntrn_math::Vec2;

pub(crate) const WORLD_SIZE: Vec2 = Vec2::new(3000.0, 2000.0);
/// A scout is fast: an average ship moves 100 per turn. It crosses the map in about
/// twenty turns and a sector in about seven.
pub(crate) const SCOUT_SPEED: f64 = 150.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Player,
    Dominion,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Fleet {
    pub(crate) id: u32,
    pub(crate) name: &'static str,
    pub(crate) owner: Side,
    pub(crate) position: Vec2,
    pub(crate) speed: f64,
    pub(crate) remaining: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Game {
    pub(crate) turn: u64,
    pub(crate) active_side: Side,
    pub(crate) fleets: Vec<Fleet>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            turn: 1,
            active_side: Side::Player,
            fleets: vec![Fleet {
                id: 0,
                name: "Farlight Scout",
                owner: Side::Player,
                position: Vec2::new(300.0, 1000.0),
                speed: SCOUT_SPEED,
                remaining: SCOUT_SPEED,
            }],
        }
    }
}
