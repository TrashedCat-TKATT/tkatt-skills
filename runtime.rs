// tkatt-skills for Horizon: a bar that turns experience into skill points, four
// trees to level and three big skills to unlock and toggle. This file is both the mod's Cargo library (its
// tests run on their own) and the module the bundle adds to the game, so it
// must not use inner attributes, external modules or include!.
//
// The game-side helpers in this bundle do one thing each: read the player's
// experience, call native_* helpers, and hand the maths to this file.

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "tkatt_skills";
/// The id this mod had before the rename: its stored files are carried over
/// on the first launch under the new id, so a rename loses no progress.
pub const LEGACY_ID: &str = "skill_trees";
pub const NAME: &str = "tkatt-skills";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Bank experience into a bar for skill points, then spend them on melee damage, armour, mining with your fists, or big skills like mining speed, Fortune and Haste. A framework for skill trees.";

/// One skill point per this much banked experience.
pub const XP_PER_POINT: f32 = 100.0;
/// Experience withdrawn from the player each tick the bar is held.
pub const XP_PER_TICK: u32 = 2;
/// Melee damage added per level: +8%, ten levels.
pub const MELEE_PER_LEVEL: f32 = 0.08;
/// Flat armour points added per level: 0.5 is half a damage point.
pub const ARMOR_POINTS_PER_LEVEL: f32 = 0.5;
/// The share of the remaining damage one Bulwark rank cuts: 0.25 is 25%.
pub const WARD_CUT_PER_RANK: f32 = 0.25;
/// The most the Bulwark ranks can cut, however far the tuning is pushed.
pub const WARD_CUT_MAX: f32 = 0.9;
/// Bulwark ranks: each one adds another cut.
pub const BULWARK_MAX: u32 = 3;
/// Every tree but the fist one stops here.
pub const MAX_LEVEL: u32 = 10;
/// Fist tiers: none, wood, stone, iron, diamond.
pub const FIST_MAX: u32 = 4;
/// The pickaxe speed a fist of each tier mines at (vanilla's materials).
pub const FIST_SPEEDS: [f32; 5] = [1.0, 2.0, 4.0, 6.0, 8.0];
/// The tiers' names, for the panel and the command.
pub const FIST_TIERS: [&str; 5] = ["Bare", "Wooden", "Stone", "Iron", "Diamond"];

/// The colours the box draws with, in the HUD's alpha-red-green-blue order.
pub const BACKGROUND: u32 = 0x7800_0000;
pub const FILL: u32 = 0xFF2E_7D32;
pub const READY: u32 = 0xFF55_FF55;
pub const INK: u32 = 0xFFFF_FFFF;
pub const DIM: u32 = 0xFFA0_A0A0;
pub const INK_DARK: u32 = 0x0014_1414;

/// The bar, and the panel under it, in GUI pixels.
pub const BAR: (f32, f32) = (182.0, 12.0);
pub const PANEL_Y: f32 = 14.0;
pub const ROW: f32 = 18.0;
/// Four levelled trees (melee, armour, bulwark, fist), then three big skills.
pub const ROWS: usize = 7;
/// The big skills' rows, in panel order.
pub const BIG_MINING: usize = 4;
pub const BIG_FORTUNE: usize = 5;
pub const BIG_HASTE: usize = 6;
/// How many big skills there are.
pub const BIG_ROWS: usize = ROWS - BIG_MINING;
/// A big skill's default last rank; the tuning can raise or lower it.
pub const BIG_MAX: u32 = 3;
/// The highest rank a stored state may keep, above any tuning's tier, so
/// lowering a tier does not erase bought ranks.
pub const BIG_CEILING: u32 = 10;

/// The modifiers' tunable numbers, kept in the mod's data folder as `tuning`
/// (next to `state`) so they change without a build, by editing that file or
/// with `/skills tune <tree> <field> <value>`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tune {
    /// Mining speed added per rank: 0.5 is +50% at rank 1, +150% at rank 3.
    pub mining_step: f32,
    /// Chance of an extra drop per rank, rolled once per block.
    pub fortune_chance: f32,
    /// Ticks of Haste given on a break.
    pub haste_ticks: i32,
    /// Haste amplifier at rank 1; each further rank adds one.
    pub haste_amp: i32,
    /// Points one rank of each big skill costs.
    pub mining_cost: u32,
    pub fortune_cost: u32,
    pub haste_cost: u32,
    /// Melee damage added per level: 0.08 is +8%.
    pub melee_step: f32,
    /// Flat armour points added per level: 0.5 is half a point.
    pub armor_points: f32,
    /// The damage cut each Bulwark rank adds: 0.25 is 25%.
    pub ward_step: f32,
    /// Each big skill's last rank.
    pub mining_tier: u32,
    pub fortune_tier: u32,
    pub haste_tier: u32,
}

impl Default for Tune {
    fn default() -> Self {
        Self {
            mining_step: 0.5,
            fortune_chance: 0.25,
            haste_ticks: 200,
            haste_amp: 1,
            mining_cost: 1,
            fortune_cost: 1,
            haste_cost: 1,
            melee_step: MELEE_PER_LEVEL,
            armor_points: ARMOR_POINTS_PER_LEVEL,
            ward_step: WARD_CUT_PER_RANK,
            mining_tier: BIG_MAX,
            fortune_tier: BIG_MAX,
            haste_tier: BIG_MAX,
        }
    }
}

impl Tune {
    /// The points one rank of `row` costs (0 for a levelled tree).
    pub fn cost(&self, row: usize) -> u32 {
        match row {
            BIG_MINING => self.mining_cost,
            BIG_FORTUNE => self.fortune_cost,
            BIG_HASTE => self.haste_cost,
            _ => 0,
        }
    }

    /// The last rank of a big skill (0 for a levelled tree).
    pub fn max(&self, row: usize) -> u32 {
        match row {
            BIG_MINING => self.mining_tier,
            BIG_FORTUNE => self.fortune_tier,
            BIG_HASTE => self.haste_tier,
            _ => 0,
        }
    }

    /// The stored line: the big skills' numbers, then each row's point cost,
    /// the melee/armour steps and the big skills' tiers.
    pub fn encode(&self) -> String {
        format!(
            "3|{:.2}|{:.2}|{}|{}|{}|{}|{}|{:.2}|{:.2}|{:.2}|{}|{}|{}",
            self.mining_step,
            self.fortune_chance,
            self.haste_ticks,
            self.haste_amp,
            self.mining_cost,
            self.fortune_cost,
            self.haste_cost,
            self.melee_step,
            self.armor_points,
            self.ward_step,
            self.mining_tier,
            self.fortune_tier,
            self.haste_tier
        )
    }

    /// From `encode`'s line. A missing or malformed field keeps its default and
    /// every value is clamped, so a half-written file never makes a skill
    /// absurd or negative.
    pub fn decode(text: &str) -> Self {
        let mut out = Self::default();
        let mut parts = text.trim().split('|');
        let version = match parts.next() {
            Some("1") => 1,
            Some("2") => 2,
            Some("3") => 3,
            _ => return out,
        };
        let mut next = |default: f32, low: f32, high: f32| {
            parts
                .next()
                .and_then(|v| v.parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .map_or(default, |v| v.clamp(low, high))
        };
        out.mining_step = next(out.mining_step, 0.0, 5.0);
        out.fortune_chance = next(out.fortune_chance, 0.0, 1.0);
        out.haste_ticks = next(out.haste_ticks as f32, 20.0, 1200.0) as i32;
        out.haste_amp = next(out.haste_amp as f32, 0.0, 4.0) as i32;
        out.mining_cost = next(out.mining_cost as f32, 1.0, 10.0) as u32;
        out.fortune_cost = next(out.fortune_cost as f32, 1.0, 10.0) as u32;
        out.haste_cost = next(out.haste_cost as f32, 1.0, 10.0) as u32;
        out.melee_step = next(out.melee_step, 0.0, 1.0);
        // Version 1 stored a percentage here; its damage reduction moved to
        // the Bulwark row, so an old line takes the new flat default.
        let armor_slot = next(ARMOR_POINTS_PER_LEVEL, 0.0, 10.0);
        if version >= 2 {
            out.armor_points = armor_slot;
        }
        // Version 3 added the cut per Bulwark rank; older lines take the
        // default instead of reading a later field out of place.
        if version >= 3 {
            out.ward_step = next(WARD_CUT_PER_RANK, 0.0, 1.0);
        }
        out.mining_tier = next(out.mining_tier as f32, 1.0, 10.0) as u32;
        out.fortune_tier = next(out.fortune_tier as f32, 1.0, 10.0) as u32;
        out.haste_tier = next(out.haste_tier as f32, 1.0, 10.0) as u32;
        out
    }

    /// Sets one field for one big skill; a field that does not belong to it is
    /// refused with what that field accepts.
    pub fn set(&mut self, tree: Tree, field: Field, value: f32) -> Result<(), String> {
        if !value.is_finite() {
            return Err(format!("tkatt-skills: {}.", field.hint()));
        }
        match (tree, field) {
            (Tree::Mining, Field::Step) => self.mining_step = value.clamp(0.0, 5.0),
            (Tree::Fortune, Field::Chance) => self.fortune_chance = value.clamp(0.0, 1.0),
            (Tree::Haste, Field::Ticks) => self.haste_ticks = value.clamp(20.0, 1200.0) as i32,
            (Tree::Haste, Field::Amp) => self.haste_amp = value.clamp(0.0, 4.0) as i32,
            (Tree::Mining, Field::Cost) => self.mining_cost = value.clamp(1.0, 10.0) as u32,
            (Tree::Fortune, Field::Cost) => self.fortune_cost = value.clamp(1.0, 10.0) as u32,
            (Tree::Haste, Field::Cost) => self.haste_cost = value.clamp(1.0, 10.0) as u32,
            (Tree::Melee, Field::Damage) => self.melee_step = value.clamp(0.0, 1.0),
            (Tree::Armor, Field::Armor) => self.armor_points = value.clamp(0.0, 10.0),
            (Tree::Bulwark, Field::Ward) => self.ward_step = value.clamp(0.0, 1.0),
            (Tree::Mining, Field::Tier) => self.mining_tier = value.clamp(1.0, 10.0) as u32,
            (Tree::Fortune, Field::Tier) => self.fortune_tier = value.clamp(1.0, 10.0) as u32,
            (Tree::Haste, Field::Tier) => self.haste_tier = value.clamp(1.0, 10.0) as u32,
            _ => return Err(format!("tkatt-skills: {}.", field.hint())),
        }
        Ok(())
    }

    /// What `/skills tune` prints.
    pub fn show(&self) -> String {
        format!(
            "tkatt-skills tuning: melee +{:.0}%/level, armour +{:.1} points/level, bulwark {:.0}%/rank; mining +{:.0}%/rank, fortune {:.0}%/rank, haste {} ticks Haste {}; costs {} / {} / {}; ranks {} / {} / {}",
            self.melee_step * 100.0,
            self.armor_points,
            self.ward_step * 100.0,
            self.mining_step * 100.0,
            self.fortune_chance * 100.0,
            self.haste_ticks,
            self.haste_amp + 1,
            self.mining_cost,
            self.fortune_cost,
            self.haste_cost,
            self.mining_tier,
            self.fortune_tier,
            self.haste_tier
        )
    }
}

/// A big skill's tunable field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// Mining speed added per rank.
    Step,
    /// Chance of an extra drop per rank.
    Chance,
    /// Haste duration in ticks.
    Ticks,
    /// Haste amplifier at rank 1.
    Amp,
    /// Points a rank costs.
    Cost,
    /// Melee damage added per level.
    Damage,
    /// Flat armour points added per level.
    Armor,
    /// The damage cut each Bulwark rank adds.
    Ward,
    /// A big skill's last rank.
    Tier,
}

impl Field {
    pub fn from_word(word: &str) -> Option<Field> {
        match word {
            "step" | "speed" => Some(Field::Step),
            "chance" => Some(Field::Chance),
            "ticks" | "duration" => Some(Field::Ticks),
            "amp" | "amplifier" => Some(Field::Amp),
            "cost" => Some(Field::Cost),
            "damage" => Some(Field::Damage),
                        "points" | "armor" | "armour" => Some(Field::Armor),
            "cut" | "ward" | "reduction" => Some(Field::Ward),
            "tier" | "ranks" | "max" => Some(Field::Tier),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Field::Step => "step",
            Field::Chance => "chance",
            Field::Ticks => "ticks",
            Field::Amp => "amp",
            Field::Cost => "cost",
            Field::Damage => "damage",
            Field::Armor => "armor",
            Field::Ward => "cut",
            Field::Tier => "tier",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Field::Step => "step is the speed added a rank, 0..5: /skills tune mining step 0.5",
            Field::Chance => "chance is 0..1 a rank: /skills tune fortune chance 0.25",
            Field::Ticks => "ticks is 20..1200: /skills tune haste ticks 200",
            Field::Amp => "amp is 0..4: /skills tune haste amp 1",
            Field::Cost => "cost is 1..10 points a rank: /skills tune mining cost 1",
            Field::Damage => "damage is 0..1 a level: /skills tune melee damage 0.08",
            Field::Armor => "points is 0..10 a level: /skills tune armour points 0.5",
            Field::Ward => "cut is 0..1 a rank: /skills tune bulwark cut 0.25",
            Field::Tier => "tier is 1..10 ranks: /skills tune fortune tier 3",
        }
    }
}
pub const PANEL: (f32, f32) = (182.0, 2.0 + ROWS as f32 * ROW);
/// The "+" box on a row, from the panel's left edge.
pub const PLUS: f32 = 12.0;
pub const PLUS_X: f32 = 166.0;

/// The rows, in the order the panel lists them: four levelled trees, then
/// three big skills.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tree {
    Melee,
    Armor,
    Bulwark,
    Fist,
    Mining,
    Fortune,
    Haste,
}

/// Every row, in panel order.
pub const TREES: [Tree; ROWS] = [
    Tree::Melee,
    Tree::Armor,
    Tree::Bulwark,
    Tree::Fist,
    Tree::Mining,
    Tree::Fortune,
    Tree::Haste,
];

impl Tree {
    pub fn index(self) -> usize {
        match self {
            Tree::Melee => 0,
            Tree::Armor => 1,
            Tree::Bulwark => 2,
            Tree::Fist => 3,
            Tree::Mining => 4,
            Tree::Fortune => 5,
            Tree::Haste => 6,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Tree::Melee => "Melee damage",
            Tree::Armor => "Armour",
            Tree::Bulwark => "Bulwark",
            Tree::Fist => "Fist mining",
            Tree::Mining => "Mining speed",
            Tree::Fortune => "Fortune",
            Tree::Haste => "Haste",
        }
    }

    /// The tree a command word names: `melee`, `armour`/`armor`, `bulwark`
    /// (also `ward` or `reduction`), `fist`, `mining`, `fortune`, `haste`.
    pub fn from_word(word: &str) -> Option<Tree> {
        match word {
            "melee" => Some(Tree::Melee),
            "armor" | "armour" => Some(Tree::Armor),
            "bulwark" | "ward" | "reduction" => Some(Tree::Bulwark),
            "fist" => Some(Tree::Fist),
            "mining" => Some(Tree::Mining),
            "fortune" => Some(Tree::Fortune),
            "haste" => Some(Tree::Haste),
            _ => None,
        }
    }

    pub fn max(self) -> u32 {
        match self {
            Tree::Melee | Tree::Armor => MAX_LEVEL,
            // Each Bulwark rank adds another cut.
            Tree::Bulwark => BULWARK_MAX,
            // A fist tier is 0..=4: bare, wood, stone, iron, diamond.
            Tree::Fist => FIST_MAX,
            // A big skill: one point a rank, three ranks.
            _ => BIG_MAX,
        }
    }

    /// What one more level gives, for the row and the command. Melee and
    /// armour print the tuning's numbers.
    pub fn per_level(self, tune: Tune) -> String {
        match self {
            Tree::Melee => format!("+{:.0}% damage", tune.melee_step * 100.0),
            Tree::Armor => format!("+{:.1} armour", tune.armor_points),
            Tree::Bulwark => format!("{:.0}% cut", tune.ward_step * 100.0),
            Tree::Fist => "one pickaxe tier with your fists".to_owned(),
            Tree::Mining => "+50% mining speed".to_owned(),
            Tree::Fortune => "an extra drop".to_owned(),
            Tree::Haste => "Haste when a block breaks".to_owned(),
        }
    }

    /// A big skill: unlocked with one point, then toggled on and off.
    pub fn is_big(self) -> bool {
        self.index() >= BIG_MINING
    }
}

/// What a click on a row's "+" did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    /// A levelled tree went up.
    Spent(usize),
    /// A big skill was unlocked (and is on).
    Unlocked(usize),
    /// A big skill went up a rank.
    Ranked(usize),
    /// A big skill was switched on.
    On(usize),
    /// A big skill was switched off.
    Off(usize),
    /// No points to spend.
    NoPoints,
    /// The tree is finished.
    Maxed,
}

/// What the game's mining code tells the mining maths about a block and the
/// held item, so `Skills::mining_factor` can be tested without a game.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningFacts {
    /// A bare hand (or nothing) is held.
    pub bare: bool,
    /// The block carries `minecraft:mineable/pickaxe`.
    pub pickaxe: bool,
    /// The hand's own speed on the block, 0 when it cannot be mined by hand.
    pub base_speed: f32,
    /// The highest `needs_*_tool` tier the block asks for, 0 for none.
    pub needs: u32,
    /// Whether the held item already satisfies the block's tool tag.
    pub covers: bool,
}

/// A player's skill state: the bar's fill, the unspent points and each tree's
/// level, plus the experience it took in all.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skills {
    /// Banked experience towards the next point, 0..XP_PER_POINT.
    pub bank: f32,
    pub points: u32,
    /// One level per tree, in `TREES` order; a big skill's is 0 or 1.
    pub levels: [u32; ROWS],
    /// Big skills only: 1 on, 2 off, 0 while locked.
    pub big: [u8; BIG_ROWS],
    /// Experience this player has put into the bar, all told.
    pub spent_xp: i64,
}

impl Skills {
    /// Takes up to `XP_PER_TICK` of the player's `available` experience into
    /// the bar and returns how much was taken, so the caller withdraws
    /// exactly that. A full bar grants a point and carries the remainder.
    pub fn pump(&mut self, available: u32) -> u32 {
        let taken = available.min(XP_PER_TICK);
        if taken == 0 {
            return 0;
        }
        self.bank += taken as f32;
        self.spent_xp = self.spent_xp.saturating_add(i64::from(taken));
        while self.bank >= XP_PER_POINT {
            self.bank -= XP_PER_POINT;
            self.points = self.points.saturating_add(1);
        }
        taken
    }

    /// Spends a point on `tree`; false when there is no point or the tree is
    /// finished.
    pub fn spend(&mut self, tree: Tree) -> bool {
        let i = tree.index();
        if self.points == 0 || self.levels[i] >= tree.max() {
            return false;
        }
        self.points -= 1;
        self.levels[i] += 1;
        true
    }

    /// Whether the big skill on `row` (a `BIG_*` row) is switched on.
    pub fn big_on(&self, row: usize) -> bool {
        row >= BIG_MINING && self.levels[row] > 0 && self.big[row - BIG_MINING] == 1
    }

    /// A big skill's rank: 0 while locked, up to `BIG_MAX`.
    pub fn big_rank(&self, row: usize) -> u32 {
        if row >= BIG_MINING {
            self.levels[row].min(BIG_CEILING)
        } else {
            0
        }
    }

    /// The mining-speed multiplier at this rank and tuning.
    pub fn mining_multiplier(&self, tune: Tune) -> f32 {
        let rank = self.big_rank(BIG_MINING).min(tune.max(BIG_MINING));
        if self.big_on(BIG_MINING) {
            1.0 + tune.mining_step * rank as f32
        } else {
            1.0
        }
    }

    /// The factor a block's mining progress is scaled by: the big mining
    /// skill's rank, times the fist tier's speed ratio on a block a bare hand
    /// can mine, times the divisor move when the tier satisfies a tool tag the
    /// hand does not.
    pub fn mining_factor(&self, tune: Tune, f: MiningFacts) -> f32 {
        let mut factor = self.mining_multiplier(tune);
        if f.bare && f.pickaxe && f.base_speed > 0.0 && self.fist_tier() > 0 {
            factor *= self.fist_speed().max(f.base_speed) / f.base_speed;
            // A bare hand never satisfies `needs_*_tool`; with the tier it
            // does, and that moves the progress divisor from 100 to 30.
            if f.needs > 0 && !f.covers && self.fist_covers(f.needs) {
                factor *= 100.0 / 30.0;
            }
        }
        factor
    }

    /// The chance of an extra drop at this rank and tuning, 0 while off.
    pub fn fortune_chance(&self, tune: Tune) -> f32 {
        let rank = self.big_rank(BIG_FORTUNE).min(tune.max(BIG_FORTUNE));
        if self.big_on(BIG_FORTUNE) {
            (tune.fortune_chance * rank as f32).min(1.0)
        } else {
            0.0
        }
    }

    pub fn fortune_on(&self) -> bool {
        self.big_on(BIG_FORTUNE)
    }

    pub fn haste_on(&self) -> bool {
        self.big_on(BIG_HASTE)
    }

    /// Ticks of Haste a break gives, 0 while the skill is off.
    pub fn haste_ticks(&self, tune: Tune) -> i32 {
        if self.haste_on() {
            tune.haste_ticks
        } else {
            0
        }
    }

    /// The Haste amplifier a break gives; rank 1 is the tuning's, each rank
    /// after it one stronger.
    pub fn haste_amp(&self, tune: Tune) -> i32 {
        if self.haste_on() {
            let rank = self.big_rank(BIG_HASTE).min(tune.max(BIG_HASTE));
            tune.haste_amp + rank as i32 - 1
        } else {
            0
        }
    }

    /// A click on a row's "+": spends a point on a levelled tree, unlocks a big
    /// skill, or toggles it. `buy` (Shift held) buys the next rank of an
    /// unlocked big skill instead of toggling it.
    pub fn press(&mut self, tree: Tree, buy: bool, tune: Tune) -> Press {
        let row = tree.index();
        if tree.is_big() {
            let rank = self.big_rank(row);
            let cost = tune.cost(row);
            let max = tune.max(row);
            if rank == 0 || buy && rank < max {
                if self.points < cost {
                    return Press::NoPoints;
                }
                self.points -= cost;
                self.levels[row] = rank + 1;
                self.big[row - BIG_MINING] = 1;
                return if rank == 0 {
                    Press::Unlocked(row)
                } else {
                    Press::Ranked(row)
                };
            }
            let on = self.big[row - BIG_MINING] == 1;
            self.big[row - BIG_MINING] = if on { 2 } else { 1 };
            return if on { Press::Off(row) } else { Press::On(row) };
        }
        if self.spend(tree) {
            Press::Spent(row)
        } else if self.points == 0 {
            Press::NoPoints
        } else {
            Press::Maxed
        }
    }

    /// Melee damage multiplier: 1.0 at level 0, 1.8 at level 10.
    pub fn melee_multiplier(&self, tune: Tune) -> f32 {
        1.0 + tune.melee_step * self.levels[Tree::Melee.index()] as f32
    }

    /// Flat armour points the armour tree has bought at this tuning.
    pub fn armor_points(&self, tune: Tune) -> f32 {
        tune.armor_points * self.levels[Tree::Armor.index()] as f32
    }

    /// How much of the remaining damage the Bulwark ranks cut, capped so a
    /// tuned step can never make the player untouchable.
    pub fn ward_cut(&self, tune: Tune) -> f32 {
        (tune.ward_step * self.levels[Tree::Bulwark.index()] as f32).clamp(0.0, WARD_CUT_MAX)
    }

    /// Incoming damage: the armour points come off first, never below zero,
    /// then the Bulwark ranks cut a share of what is left.
    pub fn incoming_damage(&self, tune: Tune, value: f32) -> f32 {
        let after = (value - self.armor_points(tune)).max(0.0);
        after * (1.0 - self.ward_cut(tune))
    }

    /// The pickaxe speed a bare hand mines at: 1.0 until the first tier.
    pub fn fist_speed(&self) -> f32 {
        FIST_SPEEDS[self.fist_tier().min(FIST_MAX) as usize]
    }

    pub fn fist_tier(&self) -> u32 {
        self.levels[Tree::Fist.index()].min(FIST_MAX)
    }

    /// Whether the fist tier reaches the tool tier a block needs: 1 wood, 2
    /// stone, 3 iron, 4 diamond (0 needs none).
    pub fn fist_covers(&self, needs: u32) -> bool {
        self.fist_tier() >= needs
    }

    /// The fist tier on a block's drop decision: the game's own answer, or true
    /// when a bare hand carries a tier and that tier covers the block's
    /// `needs_*_tool` tag (0 for a block that needs no tool).
    pub fn fist_drop(&self, value: bool, needed: u32) -> bool {
        value || self.fist_tier() > 0 && self.fist_covers(needed)
    }

    /// The bar's fill, 0..=1.
    pub fn fill(&self) -> f32 {
        (self.bank / XP_PER_POINT).clamp(0.0, 1.0)
    }

    /// The stored form, one line:
    /// `2|bank|points|melee|armor|bulwark|fist|spent|mining|fortune|haste|b1|b2|b3`.
    /// Version 1 had no Bulwark row; `decode` still reads those lines.
    pub fn encode(&self) -> String {
        let mut out = format!("2|{:.2}|{}", self.bank, self.points);
        for i in 0..BIG_MINING {
            out.push_str(&format!("|{}", self.levels[i]));
        }
        out.push_str(&format!("|{}", self.spent_xp));
        for i in BIG_MINING..ROWS {
            out.push_str(&format!("|{}", self.levels[i]));
        }
        for b in self.big {
            out.push_str(&format!("|{b}"));
        }
        out
    }

    /// From `encode`'s line; a missing or malformed value is the default, so a
    /// half-written file never loses the player's trees.
    pub fn decode(text: &str) -> Self {
        let mut out = Self::default();
        let mut parts = text.trim().split('|');
        let version = match parts.next() {
            Some("1") => 1,
            Some("2") => 2,
            _ => return out,
        };
        out.bank = parts
            .next()
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .clamp(0.0, XP_PER_POINT);
        out.points = parts
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        if version == 1 {
            // Version 1's levelled rows were melee, armour, fist.
            let slots = [
                (0usize, MAX_LEVEL),
                (1, MAX_LEVEL),
                (Tree::Fist.index(), FIST_MAX),
            ];
            for (i, max) in slots {
                out.levels[i] = parts
                    .next()
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0)
                    .min(max);
            }
        } else {
            for i in 0..BIG_MINING {
                out.levels[i] = parts
                    .next()
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0)
                    .min(TREES[i].max());
            }
        }
        out.spent_xp = parts
            .next()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        for i in BIG_MINING..ROWS {
            out.levels[i] = parts
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0)
                .min(BIG_CEILING);
        }
        for b in &mut out.big {
            *b = parts
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(0)
                .min(2);
        }
        // A locked big skill has no on/off state to keep; an unlocked one
        // without one reads as on.
        for i in BIG_MINING..ROWS {
            let b = &mut out.big[i - BIG_MINING];
            if out.levels[i] == 0 {
                *b = 0;
            } else if *b == 0 {
                *b = 1;
            }
        }
        out
    }
}

thread_local! {
    /// The player's skills, read from the mod's settings folder once.
    static STATE: std::cell::RefCell<Option<Skills>> = const { std::cell::RefCell::new(None) };
    /// The tunable big-skill numbers, read from the same folder once.
    static TUNE: std::cell::Cell<Option<Tune>> = const { std::cell::Cell::new(None) };
    /// The panel open, the bar held (this client's own view).
    static UI: std::cell::Cell<(bool, bool)> = const { std::cell::Cell::new((false, false)) };
    /// Where the box was drawn this frame, in GUI pixels.
    static ORIGIN: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
    /// The mod is on in Options > Mods (on until a hook first sees it off).
    static ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    /// The old id's files were looked at this session.
    static CARRIED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the mod is on in Options > Mods, as the last hook that had a
/// `Game` saw it. The effect hooks run where no `Game` is in reach, so they
/// read this instead.
pub fn set_enabled(on: bool) {
    ENABLED.with(|c| c.set(on));
}

pub fn enabled() -> bool {
    ENABLED.with(|c| c.get())
}

/// The player's skills and the tuning, from their stored files the first time.
/// `get` reads one of the mod's data files by key (`state`, `tuning`).
pub fn load_once(get: impl Fn(&str) -> Option<String>) -> Skills {
    if TUNE.with(|c| c.get()).is_none() {
        store_tune(Tune::decode(&get("tuning").unwrap_or_default()));
    }
    if let Some(s) = STATE.with(|c| c.borrow().clone()) {
        return s;
    }
    let s = Skills::decode(&get("state").unwrap_or_default());
    store(s.clone());
    s
}

/// The skills as they are now (`Skills::default` before the first load).
pub fn current() -> Skills {
    STATE.with(|c| c.borrow().clone().unwrap_or_default())
}

/// Keeps the skills for this session; the caller writes the file.
pub fn store(s: Skills) {
    STATE.with(|c| *c.borrow_mut() = Some(s));
}

/// The tuning as it is now (`Tune::default` before the first load).
pub fn tuning() -> Tune {
    TUNE.with(|c| c.get()).unwrap_or_default()
}

/// Keeps the tuning for this session; the caller writes the file.
pub fn store_tune(t: Tune) {
    TUNE.with(|c| c.set(Some(t)));
}

/// The files to copy from [`LEGACY_ID`] into the new folder: each of `state`
/// and `tuning` that the new folder has not got yet, taken from the old one.
/// A blank old value is nothing to carry.
pub fn carry(
    old_state: Option<String>,
    old_tuning: Option<String>,
    new_state: Option<String>,
    new_tuning: Option<String>,
) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (key, old, new) in [
        ("state", old_state, new_state),
        ("tuning", old_tuning, new_tuning),
    ] {
        if new.is_none() {
            if let Some(text) = old.filter(|t| !t.trim().is_empty()) {
                out.push((key, text));
            }
        }
    }
    out
}

/// Whether the old id's files were looked at this session.
pub fn carried() -> bool {
    CARRIED.with(|c| c.get())
}

pub fn set_carried() {
    CARRIED.with(|c| c.set(true));
}

/// The panel open, the bar held.
pub fn ui() -> (bool, bool) {
    UI.with(|c| c.get())
}

pub fn set_ui(open: bool, holding: bool) {
    UI.with(|c| c.set((open, holding)));
}

/// Where the box was drawn this frame, for hit-testing the player's clicks.
pub fn origin() -> (f32, f32) {
    ORIGIN.with(|c| c.get())
}

pub fn set_origin(at: (f32, f32)) {
    ORIGIN.with(|c| c.set(at));
}

/// What the box draws this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct View {
    pub bank: f32,
    pub points: u32,
    pub levels: [u32; ROWS],
    pub big: [u8; BIG_ROWS],
    /// The points a rank of each big skill costs.
    pub costs: [u32; BIG_ROWS],
    /// Each big skill's last rank.
    pub maxes: [u32; BIG_ROWS],
    /// The tree panel is open under the bar.
    pub open: bool,
    /// The bar is being held: it draws lighter.
    pub holding: bool,
}

impl View {
    pub fn of(skills: &Skills, open: bool, holding: bool) -> Self {
        let tune = tuning();
        Self {
            bank: skills.bank,
            points: skills.points,
            levels: skills.levels,
            big: skills.big,
            costs: [tune.mining_cost, tune.fortune_cost, tune.haste_cost],
            maxes: [tune.mining_tier, tune.fortune_tier, tune.haste_tier],
            open,
            holding,
        }
    }
}

/// One thing to draw, relative to the box's top left corner.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// x, y, w, h and argb.
    Fill([f32; 4], u32),
    /// The text, its x and y, rgb, and a shadow.
    Text(String, [f32; 2], u32, bool),
}

/// What a click at a point (in the box's own pixels) hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// The bar: hold it to bank experience.
    Bar,
    /// The "+" of the tree on this row.
    Spend(usize),
}

/// The box's size this frame.
pub fn size(v: &View) -> (f32, f32) {
    if v.open {
        (PANEL.0, PANEL_Y + PANEL.1)
    } else {
        BAR
    }
}

/// A row's effect text.
fn row_text(i: usize, v: &View) -> String {
    let tree = TREES[i];
    if tree.is_big() {
        let cost = v.costs[i - BIG_MINING];
        return if v.levels[i] == 0 {
            format!(
                "{}  {} point{}",
                tree.title(),
                cost,
                if cost == 1 { "" } else { "s" }
            )
        } else {
            let on = v.big[i - BIG_MINING] == 1;
            format!(
                "{}  {}/{}  {}",
                tree.title(),
                v.levels[i].min(v.maxes[i - BIG_MINING]),
                v.maxes[i - BIG_MINING],
                if on { "on" } else { "off" }
            )
        };
    }
    let level = v.levels[i].min(tree.max());
    if level >= tree.max() {
        format!("{}  {}/{}  max", tree.title(), level, tree.max())
    } else {
        format!(
            "{}  {}/{}  {}",
            tree.title(),
            level,
            tree.max(),
            tree.per_level(tuning())
        )
    }
}

/// The bar and, when open, the three tree rows under it.
pub fn shapes(v: &View) -> Vec<Shape> {
    let (w, h) = BAR;
    let mut out = Vec::with_capacity(20);
    out.push(Shape::Fill([0.0, 0.0, w, h], BACKGROUND));
    let filled = (w - 2.0) * (v.bank / XP_PER_POINT).clamp(0.0, 1.0);
    if filled > 0.0 {
        out.push(Shape::Fill(
            [1.0, 1.0, filled, h - 2.0],
            if v.holding { READY } else { FILL },
        ));
    }
    let label = format!("XP {:>3}%", (v.bank / XP_PER_POINT * 100.0).round() as u32);
    out.push(Shape::Text(label, [3.0, 2.0], INK, true));
    let points = format!("{} point{}", v.points, if v.points == 1 { "" } else { "s" });
    let width = points.len() as f32 * 6.0;
    out.push(Shape::Text(points, [w - width - 3.0, 2.0], INK, true));
    if !v.open {
        return out;
    }
    out.push(Shape::Fill([0.0, PANEL_Y, PANEL.0, PANEL.1], BACKGROUND));
    for i in 0..ROWS {
        let y = PANEL_Y + 1.0 + i as f32 * ROW;
        let ready = if TREES[i].is_big() {
            // Unlocked and on, or one point away from unlocking.
            v.levels[i] > 0 && v.big[i - BIG_MINING] == 1 || v.levels[i] == 0 && v.points > 0
        } else {
            v.points > 0 && v.levels[i] < TREES[i].max()
        };
        out.push(Shape::Text(
            row_text(i, v),
            [3.0, y + 4.0],
            if ready { READY } else { INK },
            true,
        ));
        out.push(Shape::Fill(
            [PLUS_X, y + 3.0, PLUS, PLUS],
            if ready { FILL } else { DIM },
        ));
        out.push(Shape::Text("+".to_owned(), [PLUS_X + 3.0, y + 4.0], INK, false));
    }
    out
}

/// What a click at `at`, local to the box, does: the bar takes the whole
/// first row, each tree its "+" box.
pub fn hit(v: &View, at: (f32, f32)) -> Option<Hit> {
    let (x, y) = at;
    if x < 0.0 || x >= PANEL.0 {
        return None;
    }
    if y >= 0.0 && y < BAR.1 {
        return Some(Hit::Bar);
    }
    if !v.open {
        return None;
    }
    let top = PANEL_Y + 1.0;
    if y < top || y >= top + ROWS as f32 * ROW {
        return None;
    }
    let row = ((y - top) / ROW) as usize;
    (x >= PLUS_X && x < PLUS_X + PLUS).then_some(Hit::Spend(row.min(ROWS - 1)))
}

/// Whether a click at `cursor` (GUI pixels) is the box's, and so must not
/// also swing at the world. Hit-testing the pointer, rather than remembering
/// the last press, means the order of the click and the key hook's own event
/// cannot matter.
pub fn blocks_attack(
    skills: &Skills,
    open: bool,
    holding: bool,
    origin: (f32, f32),
    cursor: (f32, f32),
) -> bool {
    let (x, y) = (cursor.0 - origin.0, cursor.1 - origin.1);
    let (w, h) = size(&View::of(skills, open, holding));
    // The whole box, not just the bar and the "+" boxes: anywhere on it is
    // the interface's, including a row's label while the panel is open.
    x >= 0.0 && y >= 0.0 && x < w && y < h
}

/// The `/skills` output: the bar, the points and every tree.
pub fn status(s: &Skills) -> String {
    let mut out = format!(
        "tkatt-skills: {} / {} xp banked, {} point(s) to spend\n",
        s.bank.round(),
        XP_PER_POINT,
        s.points
    );
    for tree in TREES {
        let i = tree.index();
        if tree.is_big() {
            let state = if s.levels[i] == 0 {
                format!("{} point(s)", tuning().cost(i))
            } else if s.big_on(i) {
                "on".to_owned()
            } else {
                "off".to_owned()
            };
            let max = tuning().max(i);
            out.push_str(&format!(
                "  {} {}/{} {}\n",
                tree.title(),
                s.big_rank(i).min(max),
                max,
                state
            ));
        } else {
            out.push_str(&format!(
                "  {} {}/{} {}\n",
                tree.title(),
                s.levels[i],
                tree.max(),
                tree.per_level(tuning())
            ));
        }
    }
    if s.fist_tier() > 0 {
        out.push_str(&format!(
            "  Fists mine at {} speed, like a {} pickaxe\n",
            s.fist_speed(),
            FIST_TIERS[s.fist_tier() as usize].to_lowercase()
        ));
    }
    out.trim_end().to_owned()
}

/// A deterministic roll in 0..1 for the block at `at`, so Fortune answers the
/// same way each time that block breaks instead of owning a random source.
pub fn fortune_roll(at: [i32; 3]) -> f32 {
    let mut h = 0x9E37_79B9_7F4A_7C15u64;
    for v in at {
        h ^= u64::from(v as u32).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = h.rotate_left(27).wrapping_mul(0x94D0_49BB_1331_11EB);
    }
    ((h >> 40) as u32) as f32 / (1u32 << 24) as f32
}

/// Where Fortune's extra item goes: the index of the stack to add one to, or
/// none. The first drop is the one, the way a block's own fortune multiplies
/// what it already gives; an empty list or a roll that misses adds nothing.
pub fn fortune_target(len: usize, chance: f32, roll: f32) -> Option<usize> {
    (len > 0 && chance > 0.0 && roll < chance.min(1.0)).then_some(0)
}

/// The testing commands' usage line, for an unknown or incomplete command.
pub const USAGE: &str = "tkatt-skills: /skills, /skills reset, /skills points <n>, /skills level <row> <n>, /skills tune [<row> <field> <value>] (melee, armour, bulwark, fist, mining, fortune, haste)";

/// What a `/skills` line asked for, after its first word.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    /// No argument: show the trees and toggle the panel.
    Status,
    /// Clear every tree; the points stay.
    Reset,
    /// Add points, to try a tree without banking 100 xp a point.
    Points(u32),
    /// Set one tree's level, clamped to its end, to try its effect at once.
    Level(Tree, u32),
    /// No argument: show the tuning.
    TuneShow,
    /// Change one of a big skill's numbers.
    Tune(Tree, Field, f32),
}

impl Op {
    /// Whether the command changes the stored state (`Status` and `TuneShow`
    /// do not).
    pub fn changes(self) -> bool {
        !matches!(self, Op::Status | Op::TuneShow)
    }
}

/// Reads the words after `skills`; an unknown or incomplete command is the
/// usage line.
pub fn op(words: &[&str]) -> Result<Op, String> {
    let bad = || USAGE.to_owned();
    match words {
        [] => Ok(Op::Status),
        ["reset"] => Ok(Op::Reset),
        ["points", n] => n.parse::<u32>().map(Op::Points).map_err(|_| bad()),
        ["level", tree, n] => {
            let tree = Tree::from_word(tree).ok_or_else(bad)?;
            let level = n.parse::<u32>().map_err(|_| bad())?;
            Ok(Op::Level(tree, level))
        }
        ["tune"] => Ok(Op::TuneShow),
        ["tune", tree, field, value] => {
            let tree = Tree::from_word(tree).ok_or_else(bad)?;
            let field = Field::from_word(field).ok_or_else(bad)?;
            let value = value.parse::<f32>().map_err(|_| bad())?;
            Ok(Op::Tune(tree, field, value))
        }
        _ => Err(bad()),
    }
}

impl Skills {
    /// Applies a `Reset`, `Points` or `Level` op; `Status` changes nothing.
    pub fn run(&mut self, op: Op) {
        match op {
            Op::Status | Op::TuneShow | Op::Tune(..) => {}
            Op::Reset => {
                self.levels = [0; ROWS];
                self.big = [0; BIG_ROWS];
            }
            Op::Points(n) => self.points = self.points.saturating_add(n),
            Op::Level(tree, n) => {
                let i = tree.index();
                let max = if tree.is_big() {
                    tuning().max(i)
                } else {
                    tree.max()
                };
                self.levels[i] = n.min(max);
                if tree.is_big() {
                    self.big[i - BIG_MINING] = if self.levels[i] == 0 { 0 } else { 1 };
                }
            }
        }
    }

    /// The reply to a command that changed the state.
    pub fn done(&self, op: Op) -> String {
        match op {
            Op::Reset => format!("tkatt-skills: every tree reset; {} point(s) kept.", self.points),
            Op::Points(n) => format!("tkatt-skills: {} point(s) added; {} to spend.", n, self.points),
            Op::Level(tree, _) => {
                let i = tree.index();
                let max = if tree.is_big() {
                    tuning().max(i)
                } else {
                    tree.max()
                };
                format!(
                    "tkatt-skills: {} {}/{}, {} point(s) to spend.",
                    tree.title(),
                    self.levels[i].min(max),
                    max,
                    self.points
                )
            }
            Op::Status | Op::TuneShow | Op::Tune(..) => status(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banks_experience_and_grants_a_point_at_the_threshold() {
        let mut s = Skills::default();
        assert_eq!(s.pump(2), 2);
        assert_eq!(s.bank, 2.0);
        // 100 xp: one point, the bar empty again.
        for _ in 0..49 {
            s.pump(2);
        }
        assert_eq!(s.bank, 0.0);
        assert_eq!(s.points, 1);
        assert_eq!(s.spent_xp, 100);
        // Nothing left to take: nothing banked.
        assert_eq!(s.pump(0), 0);
        assert_eq!(s.bank, 0.0);
    }

    #[test]
    fn a_tick_takes_only_the_rate_or_what_is_there() {
        let mut s = Skills::default();
        assert_eq!(s.pump(1), 1);
        assert_eq!(s.pump(50), XP_PER_TICK);
        assert_eq!(s.bank, 3.0);
    }

    #[test]
    fn spends_points_and_stops_at_the_end_of_a_tree() {
        let mut s = Skills {
            points: 2,
            ..Skills::default()
        };
        assert!(s.spend(Tree::Melee));
        assert_eq!(s.levels, [1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(s.points, 1);
        assert!(s.spend(Tree::Melee));
        s.points = 40;
        for _ in 0..20 {
            s.spend(Tree::Fist);
        }
        assert_eq!(s.fist_tier(), FIST_MAX);
        assert_eq!(s.fist_speed(), 8.0);
        assert_eq!(Skills::default().points, 0);
        assert!(!Skills::default().spend(Tree::Armor));
    }

    #[test]
    fn the_trees_scale_the_way_they_read() {
        let mut s = Skills::default();
        let tune = Tune::default();
        assert_eq!(s.melee_multiplier(tune), 1.0);
        assert_eq!(s.armor_points(tune), 0.0);
        assert_eq!(s.fist_speed(), 1.0);
        s.levels = [10, 10, 0, 4, 0, 0, 0];
        assert!((s.melee_multiplier(tune) - 1.8).abs() < 1e-6);
        assert!((s.armor_points(tune) - 5.0).abs() < 1e-6);
        assert!((s.incoming_damage(tune, 9.0) - 4.0).abs() < 1e-6);
        assert_eq!(s.fist_speed(), 8.0);
        assert!(s.fist_covers(3));
        let mut weak = Skills {
            levels: [0, 0, 0, 2, 0, 0, 0],
            ..Skills::default()
        };
        assert_eq!(weak.fist_speed(), 4.0);
        assert!(weak.fist_covers(2));
        assert!(!weak.fist_covers(3));
        weak.levels[Tree::Fist.index()] = 4;
        assert!(weak.fist_covers(4));
    }

    #[test]
    fn the_old_ids_files_are_carried_when_the_new_folder_has_none() {
        // Both files move on the first launch after the rename.
        let both = carry(
            Some("2|0.00|0|1|0|0|0|0|0|0|0|0|0|0".into()),
            Some("3|0.50|0.25|200|1|1|1|1|0.08|0.50|0.25|3|3|3".into()),
            None,
            None,
        );
        assert_eq!(both.len(), 2);
        assert_eq!(both[0].0, "state");
        assert_eq!(both[1].0, "tuning");
        // A file the new folder already has wins; nothing is overwritten.
        assert!(
            carry(Some("old".into()), Some("t".into()), Some("new".into()), Some("3|0.50".into()))
                .is_empty()
        );
        // One new file can be there while the other is still to carry.
        assert_eq!(
            carry(Some("old".into()), Some("t".into()), Some("new".into()), None),
            vec![("tuning", "t".to_owned())]
        );
        // One file can be carried without the other, and a blank carries nothing.
        assert_eq!(
            carry(None, Some("3|0.50".into()), None, None),
            vec![("tuning", "3|0.50".to_owned())]
        );
        assert!(carry(Some("   ".into()), None, None, None).is_empty());
        assert!(carry(None, None, None, None).is_empty());
    }

    #[test]
    fn a_stored_line_round_trips_and_bad_values_fall_back() {
        let s = Skills {
            bank: 42.5,
            points: 3,
            levels: [2, 1, 0, 4, 1, 1, 1],
            big: [1, 2, 1],
            spent_xp: 950,
        };
        assert_eq!(Skills::decode(&s.encode()), s);
        assert_eq!(Skills::decode(""), Skills::default());
        assert_eq!(Skills::decode("3|1|2|3|4|5|6"), Skills::default());
        // Out-of-range levels are cut to each tree's end, not rejected.
        let cut = Skills::decode("1|0|0|99|0|99|0");
        assert_eq!(cut.levels, [MAX_LEVEL, 0, 0, FIST_MAX, 0, 0, 0]);
        assert_eq!(cut.big, [0; BIG_ROWS]);
        assert_eq!(cut.bank, 0.0);
        // Big fields past the end of an old line are locked, not garbage.
        let old = Skills::decode("1|10|1|2|0|1|300");
        assert_eq!(old.levels, [2, 0, 0, 1, 0, 0, 0]);
        assert_eq!(old.big, [0; BIG_ROWS]);
    }

    #[test]
    fn bulwark_ranks_cut_a_tunable_share_of_what_armour_leaves() {
        let tune = Tune::default();
        let mut s = Skills {
            levels: [0, 10, 0, 0, 0, 0, 0],
            ..Skills::default()
        };
        // Ten armour levels at half a point each.
        assert!((s.armor_points(tune) - 5.0).abs() < 1e-6);
        // No ranks: the flat points still come off, never below zero.
        assert_eq!(s.ward_cut(tune), 0.0);
        assert!((s.incoming_damage(tune, 8.0) - 3.0).abs() < 1e-6);
        assert_eq!(s.incoming_damage(tune, 2.0), 0.0);
        // One rank cuts 25% of what is left, two half, three 75%.
        s.levels[Tree::Bulwark.index()] = 1;
        assert!((s.incoming_damage(tune, 8.0) - 2.25).abs() < 1e-6);
        s.levels[Tree::Bulwark.index()] = 2;
        assert!((s.incoming_damage(tune, 8.0) - 1.5).abs() < 1e-6);
        s.levels[Tree::Bulwark.index()] = 3;
        assert!((s.incoming_damage(tune, 8.0) - 0.75).abs() < 1e-6);
        // The cut a rank adds is tunable, and capped below the whole hit.
        let mut tuned = Tune::default();
        assert!(tuned.set(Tree::Bulwark, Field::Ward, 0.1).is_ok());
        assert!((s.ward_cut(tuned) - 0.3).abs() < 1e-6);
        assert!(tuned.set(Tree::Bulwark, Field::Ward, 1.0).is_ok());
        assert_eq!(s.ward_cut(tuned), WARD_CUT_MAX);
        assert!(s.incoming_damage(tuned, 8.0) > 0.0);
        // Without armour the ranks still cut the hit.
        let ward = Skills {
            levels: [0, 0, 2, 0, 0, 0, 0],
            ..Skills::default()
        };
        assert!((ward.incoming_damage(tune, 8.0) - 4.0).abs() < 1e-6);
        // A rank costs a point, and the row stops at three.
        let mut s = Skills {
            points: 4,
            ..Skills::default()
        };
        assert_eq!(s.press(Tree::Bulwark, false, tune), Press::Spent(2));
        assert_eq!(s.press(Tree::Bulwark, false, tune), Press::Spent(2));
        assert_eq!(s.press(Tree::Bulwark, false, tune), Press::Spent(2));
        assert_eq!(s.levels[Tree::Bulwark.index()], 3);
        assert_eq!(s.press(Tree::Bulwark, false, tune), Press::Maxed);
    }

    #[test]
    fn the_box_hits_the_bar_and_each_plus() {
        let closed = View::default();
        assert_eq!(hit(&closed, (90.0, 6.0)), Some(Hit::Bar));
        assert_eq!(hit(&closed, (90.0, 20.0)), None);
        assert_eq!(hit(&closed, (200.0, 6.0)), None);
        let open = View {
            open: true,
            ..View::default()
        };
        assert_eq!(hit(&open, (90.0, 6.0)), Some(Hit::Bar));
        for row in 0..ROWS {
            let y = PANEL_Y + 1.0 + row as f32 * ROW + 6.0;
            assert_eq!(hit(&open, (PLUS_X + 4.0, y)), Some(Hit::Spend(row)));
            // The row's label is not a button.
            assert_eq!(hit(&open, (20.0, y)), None);
        }
        assert_eq!(hit(&open, (20.0, PANEL_Y + 1.0 + ROWS as f32 * ROW + 2.0)), None);
        assert_eq!(size(&open), (PANEL.0, PANEL_Y + PANEL.1));
        assert_eq!(size(&closed), BAR);
    }

    #[test]
    fn the_box_draws_the_fill_points_and_rows() {
        let v = View {
            bank: 50.0,
            points: 2,
            levels: [3, 0, 0, 0, 0, 0, 0],
            big: [0; BIG_ROWS],
            costs: [1; BIG_ROWS],
            maxes: [BIG_MAX; BIG_ROWS],
            open: true,
            holding: true,
        };
        let out = shapes(&v);
        assert_eq!(out[0], Shape::Fill([0.0, 0.0, 182.0, 12.0], BACKGROUND));
        // Half the bar, lit while held.
        assert_eq!(out[1], Shape::Fill([1.0, 1.0, 90.0, 10.0], READY));
        assert!(out.contains(&Shape::Text("XP  50%".to_owned(), [3.0, 2.0], INK, true)));
        assert!(out.contains(&Shape::Text(
            "Melee damage  3/10  +8% damage".to_owned(),
            [3.0, PANEL_Y + 5.0],
            READY,
            true
        )));
        assert!(out.contains(&Shape::Text(
            "Fist mining  0/4  one pickaxe tier with your fists".to_owned(),
            [3.0, PANEL_Y + 1.0 + 3.0 * ROW + 4.0],
            READY,
            true
        )));
    }

    #[test]
    fn a_click_on_the_box_is_not_an_attack() {
        let skills = Skills::default();
        let origin = (10.0, 100.0);
        // On the bar: suppressed.
        assert!(blocks_attack(&skills, false, false, origin, (100.0, 106.0)));
        // Right of the bar, or under it while the panel is shut: an attack.
        assert!(!blocks_attack(&skills, false, false, origin, (300.0, 106.0)));
        assert!(!blocks_attack(&skills, false, false, origin, (100.0, 130.0)));
        // The panel's rows belong to the box once it is open.
        let row = (60.0, 130.0);
        assert!(blocks_attack(&skills, true, false, origin, row));
        assert!(!blocks_attack(&skills, false, false, origin, row));
    }

    #[test]
    fn the_status_lines_name_every_tree() {
        let s = Skills {
            bank: 25.0,
            points: 1,
            levels: [0, 0, 0, 2, 1, 0, 0],
            big: [1, 0, 0],
            spent_xp: 0,
        };
        let text = status(&s);
        assert!(text.starts_with("tkatt-skills: 25 / 100 xp banked, 1 point(s) to spend"));
        assert!(text.contains("Melee damage 0/10"));
        assert!(text.contains("Armour 0/10"));
        assert!(text.contains("Armour 0/10 +0.5 armour"));
        assert!(text.contains("Bulwark 0/3 25% cut"));
        assert!(text.contains("Fist mining 2/4"));
        assert!(text.contains("like a stone pickaxe"));
        assert!(text.contains("Mining speed 1/3 on"));
        assert!(text.contains("Fortune 0/3 1 point(s)"));
    }

    #[test]
    fn a_big_skill_is_unlocked_with_one_point_then_toggled() {
        let tune = Tune::default();
        let mut s = Skills {
            points: 2,
            ..Skills::default()
        };
        assert_eq!(s.press(Tree::Mining, false, tune), Press::Unlocked(BIG_MINING));
        assert_eq!(s.points, 1);
        assert!((s.mining_multiplier(tune) - 1.5).abs() < 1e-6);
        // Shift buys the next rank; the tuning sets the price.
        assert_eq!(s.press(Tree::Mining, true, tune), Press::Ranked(BIG_MINING));
        assert_eq!(s.big_rank(BIG_MINING), 2);
        assert_eq!(s.points, 0);
        assert!((s.mining_multiplier(tune) - 2.0).abs() < 1e-6);
        // Without a point a rank is refused and changes nothing.
        assert_eq!(s.press(Tree::Mining, true, tune), Press::NoPoints);
        assert_eq!(s.big_rank(BIG_MINING), 2);
        // A plain click toggles an unlocked skill.
        assert_eq!(s.press(Tree::Mining, false, tune), Press::Off(BIG_MINING));
        assert_eq!(s.mining_multiplier(tune), 1.0);
        assert_eq!(s.press(Tree::Mining, false, tune), Press::On(BIG_MINING));
        // An unaffordable unlock changes nothing.
        let mut broke = Skills::default();
        assert_eq!(broke.press(Tree::Fortune, false, tune), Press::NoPoints);
        assert_eq!(broke.levels, [0; ROWS]);
        assert_eq!(broke.big, [0; BIG_ROWS]);
        // A levelled tree still spends, and stops at its end.
        let mut s = Skills {
            points: 1,
            ..Skills::default()
        };
        assert_eq!(s.press(Tree::Melee, false, tune), Press::Spent(0));
        assert_eq!(s.levels[0], 1);
        s.points = 3;
        s.levels[0] = MAX_LEVEL;
        assert_eq!(s.press(Tree::Melee, false, tune), Press::Maxed);
        // A tuned tier caps buying: at the last rank Shift toggles instead.
        let mut capped = Tune::default();
        assert!(capped.set(Tree::Fortune, Field::Tier, 1.0).is_ok());
        let mut s = Skills {
            points: 5,
            ..Skills::default()
        };
        assert_eq!(
            s.press(Tree::Fortune, false, capped),
            Press::Unlocked(BIG_FORTUNE)
        );
        assert_eq!(s.press(Tree::Fortune, true, capped), Press::Off(BIG_FORTUNE));
        assert_eq!(s.big_rank(BIG_FORTUNE), 1);
    }

    #[test]
    fn ranks_and_tuning_scale_the_big_effects() {
        let mut tune = Tune::default();
        let mut s = Skills {
            levels: [0, 0, 0, 0, 3, 2, 1],
            big: [1, 1, 1],
            ..Skills::default()
        };
        assert!((s.mining_multiplier(tune) - 2.5).abs() < 1e-6);
        assert!((s.fortune_chance(tune) - 0.5).abs() < 1e-6);
        assert_eq!(s.haste_ticks(tune), 200);
        assert_eq!(s.haste_amp(tune), 1);
        // Tuning moves every rank with it.
        assert!(tune.set(Tree::Mining, Field::Step, 1.0).is_ok());
        assert!(tune.set(Tree::Fortune, Field::Chance, 0.4).is_ok());
        assert!(tune.set(Tree::Haste, Field::Amp, 2.0).is_ok());
        assert!((s.mining_multiplier(tune) - 4.0).abs() < 1e-6);
        assert!((s.fortune_chance(tune) - 0.8).abs() < 1e-6);
        assert_eq!(s.haste_amp(tune), 2);
        // Chance never passes one, and cost is read per row.
        s.levels[BIG_FORTUNE] = 3;
        assert_eq!(s.fortune_chance(tune), 1.0);
        assert!(tune.set(Tree::Fortune, Field::Cost, 3.0).is_ok());
        assert_eq!(tune.cost(BIG_FORTUNE), 3);
        // A field that does not belong to a tree is refused.
        assert!(tune.set(Tree::Mining, Field::Ticks, 100.0).is_err());
        assert!(tune.set(Tree::Melee, Field::Cost, 1.0).is_err());
        // The line round trips, and bad fields keep or clamp their defaults.
        assert_eq!(Tune::decode(&tune.encode()), tune);
        assert_eq!(Tune::decode("nonsense"), Tune::default());
        let clamped = Tune::decode("1|-1|9|1|99|0|0|0");
        assert_eq!(clamped.mining_step, 0.0);
        assert_eq!(clamped.fortune_chance, 1.0);
        assert_eq!(clamped.haste_ticks, 20);
        assert_eq!(clamped.haste_amp, 4);
        assert_eq!(clamped.mining_cost, 1);
        // The melee and armour percentages and the tiers are tunable too.
        assert!(tune.set(Tree::Melee, Field::Damage, 0.1).is_ok());
        assert!(tune.set(Tree::Armor, Field::Armor, 0.05).is_ok());
        assert!(tune.set(Tree::Bulwark, Field::Ward, 0.2).is_ok());
        assert_eq!(tune.ward_step, 0.2);
        let trees = Skills {
            levels: [10, 10, 0, 0, 0, 0, 0],
            ..Skills::default()
        };
        assert!((trees.melee_multiplier(tune) - 2.0).abs() < 1e-6);
        assert!((trees.armor_points(tune) - 0.5).abs() < 1e-6);
        assert_eq!(trees.ward_cut(tune), 0.0);
        assert!(tune.set(Tree::Fortune, Field::Tier, 5.0).is_ok());
        assert_eq!(tune.max(BIG_FORTUNE), 5);
        // An old line without them keeps the defaults.
        let old = Tune::decode("1|0.50|0.25|200|1|1|1|1");
        assert_eq!(old.melee_step, MELEE_PER_LEVEL);
        assert_eq!(old.armor_points, ARMOR_POINTS_PER_LEVEL);
        // A version-2 line has no cut field, so later fields stay aligned.
        let v2 = Tune::decode("2|0.50|0.25|200|1|1|1|1|0.08|1.50|3|4|5");
        assert_eq!(v2.ward_step, WARD_CUT_PER_RANK);
        assert_eq!(v2.armor_points, 1.5);
        assert_eq!(v2.mining_tier, 3);
        assert_eq!(v2.fortune_tier, 4);
        assert_eq!(v2.haste_tier, 5);
        assert_eq!(old.mining_tier, BIG_MAX);
        assert!(tune.show().contains("mining +100%/rank"));
    }

    #[test]
    fn the_big_skills_read_their_own_state() {
        let tune = Tune::default();
        let mut s = Skills {
            levels: [0, 0, 0, 0, 1, 1, 0],
            big: [1, 2, 1],
            ..Skills::default()
        };
        assert!(s.big_on(BIG_MINING));
        assert!(!s.big_on(BIG_FORTUNE));
        assert!(!s.haste_on());
        assert!(!s.fortune_on());
        s.levels[BIG_HASTE] = 1;
        s.big[BIG_HASTE - BIG_MINING] = 1;
        assert!(s.haste_on());
        assert_eq!(s.haste_amp(tune), 1);
        // A locked big skill is off whatever its stored flag says.
        s.levels[BIG_FORTUNE] = 0;
        s.big[BIG_FORTUNE - BIG_MINING] = 1;
        assert!(!s.fortune_on());
        assert_eq!(s.fortune_chance(tune), 0.0);
        let a = fortune_roll([1, 2, 3]);
        assert!((0.0..1.0).contains(&a));
        assert_eq!(a, fortune_roll([1, 2, 3]));
        assert_ne!(a, fortune_roll([3, 2, 1]));
    }

    #[test]
    fn the_mining_factor_combines_the_big_skill_and_the_fist_tier() {
        let tune = Tune::default();
        let mut s = Skills::default();
        let stone = MiningFacts {
            bare: true,
            pickaxe: true,
            base_speed: 1.0,
            needs: 2,
            covers: false,
        };
        // Nothing bought: no change, even on a bare hand.
        assert_eq!(s.mining_factor(tune, stone), 1.0);
        // The big skill alone speeds any block, tool or not.
        s.levels[BIG_MINING] = 2;
        s.big[0] = 1;
        let held = MiningFacts {
            bare: false,
            pickaxe: true,
            base_speed: 8.0,
            needs: 4,
            covers: true,
        };
        assert!((s.mining_factor(tune, held) - 2.0).abs() < 1e-6);
        // A stone fist mines at 4.0 where the hand had 1.0 and satisfies the
        // stone tag, so the divisor moves from 100 to 30 too.
        s.levels[Tree::Fist.index()] = 2;
        let expected = 2.0 * 4.0 * (100.0 / 30.0);
        assert!((s.mining_factor(tune, stone) - expected).abs() < 1e-5);
        // An iron block: the same fist does not cover it, so no divisor move.
        let iron = MiningFacts {
            needs: 3,
            ..stone
        };
        assert!((s.mining_factor(tune, iron) - 2.0 * 4.0).abs() < 1e-5);
        // A block the hand cannot mine is untouched by the fist.
        let impossible = MiningFacts {
            base_speed: 0.0,
            ..stone
        };
        assert!((s.mining_factor(tune, impossible) - 2.0).abs() < 1e-6);
        // A diamond fist covers diamond.
        s.levels[Tree::Fist.index()] = FIST_MAX;
        let diamond = MiningFacts {
            needs: 4,
            ..stone
        };
        assert!((s.mining_factor(tune, diamond) - 2.0 * 8.0 * (100.0 / 30.0)).abs() < 1e-4);
    }

    #[test]
    fn the_fortune_roll_names_the_drop_to_add_to() {
        assert_eq!(fortune_target(0, 0.5, 0.1), None);
        assert_eq!(fortune_target(2, 0.0, 0.0), None);
        assert_eq!(fortune_target(2, 0.5, 0.49), Some(0));
        assert_eq!(fortune_target(2, 0.5, 0.5), None);
        assert_eq!(fortune_target(2, 2.0, 0.99), Some(0));
        assert_eq!(fortune_target(2, 1.0, 0.999), Some(0));
    }

    #[test]
    fn the_fist_tier_decides_a_bare_hands_drops() {
        let mut s = Skills::default();
        // No tier: the game's own answer stands.
        assert!(!s.fist_drop(false, 0));
        assert!(s.fist_drop(true, 4));
        // A stone fist keeps stone and dirt (needs 0), not iron or diamond.
        s.levels[Tree::Fist.index()] = 2;
        assert!(s.fist_drop(false, 0));
        assert!(s.fist_drop(false, 2));
        assert!(!s.fist_drop(false, 3));
        assert!(s.fist_drop(true, 3));
        // A diamond fist covers every tag.
        s.levels[Tree::Fist.index()] = FIST_MAX;
        assert!(s.fist_drop(false, 4));
    }

    #[test]
    fn the_test_commands_parse_and_name_bad_input() {
        assert_eq!(op(&[]), Ok(Op::Status));
        assert_eq!(op(&["reset"]), Ok(Op::Reset));
        assert_eq!(op(&["points", "7"]), Ok(Op::Points(7)));
        assert_eq!(op(&["points"]), Err(USAGE.to_owned()));
        assert_eq!(op(&["points", "lots"]), Err(USAGE.to_owned()));
        assert_eq!(op(&["level", "armour", "4"]), Ok(Op::Level(Tree::Armor, 4)));
        assert_eq!(op(&["level", "armor", "4"]), Ok(Op::Level(Tree::Armor, 4)));
        assert_eq!(op(&["level", "fist", "9"]), Ok(Op::Level(Tree::Fist, 9)));
        assert_eq!(op(&["level", "speed", "1"]), Err(USAGE.to_owned()));
        assert_eq!(op(&["level", "melee"]), Err(USAGE.to_owned()));
        assert_eq!(op(&["tune"]), Ok(Op::TuneShow));
        assert_eq!(
            op(&["tune", "mining", "step", "0.75"]),
            Ok(Op::Tune(Tree::Mining, Field::Step, 0.75))
        );
        assert_eq!(op(&["tune", "mining"]), Err(USAGE.to_owned()));
        assert_eq!(
            op(&["tune", "mining", "nonsense", "1"]),
            Err(USAGE.to_owned())
        );
        assert_eq!(op(&["nonsense"]), Err(USAGE.to_owned()));
    }

    #[test]
    fn a_test_command_sets_points_and_levels_within_bounds() {
        assert!(!Op::Status.changes());
        assert!(!Op::TuneShow.changes());
        assert!(Op::Tune(Tree::Mining, Field::Step, 0.5).changes());
        let mut s = Skills {
            points: 2,
            ..Skills::default()
        };
        s.run(Op::Points(3));
        assert_eq!(s.points, 5);
        assert_eq!(s.done(Op::Points(3)), "tkatt-skills: 3 point(s) added; 5 to spend.");
        s.run(Op::Level(Tree::Armor, 99));
        assert_eq!(s.levels[1], MAX_LEVEL);
        s.run(Op::Level(Tree::Fist, 3));
        assert_eq!(s.fist_speed(), 6.0);
        assert_eq!(
            s.done(Op::Level(Tree::Fist, 3)),
            "tkatt-skills: Fist mining 3/4, 5 point(s) to spend."
        );
        s.run(Op::Reset);
        assert_eq!(s.levels, [0; ROWS]);
        assert_eq!(s.points, 5);
        // A status command is not a change and keeps the state as it is.
        s.run(Op::Status);
        assert_eq!(s.levels, [0; ROWS]);
    }
}
