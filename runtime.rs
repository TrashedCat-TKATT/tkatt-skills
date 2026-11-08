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
pub const VERSION: &str = "0.2.2";
pub const DESCRIPTION: &str = "Bank experience into a bar for skill points, then spend them on melee damage, armour, mining with your fists, or big skills like mining speed, Fortune and Haste. A framework for skill trees.";

/// One skill point per this much banked experience.
pub const XP_PER_POINT: f32 = 100.0;
/// Experience withdrawn from the player each tick the bar is held.
pub const XP_PER_TICK: u32 = 2;
/// Melee damage added per level: +8%, ten levels.
pub const MELEE_PER_LEVEL: f32 = 0.08;
/// The share of an incoming hit the armour tree cuts per level: 0.025 is
/// 2.5%, so ten levels cut a quarter of the hit.
pub const ARMOR_CUT_PER_LEVEL: f32 = 0.025;
/// The share of the remaining damage one Bulwark rank cuts: 0.10 is 10%.
pub const WARD_CUT_PER_RANK: f32 = 0.10;
/// The most the Bulwark ranks can cut, however far the tuning is pushed.
pub const WARD_CUT_MAX: f32 = 0.30;
/// The most the armour tree and the Bulwark ranks can cut together: half the
/// hit, before vanilla armour takes its own share of what is left.
pub const INCOMING_CUT_MAX: f32 = 0.5;
/// A landed hit never comes out below this, so the game's hurt step — and with
/// it knockback, the damage tint and an arrow's hit — always runs.
pub const INCOMING_FLOOR: f32 = 1.0;
/// Bulwark ranks: each one adds another cut.
pub const BULWARK_MAX: u32 = 3;
/// Every tree but the fist one stops here.
pub const MAX_LEVEL: u32 = 10;
/// Fist tiers: none, wood, stone, iron, diamond, netherite.
pub const FIST_MAX: u32 = 5;
/// The pickaxe speed a fist of each tier mines at (vanilla's materials).
pub const FIST_SPEEDS: [f32; 6] = [1.0, 2.0, 4.0, 6.0, 8.0, 9.0];
/// The tiers' names, for the panel and the command.
pub const FIST_TIERS: [&str; 6] = ["Bare", "Wooden", "Stone", "Iron", "Diamond", "Netherite"];

/// The colours the box draws with, in the HUD's alpha-red-green-blue order.
pub const BACKGROUND: u32 = 0x7800_0000;
pub const FILL: u32 = 0xFF2E_7D32;
pub const READY: u32 = 0xFF55_FF55;
pub const INK: u32 = 0xFFFF_FFFF;
pub const DIM: u32 = 0xFFA0_A0A0;

/// The bar, and the panel under it, in GUI pixels.
pub const BAR: (f32, f32) = (182.0, 12.0);
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
    /// The share of a hit the armour tree cuts per level: 0.025 is 2.5%.
    pub armor_cut: f32,
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
            armor_cut: ARMOR_CUT_PER_LEVEL,
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
            "4|{:.3}|{:.2}|{}|{}|{}|{}|{}|{:.2}|{:.3}|{:.2}|{}|{}|{}",
            self.mining_step,
            self.fortune_chance,
            self.haste_ticks,
            self.haste_amp,
            self.mining_cost,
            self.fortune_cost,
            self.haste_cost,
            self.melee_step,
            self.armor_cut,
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
            Some("4") => 4,
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
        // Versions 1 to 3 stored flat armour points here; version 4 stores the
        // share of a hit the tree cuts. An older line takes the new default
        // rather than reading half a damage point as half a hit.
        let armor_slot = next(ARMOR_CUT_PER_LEVEL, 0.0, 1.0);
        if version >= 4 {
            out.armor_cut = armor_slot;
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
            (Tree::Armor, Field::Armor) => self.armor_cut = value.clamp(0.0, 1.0),
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
            "tkatt-skills tuning: melee +{:.0}%/level, armour -{:.1}%/level, bulwark {:.0}%/rank; mining +{:.0}%/rank, fortune {:.0}%/rank, haste {} ticks Haste {}; costs {} / {} / {}; ranks {} / {} / {}",
            self.melee_step * 100.0,
            self.armor_cut * 100.0,
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
    /// The share of a hit the tree cuts per level.
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
                        "share" | "points" | "armor" | "armour" => Some(Field::Armor),
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
            Field::Armor => "share",
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
            Field::Armor => "share is 0..1 a level: /skills tune armour share 0.025",
            Field::Ward => "cut is 0..1 a rank: /skills tune bulwark cut 0.25",
            Field::Tier => "tier is 1..10 ranks: /skills tune fortune tier 3",
        }
    }
}

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
            // A fist tier is 0..=5: bare, wood, stone, iron, diamond, netherite.
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
            Tree::Armor => format!("-{:.1}% damage", tune.armor_cut * 100.0),
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

    /// The share of a hit the armour tree cuts at this tuning.
    pub fn armor_cut(&self, tune: Tune) -> f32 {
        (tune.armor_cut * self.levels[Tree::Armor.index()] as f32).clamp(0.0, 1.0)
    }

    /// How much of the remaining damage the Bulwark ranks cut, capped so a
    /// tuned step can never make the player untouchable.
    pub fn ward_cut(&self, tune: Tune) -> f32 {
        (tune.ward_step * self.levels[Tree::Bulwark.index()] as f32).clamp(0.0, WARD_CUT_MAX)
    }

    /// Incoming damage: the armour tree and the Bulwark ranks cut their shares,
    /// together never more than half the hit, and a landed hit never comes out
    /// below one point — zero damage would skip the game's hurt step, and with
    /// it the knockback, the damage tint and the arrow's hit.
    pub fn incoming_damage(&self, tune: Tune, value: f32) -> f32 {
        if value <= 0.0 {
            return value;
        }
        let cut = (self.armor_cut(tune) + self.ward_cut(tune)).clamp(0.0, INCOMING_CUT_MAX);
        (value * (1.0 - cut)).max(value.min(INCOMING_FLOOR))
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
    /// The screen's branch, its selected row and whether Bank is held.
    static SCREEN: std::cell::Cell<ScreenState> = const {
        std::cell::Cell::new(ScreenState {
            branch: Branch::Combat,
            selected: Some(MELEE_ROW),
            banking: false,
        })
    };
    /// What the screen's `&self` hooks leave for the tick to say or play.
    static NOTES: std::cell::RefCell<Vec<Note>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Whether the emblem made it into the atlas this session.
    static ICON: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// The player's skills and the tuning are shared by every thread rather than
// kept per thread: single player breaks blocks on the game's server thread
// while the screen is drawn and clicked on the window's, so a copy per thread
// left the effect hooks reading a bare state and a point spent in the screen
// did nothing until the next launch. These are the same on the window, its
// server thread and a LAN host, like the mod's settings folder is.
static STATE: std::sync::Mutex<Option<Skills>> = std::sync::Mutex::new(None);
static TUNE: std::sync::Mutex<Option<Tune>> = std::sync::Mutex::new(None);
/// The mod is on in Options > Mods (on until a hook first sees it off).
static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
/// The old id's files were looked at this session.
static CARRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The stored skills, or None before the first load. A poisoned lock still
/// holds the last state written, so it is taken rather than propagated.
fn state_slot() -> std::sync::MutexGuard<'static, Option<Skills>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

fn tune_slot() -> std::sync::MutexGuard<'static, Option<Tune>> {
    TUNE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether the mod is on in Options > Mods, as the last hook that had a
/// `Game` saw it. The effect hooks run where no `Game` is in reach, so they
/// read this instead.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// The player's skills and the tuning, from their stored files the first time.
/// `get` reads one of the mod's data files by key (`state`, `tuning`).
pub fn load_once(get: impl Fn(&str) -> Option<String>) -> Skills {
    let have_tune = tune_slot().is_some();
    if !have_tune {
        store_tune(Tune::decode(&get("tuning").unwrap_or_default()));
    }
    if let Some(s) = state_slot().clone() {
        return s;
    }
    let s = Skills::decode(&get("state").unwrap_or_default());
    store(s.clone());
    s
}

/// The skills as they are now (`Skills::default` before the first load).
pub fn current() -> Skills {
    state_slot().clone().unwrap_or_default()
}

/// Keeps the skills for this session and for every thread that reads them; the
/// caller writes the file.
pub fn store(s: Skills) {
    *state_slot() = Some(s);
}

/// The tuning as it is now (`Tune::default` before the first load).
pub fn tuning() -> Tune {
    (*tune_slot()).unwrap_or_default()
}

/// Keeps the tuning for this session and for every thread that reads it; the
/// caller writes the file.
pub fn store_tune(t: Tune) {
    *tune_slot() = Some(t);
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
    CARRIED.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_carried() {
    CARRIED.store(true, std::sync::atomic::Ordering::Relaxed);
}

// The in-world bar and the skill screen's layout, state and click rules.
//
// The bar is display only: it shows the bank and the points, and nothing on it
// takes a click. Everything the player does happens in the screen, where the
// pointer is free: pick a branch, pick a node, then use the footer button (or
// click the same node again) to spend, unlock, rank or toggle.

/// What the in-world bar shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BarView {
    pub bank: f32,
    pub points: u32,
}

/// One thing the bar draws, relative to its top left corner.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// x, y, w, h and argb.
    Fill([f32; 4], u32),
    /// The text, its x and y, rgb, and a shadow.
    Text(String, [f32; 2], u32, bool),
}

/// The bar in its corner: the bank's fill, `XP n%`, the points waiting.
pub fn bar(v: &BarView) -> Vec<Shape> {
    let (w, h) = BAR;
    let mut out = Vec::with_capacity(4);
    out.push(Shape::Fill([0.0, 0.0, w, h], BACKGROUND));
    let filled = (w - 2.0) * (v.bank / XP_PER_POINT).clamp(0.0, 1.0);
    if filled > 0.0 {
        out.push(Shape::Fill([1.0, 1.0, filled, h - 2.0], FILL));
    }
    let label = format!("XP {:>3}%", (v.bank / XP_PER_POINT * 100.0).round() as u32);
    out.push(Shape::Text(label, [3.0, 2.0], INK, true));
    let points = format!(
        "{} point{}",
        v.points,
        if v.points == 1 { "" } else { "s" }
    );
    let width = points.len() as f32 * 6.0;
    out.push(Shape::Text(
        points,
        [w - width - 3.0, 2.0],
        if v.points > 0 { READY } else { INK },
        true,
    ));
    out
}

/// Which half of the tree the screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    Combat,
    Gathering,
}

/// The rows, by index in `TREES`.
pub const MELEE_ROW: usize = 0;
pub const ARMOUR_ROW: usize = 1;
pub const BULWARK_ROW: usize = 2;
pub const FIST_ROW: usize = 3;

/// The combat branch's rows, in the order the screen lists them.
pub const COMBAT_ROWS: [usize; 3] = [MELEE_ROW, ARMOUR_ROW, BULWARK_ROW];
/// The gathering branch's rows: the fist tier, then the three big skills.
pub const GATHERING_ROWS: [usize; 4] = [FIST_ROW, BIG_MINING, BIG_MINING + 1, BIG_MINING + 2];

impl Branch {
    pub const ALL: [Branch; 2] = [Branch::Combat, Branch::Gathering];

    pub fn title(self) -> &'static str {
        match self {
            Branch::Combat => "Combat",
            Branch::Gathering => "Gathering",
        }
    }

    /// The tree rows of this branch, in the order the screen lists them.
    pub fn rows(self) -> &'static [usize] {
        match self {
            Branch::Combat => &COMBAT_ROWS,
            Branch::Gathering => &GATHERING_ROWS,
        }
    }
}

/// What the screen remembers between frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenState {
    pub branch: Branch,
    /// The row selected in that branch.
    pub selected: Option<usize>,
    /// The Bank button is held down.
    pub banking: bool,
}

impl Default for ScreenState {
    fn default() -> Self {
        Self {
            branch: Branch::Combat,
            selected: Some(MELEE_ROW),
            banking: false,
        }
    }
}

/// The screen's own state.
pub fn screen() -> ScreenState {
    SCREEN.with(|c| c.get())
}

pub fn set_screen(s: ScreenState) {
    SCREEN.with(|c| c.set(s));
}

/// The state a freshly opened screen starts in: combat, its first node.
pub fn open_screen() {
    set_screen(ScreenState::default());
}

/// Stops banking, on close.
pub fn close_screen() {
    let mut s = screen();
    s.banking = false;
    set_screen(s);
}

/// Something a `&self` hook wants said, played or done, kept for the tick.
#[derive(Clone, Debug, PartialEq)]
pub enum Note {
    /// A line in the action bar.
    Bar(String),
    /// A sound at the player: its name, volume and pitch.
    Sound(&'static str, f32, f32),
    /// The screen is done: close it.
    Close,
}

pub fn note(n: Note) {
    NOTES.with(|q| q.borrow_mut().push(n));
}

/// Takes what the screen's hooks left for the tick to say or do.
pub fn take_notes() -> Vec<Note> {
    NOTES.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Whether the emblem uploaded into the atlas this session.
pub fn icon_ready() -> bool {
    ICON.with(|c| c.get())
}

pub fn set_icon_ready(ok: bool) {
    ICON.with(|c| c.set(ok));
}

/// The button press sound.
pub const CLICK: &str = "minecraft:ui.button.click";

/// The screen, in GUI pixels. 300x200 fits vanilla's 320x240 smallest GUI.
pub const SCREEN_W: f32 = 300.0;
pub const SCREEN_H: f32 = 200.0;
const SCREEN_BG: u32 = 0xF0_14_14_14;
const SCREEN_EDGE: u32 = 0xFF_00_00_00;
const SCREEN_INNER: u32 = 0xFF_36_36_36;
const ROW_HOVER: u32 = 0xFF_26_26_26;
const ROW_SELECTED: u32 = 0xFF_33_33_33;
const NODE_BOX: u32 = 0xFF_26_26_26;
const NODE_INNER: u32 = 0xFF_1B_1B1B;
const ACCENT: u32 = 0xFF_FF_FF_55;
const GOOD: u32 = 0xFF_7C_FC_7C;

/// The two branch tabs' boxes.
pub const TAB_RECTS: [[f32; 4]; 2] = [[10.0, 46.0, 78.0, 18.0], [10.0, 68.0, 78.0, 18.0]];
pub const BANK_RECT: [f32; 4] = [212.0, 18.0, 78.0, 16.0];
pub const DONE_RECT: [f32; 4] = [10.0, 176.0, 60.0, 18.0];
pub const ACTION_RECT: [f32; 4] = [174.0, 176.0, 116.0, 18.0];
pub const TOGGLE_RECT: [f32; 4] = [106.0, 176.0, 64.0, 18.0];
pub const NODE_TOP: f32 = 46.0;
pub const NODE_PITCH: f32 = 28.0;

/// Where the panel's top left corner is, for a window `rect`.
pub fn panel_origin(rect: [f32; 4]) -> [f32; 2] {
    [
        ((rect[2] - SCREEN_W) / 2.0).floor(),
        ((rect[3] - SCREEN_H) / 2.0).floor(),
    ]
}

/// A row's clickable strip.
pub fn row_rect(i: usize) -> [f32; 4] {
    [
        90.0,
        NODE_TOP + i as f32 * NODE_PITCH,
        SCREEN_W - 90.0 - 10.0,
        NODE_PITCH,
    ]
}

/// Where row `i`'s icon, label and effect are drawn, panel-local.
pub fn icon_at(i: usize) -> [f32; 2] {
    [96.0, NODE_TOP + 1.0 + i as f32 * NODE_PITCH]
}

pub fn label_at(i: usize) -> [f32; 2] {
    [126.0, NODE_TOP + 2.0 + i as f32 * NODE_PITCH]
}

pub fn effect_at(i: usize) -> [f32; 2] {
    [126.0, NODE_TOP + 13.0 + i as f32 * NODE_PITCH]
}

/// What the screen draws this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenView {
    pub bank: f32,
    pub points: u32,
    pub levels: [u32; ROWS],
    pub big: [u8; BIG_ROWS],
    pub costs: [u32; BIG_ROWS],
    pub maxes: [u32; BIG_ROWS],
    pub tune: Tune,
    pub state: ScreenState,
    /// The pointer, panel-local, for hover highlights.
    pub mouse: [f32; 2],
}

impl ScreenView {
    pub fn of(skills: &Skills, state: ScreenState, mouse: [f32; 2]) -> Self {
        let tune = tuning();
        Self {
            bank: skills.bank,
            points: skills.points,
            levels: skills.levels,
            big: skills.big,
            costs: [tune.mining_cost, tune.fortune_cost, tune.haste_cost],
            maxes: [tune.mining_tier, tune.fortune_tier, tune.haste_tier],
            tune,
            state,
            mouse,
        }
    }

    /// A big skill's rank, clamped to its tuned end; 0 for a levelled tree.
    pub fn rank(&self, row: usize) -> u32 {
        if row >= BIG_MINING {
            self.levels[row].min(self.maxes[row - BIG_MINING])
        } else {
            0
        }
    }

    /// Whether the big skill on `row` is switched on.
    pub fn on(&self, row: usize) -> bool {
        row >= BIG_MINING && self.rank(row) > 0 && self.big[row - BIG_MINING] == 1
    }

    /// What one level or rank of `row` costs.
    pub fn cost(&self, row: usize) -> u32 {
        if row >= BIG_MINING {
            self.costs[row - BIG_MINING]
        } else {
            1
        }
    }
}

/// One thing the screen draws, panel-local.
#[derive(Clone, Debug, PartialEq)]
pub enum ScreenShape {
    Fill([f32; 4], u32),
    Text(String, [f32; 2], u32, bool),
    /// Text whose right edge is at x.
    TextRight(String, f32, f32, u32, bool),
    /// Text centred in the panel at y.
    TextCenter(String, f32, u32, bool),
    Button {
        label: String,
        rect: [f32; 4],
        enabled: bool,
        hovered: bool,
    },
    /// A vanilla item icon at a point, `size` pixels square.
    Icon(&'static str, [f32; 2], f32),
    /// The mod's own emblem, from the atlas slot `horizon/modimage`.
    Emblem([f32; 2], f32),
}

/// What a point in the panel hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenHit {
    Tab(usize),
    Node(usize),
    Bank,
    Done,
    /// The one footer button: level up, unlock, or turn a big skill on/off.
    Action,
    /// The big skill's turn-on/off button.
    Toggle,
    /// The big skill's rank-up button.
    Rank,
}

/// What a node's main action is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Level,
    Unlock,
    Toggle,
    Rank,
}

fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0]
        && at[1] >= rect[1]
        && at[0] < rect[0] + rect[2]
        && at[1] < rect[1] + rect[3]
}

/// What a click at `at` (panel-local) hits, if anything.
pub fn hit(v: &ScreenView, at: [f32; 2]) -> Option<ScreenHit> {
    if !inside([0.0, 0.0, SCREEN_W, SCREEN_H], at) {
        return None;
    }
    for (i, rect) in TAB_RECTS.iter().enumerate() {
        if inside(*rect, at) {
            return Some(ScreenHit::Tab(i));
        }
    }
    for (i, row) in v.state.branch.rows().iter().enumerate() {
        if inside(row_rect(i), at) {
            return Some(ScreenHit::Node(*row));
        }
    }
    if inside(BANK_RECT, at) {
        return Some(ScreenHit::Bank);
    }
    if inside(DONE_RECT, at) {
        return Some(ScreenHit::Done);
    }
    let rank_shown = v
        .state
        .selected
        .is_some_and(|row| TREES[row].is_big() && v.rank(row) > 0);
    if rank_shown {
        if inside(TOGGLE_RECT, at) {
            return Some(ScreenHit::Toggle);
        }
        if inside(ACTION_RECT, at) {
            return Some(ScreenHit::Rank);
        }
    } else if v.state.selected.is_some() && inside(ACTION_RECT, at) {
        return Some(ScreenHit::Action);
    }
    None
}

/// The selected node's main action, or None when it is finished.
pub fn primary(v: &ScreenView, row: usize) -> Option<Action> {
    if TREES[row].is_big() {
        if v.rank(row) == 0 {
            Some(Action::Unlock)
        } else {
            Some(Action::Toggle)
        }
    } else if v.levels[row] < TREES[row].max() {
        Some(Action::Level)
    } else {
        None
    }
}

/// Whether the points are there for `row`'s next level or rank.
pub fn affordable(v: &ScreenView, row: usize) -> bool {
    v.points >= v.cost(row)
}

/// A row's progress, as the row and the tooltip show it.
pub fn progress(v: &ScreenView, row: usize) -> String {
    let tree = TREES[row];
    if tree.is_big() {
        let max = v.maxes[row - BIG_MINING];
        if v.rank(row) == 0 {
            format!("0/{max}")
        } else {
            format!(
                "{}/{max} {}",
                v.rank(row),
                if v.on(row) { "on" } else { "off" }
            )
        }
    } else {
        format!("{}/{}", v.levels[row], tree.max())
    }
}

/// What a row gives now.
pub fn effect(v: &ScreenView, row: usize) -> String {
    match row {
        MELEE_ROW => format!(
            "+{:.0}% damage",
            v.tune.melee_step * v.levels[row] as f32 * 100.0
        ),
        ARMOUR_ROW => format!(
            "-{:.1}% damage",
            v.tune.armor_cut * v.levels[row] as f32 * 100.0
        ),
        BULWARK_ROW => format!(
            "-{:.0}% damage taken",
            v.tune.ward_step * v.levels[row] as f32 * 100.0
        ),
        FIST_ROW => format!(
            "{} fists",
            FIST_TIERS[v.levels[row].min(FIST_MAX) as usize].to_lowercase()
        ),
        row if row == BIG_MINING => format!("+{}% mining speed", 50 * v.rank(row)),
        row if row == BIG_MINING + 1 => format!("{}% extra drop", 25 * v.rank(row)),
        _ => format!("{} Haste", if v.on(row) { "gives" } else { "no" }),
    }
}

/// The line under a row's progress in the tooltip: what one more buys.
pub fn next_line(v: &ScreenView, row: usize) -> String {
    let tree = TREES[row];
    if tree.is_big() {
        if v.rank(row) == 0 {
            format!(
                "Unlock for {} point{}",
                v.cost(row),
                if v.cost(row) == 1 { "" } else { "s" }
            )
        } else if v.rank(row) < v.maxes[row - BIG_MINING] {
            format!(
                "Next rank for {} point{}",
                v.cost(row),
                if v.cost(row) == 1 { "" } else { "s" }
            )
        } else {
            "Last rank".to_owned()
        }
    } else if v.levels[row] < tree.max() {
        format!("Next: {} for 1 point", tree.per_level(v.tune))
    } else {
        "Maxed".to_owned()
    }
}

/// The hovered row's vanilla tooltip.
pub fn tooltip(v: &ScreenView, row: usize) -> Vec<String> {
    vec![
        TREES[row].title().to_owned(),
        progress(v, row),
        effect(v, row),
        String::new(),
        next_line(v, row),
    ]
}

/// The row the pointer is over, if any.
pub fn hovered(v: &ScreenView) -> Option<usize> {
    for (i, row) in v.state.branch.rows().iter().enumerate() {
        if inside(row_rect(i), v.mouse) {
            return Some(*row);
        }
    }
    None
}

/// The headline button's label.
fn primary_label(v: &ScreenView, row: usize) -> String {
    match primary(v, row) {
        Some(Action::Level) => "Level up — 1 pt".to_owned(),
        Some(Action::Unlock) => format!("Unlock — {} pt", v.cost(row)),
        Some(Action::Toggle) => {
            if v.on(row) {
                "Turn off".to_owned()
            } else {
                "Turn on".to_owned()
            }
        }
        Some(Action::Rank) => format!("Rank up — {} pt", v.cost(row)),
        None => "Maxed".to_owned(),
    }
}

/// The footer's line: what the selected node is.
fn footer(v: &ScreenView, row: usize) -> String {
    format!("{} — {}", TREES[row].title(), effect(v, row))
}

/// The tab, node and button shapes of the whole screen.
pub fn screen_shapes(v: &ScreenView, emblem: bool) -> Vec<ScreenShape> {
    let mut out = Vec::with_capacity(48);
    // The panel, its shadow and its border.
    out.push(ScreenShape::Fill(
        [2.0, 2.0, SCREEN_W, SCREEN_H],
        0x60_00_00_00,
    ));
    out.push(ScreenShape::Fill([0.0, 0.0, SCREEN_W, SCREEN_H], SCREEN_EDGE));
    out.push(ScreenShape::Fill(
        [1.0, 1.0, SCREEN_W - 2.0, SCREEN_H - 2.0],
        SCREEN_BG,
    ));
    out.push(ScreenShape::Fill(
        [8.0, 38.0, SCREEN_W - 16.0, 1.0],
        SCREEN_INNER,
    ));
    out.push(ScreenShape::Fill(
        [8.0, 160.0, SCREEN_W - 16.0, 1.0],
        SCREEN_INNER,
    ));
    // Header.
    out.push(ScreenShape::TextCenter(
        "Skill Trees".to_owned(),
        7.0,
        INK,
        true,
    ));
    out.push(ScreenShape::TextRight(
        format!(
            "{} point{}",
            v.points,
            if v.points == 1 { "" } else { "s" }
        ),
        SCREEN_W - 10.0,
        7.0,
        if v.points > 0 { ACCENT } else { DIM },
        true,
    ));
    out.push(ScreenShape::Fill([10.0, 22.0, 130.0, 8.0], 0xFF_00_00_00));
    let filled = 130.0 * (v.bank / XP_PER_POINT).clamp(0.0, 1.0);
    if filled > 0.0 {
        out.push(ScreenShape::Fill([10.0, 22.0, filled, 8.0], FILL));
    }
    out.push(ScreenShape::Text(
        format!("{:.0}%", (v.bank / XP_PER_POINT * 100.0).min(100.0)),
        [146.0, 21.0],
        INK,
        false,
    ));
    out.push(ScreenShape::Button {
        label: if v.state.banking {
            "Banking...".to_owned()
        } else {
            "Bank XP".to_owned()
        },
        rect: BANK_RECT,
        enabled: true,
        hovered: v.state.banking || inside(BANK_RECT, v.mouse),
    });
    // Branch tabs.
    for (i, branch) in Branch::ALL.iter().enumerate() {
        out.push(ScreenShape::Button {
            label: branch.title().to_owned(),
            rect: TAB_RECTS[i],
            enabled: true,
            hovered: v.state.branch == *branch || inside(TAB_RECTS[i], v.mouse),
        });
    }
    // The branch's rows.
    for (i, row) in v.state.branch.rows().iter().enumerate() {
        let row = *row;
        let rect = row_rect(i);
        let selected = v.state.selected == Some(row);
        if selected {
            out.push(ScreenShape::Fill(rect, ROW_SELECTED));
            out.push(ScreenShape::Fill([rect[0], rect[1], 2.0, rect[3]], ACCENT));
        } else if inside(rect, v.mouse) {
            out.push(ScreenShape::Fill(rect, ROW_HOVER));
        }
        let [x, y] = icon_at(i);
        if row == FIST_ROW && emblem {
            out.push(ScreenShape::Emblem([x, y], 24.0));
        } else {
            out.push(ScreenShape::Fill([x, y, 24.0, 24.0], NODE_BOX));
            out.push(ScreenShape::Fill(
                [x + 1.0, y + 1.0, 22.0, 22.0],
                NODE_INNER,
            ));
            out.push(ScreenShape::Icon(
                node_item(v, row),
                [x + 4.0, y + 4.0],
                16.0,
            ));
        }
        let [lx, ly] = label_at(i);
        out.push(ScreenShape::Text(
            TREES[row].title().to_owned(),
            [lx, ly],
            INK,
            true,
        ));
        out.push(ScreenShape::TextRight(
            progress(v, row),
            SCREEN_W - 10.0,
            ly,
            if primary(v, row).is_some() && affordable(v, row) {
                GOOD
            } else {
                DIM
            },
            false,
        ));
        let [ex, ey] = effect_at(i);
        out.push(ScreenShape::Text(
            effect(v, row),
            [ex, ey],
            if row >= BIG_MINING && !v.on(row) {
                DIM
            } else {
                GOOD
            },
            false,
        ));
    }
    // Footer.
    if let Some(row) = v.state.selected {
        out.push(ScreenShape::Text(
            footer(v, row),
            [10.0, 166.0],
            DIM,
            false,
        ));
        if TREES[row].is_big() && v.rank(row) > 0 {
            out.push(ScreenShape::Button {
                label: if v.on(row) {
                    "Turn off".to_owned()
                } else {
                    "Turn on".to_owned()
                },
                rect: TOGGLE_RECT,
                enabled: true,
                hovered: inside(TOGGLE_RECT, v.mouse),
            });
            let maxed = v.rank(row) >= v.maxes[row - BIG_MINING];
            out.push(ScreenShape::Button {
                label: if maxed {
                    "Maxed".to_owned()
                } else {
                    format!("Rank up — {} pt", v.cost(row))
                },
                rect: ACTION_RECT,
                enabled: !maxed && affordable(v, row),
                hovered: inside(ACTION_RECT, v.mouse),
            });
        } else {
            out.push(ScreenShape::Button {
                label: primary_label(v, row),
                rect: ACTION_RECT,
                enabled: match primary(v, row) {
                    Some(Action::Level) | Some(Action::Unlock) => affordable(v, row),
                    Some(Action::Toggle) => true,
                    _ => false,
                },
                hovered: inside(ACTION_RECT, v.mouse),
            });
        }
    }
    out.push(ScreenShape::Button {
        label: "Done".to_owned(),
        rect: DONE_RECT,
        enabled: true,
        hovered: inside(DONE_RECT, v.mouse),
    });
    out
}

/// The icon a row draws when the emblem is not up: the fist's changes with
/// the tier it has bought.
pub fn node_item(v: &ScreenView, row: usize) -> &'static str {
    match row {
        MELEE_ROW => "minecraft:iron_sword",
        ARMOUR_ROW => "minecraft:iron_chestplate",
        BULWARK_ROW => "minecraft:shield",
        FIST_ROW => FIST_ICONS[v.levels[row].min(FIST_MAX) as usize],
        row if row == BIG_MINING => "minecraft:golden_pickaxe",
        row if row == BIG_MINING + 1 => "minecraft:diamond",
        _ => "minecraft:blaze_powder",
    }
}

/// The item a fist tier falls back to when the emblem is not uploaded.
pub const FIST_ICONS: [&str; 6] = [
    "minecraft:stick",
    "minecraft:wooden_pickaxe",
    "minecraft:stone_pickaxe",
    "minecraft:iron_pickaxe",
    "minecraft:diamond_pickaxe",
    "minecraft:netherite_pickaxe",
];

/// A press on the screen: what it does to the state and to the skills, and
/// what the tick should say.
pub fn press(v: &ScreenView, hit: ScreenHit, s: &mut Skills) -> (ScreenState, Vec<Note>) {
    let mut state = v.state;
    let mut notes = Vec::new();
    match hit {
        ScreenHit::Tab(i) => {
            let branch = Branch::ALL[i.min(1)];
            state.branch = branch;
            state.selected = branch.rows().first().copied();
            state.banking = false;
        }
        ScreenHit::Node(row) => {
            if state.selected == Some(row) {
                if let Some(action) = primary(v, row) {
                    notes = apply(v, row, action, s);
                }
            } else {
                state.selected = Some(row);
            }
        }
        ScreenHit::Bank => state.banking = true,
        ScreenHit::Done => notes.push(Note::Close),
        ScreenHit::Action => {
            if let Some(row) = state.selected {
                if let Some(action) = primary(v, row) {
                    notes = apply(v, row, action, s);
                }
            }
        }
        ScreenHit::Toggle => {
            if let Some(row) = state.selected {
                if TREES[row].is_big() && v.rank(row) > 0 {
                    notes = apply(v, row, Action::Toggle, s);
                }
            }
        }
        ScreenHit::Rank => {
            if let Some(row) = state.selected {
                if TREES[row].is_big() {
                    notes = apply(v, row, Action::Rank, s);
                }
            }
        }
    }
    (state, notes)
}

/// Spends on `row` as `action` says, and phrases what happened.
fn apply(v: &ScreenView, row: usize, action: Action, s: &mut Skills) -> Vec<Note> {
    let tree = TREES[row];
    let buy = action == Action::Rank;
    let out = s.press(tree, buy, v.tune);
    let max = if tree.is_big() {
        v.maxes[row - BIG_MINING]
    } else {
        tree.max()
    };
    let level = if tree.is_big() {
        s.big_rank(row).min(max)
    } else {
        s.levels[row]
    };
    let said = match out {
        Press::Spent(_) | Press::Ranked(_) => format!("{} is now {}/{}", tree.title(), level, max),
        Press::Unlocked(_) => format!("{} unlocked", tree.title()),
        Press::On(_) => format!("{} on", tree.title()),
        Press::Off(_) => format!("{} off", tree.title()),
        Press::NoPoints => "No points to spend: hold Bank XP to bank experience".to_owned(),
        Press::Maxed => format!("{} is already at its last level", tree.title()),
    };
    let ok = !matches!(out, Press::NoPoints | Press::Maxed);
    let mut notes = vec![Note::Bar(said)];
    if ok {
        notes.push(Note::Sound(CLICK, 0.5, 1.2));
    }
    notes
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

    /// The screen spends on the window's thread; single player breaks blocks
    /// on the game's server thread. The point has to be there on both, which a
    /// copy per thread would not do.
    #[test]
    fn a_spent_point_reaches_the_thread_that_breaks_blocks() {
        let mut s = Skills::default();
        s.points = 1;
        assert!(s.spend(Tree::Fist));
        store(s.clone());
        let seen = std::thread::spawn(current).join().expect("thread");
        assert_eq!(seen.fist_tier(), 1);
        assert_eq!(seen, s);
        // Leave the shared state as the other tests expect to find it.
        store(Skills::default());
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
        assert_eq!(s.fist_speed(), 9.0);
        assert_eq!(Skills::default().points, 0);
        assert!(!Skills::default().spend(Tree::Armor));
    }

    #[test]
    fn the_trees_scale_the_way_they_read() {
        let mut s = Skills::default();
        let tune = Tune::default();
        assert_eq!(s.melee_multiplier(tune), 1.0);
        assert_eq!(s.armor_cut(tune), 0.0);
        assert_eq!(s.fist_speed(), 1.0);
        s.levels = [10, 10, 0, 4, 0, 0, 0];
        assert!((s.melee_multiplier(tune) - 1.8).abs() < 1e-6);
        assert!((s.armor_cut(tune) - 0.25).abs() < 1e-6);
        assert!((s.incoming_damage(tune, 9.0) - 6.75).abs() < 1e-6);
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
        assert_eq!(weak.fist_speed(), 8.0);
        assert!(weak.fist_covers(4));
        // The last point is netherite: faster than diamond, and it still
        // harvests everything diamond does.
        weak.levels[Tree::Fist.index()] = 5;
        assert_eq!(weak.fist_speed(), 9.0);
        assert!(weak.fist_covers(4));
        assert!(weak.fist_drop(false, 4));
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
        // Ten armour levels cut a quarter of the hit.
        assert!((s.armor_cut(tune) - 0.25).abs() < 1e-6);
        // No ranks: the armour share still comes off, never to nothing.
        assert_eq!(s.ward_cut(tune), 0.0);
        assert!((s.incoming_damage(tune, 8.0) - 6.0).abs() < 1e-6);
        assert!((s.incoming_damage(tune, 2.0) - 1.5).abs() < 1e-6);
        // Each rank adds 10%, and all three still stop at half the hit.
        s.levels[Tree::Bulwark.index()] = 1;
        assert!((s.incoming_damage(tune, 8.0) - 5.2).abs() < 1e-6);
        s.levels[Tree::Bulwark.index()] = 2;
        assert!((s.incoming_damage(tune, 8.0) - 4.4).abs() < 1e-6);
        s.levels[Tree::Bulwark.index()] = 3;
        assert!((s.incoming_damage(tune, 8.0) - 4.0).abs() < 1e-6);
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
        assert!((ward.incoming_damage(tune, 8.0) - 6.4).abs() < 1e-6);
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
    fn the_bar_draws_the_fill_and_the_points() {
        let out = bar(&BarView {
            bank: 50.0,
            points: 2,
        });
        assert_eq!(out[0], Shape::Fill([0.0, 0.0, 182.0, 12.0], BACKGROUND));
        assert_eq!(out[1], Shape::Fill([1.0, 1.0, 90.0, 10.0], FILL));
        assert!(out.contains(&Shape::Text("XP  50%".to_owned(), [3.0, 2.0], INK, true)));
        assert!(
            out.iter()
                .any(|s| matches!(s, Shape::Text(t, _, _, _) if t == "2 points"))
        );
    }

    #[test]
    fn the_screen_lists_each_branch_and_hits_its_buttons() {
        let s = Skills::default();
        let v = ScreenView::of(&s, ScreenState::default(), [0.0, 0.0]);
        // Combat starts on its first node.
        assert_eq!(v.state.branch, Branch::Combat);
        assert_eq!(v.state.selected, Some(MELEE_ROW));
        assert_eq!(Branch::Gathering.rows(), &GATHERING_ROWS[..]);
        // The taller branch's four rows fit above the footer.
        let last = row_rect(3);
        assert!(last[1] + last[3] <= 160.0);
        // Tabs, rows, the bank, the footer and Done.
        assert_eq!(
            hit(&v, [TAB_RECTS[1][0] + 2.0, TAB_RECTS[1][1] + 2.0]),
            Some(ScreenHit::Tab(1))
        );
        assert_eq!(
            hit(&v, [100.0, NODE_TOP + 2.0]),
            Some(ScreenHit::Node(MELEE_ROW))
        );
        assert_eq!(
            hit(&v, [BANK_RECT[0] + 1.0, BANK_RECT[1] + 1.0]),
            Some(ScreenHit::Bank)
        );
        assert_eq!(
            hit(&v, [DONE_RECT[0] + 1.0, DONE_RECT[1] + 1.0]),
            Some(ScreenHit::Done)
        );
        assert_eq!(
            hit(&v, [ACTION_RECT[0] + 1.0, ACTION_RECT[1] + 1.0]),
            Some(ScreenHit::Action)
        );
        // Outside the panel is nothing.
        assert_eq!(hit(&v, [SCREEN_W + 4.0, 10.0]), None);
    }

    #[test]
    fn a_click_selects_then_spends() {
        let mut s = Skills {
            points: 2,
            ..Skills::default()
        };
        let state = ScreenState {
            selected: None,
            ..ScreenState::default()
        };
        // The first click on a node only selects it.
        let v = ScreenView::of(&s, state, [0.0, 0.0]);
        let (state, notes) = press(&v, ScreenHit::Node(MELEE_ROW), &mut s);
        assert_eq!(state.selected, Some(MELEE_ROW));
        assert!(notes.is_empty());
        assert_eq!(s.levels[MELEE_ROW], 0);
        // The second click spends a point.
        let v = ScreenView::of(&s, state, [0.0, 0.0]);
        let (_, notes) = press(&v, ScreenHit::Node(MELEE_ROW), &mut s);
        assert_eq!(s.levels[MELEE_ROW], 1);
        assert_eq!(s.points, 1);
        assert!(matches!(notes.first(), Some(Note::Bar(_))));
        // A branch switch selects that branch's first node.
        let v = ScreenView::of(&s, state, [0.0, 0.0]);
        let (state, _) = press(&v, ScreenHit::Tab(1), &mut s);
        assert_eq!(state.branch, Branch::Gathering);
        assert_eq!(state.selected, Some(FIST_ROW));
    }

    #[test]
    fn the_fist_node_draws_the_emblem_when_it_is_up() {
        let s = Skills::default();
        let state = ScreenState {
            branch: Branch::Gathering,
            selected: Some(FIST_ROW),
            banking: false,
        };
        let v = ScreenView::of(&s, state, [0.0, 0.0]);
        let with = screen_shapes(&v, true);
        let without = screen_shapes(&v, false);
        assert!(
            with.iter()
                .any(|s| matches!(s, ScreenShape::Emblem(_, 24.0)))
        );
        assert!(
            !without
                .iter()
                .any(|s| matches!(s, ScreenShape::Emblem(_, 24.0)))
        );
        // The fallback is the tier's pickaxe: bare hands at tier 0.
        assert!(without.contains(&ScreenShape::Icon(
            "minecraft:stick",
            [100.0, NODE_TOP + 5.0],
            16.0
        )));
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
        assert!(text.contains("Armour 0/10 -2.5% damage"));
        assert!(text.contains("Bulwark 0/3 10% cut"));
        assert!(text.contains("Fist mining 2/5"));
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
        assert!((trees.armor_cut(tune) - 0.5).abs() < 1e-6);
        assert_eq!(trees.ward_cut(tune), 0.0);
        assert!(tune.set(Tree::Fortune, Field::Tier, 5.0).is_ok());
        assert_eq!(tune.max(BIG_FORTUNE), 5);
        // An old line without them keeps the defaults.
        let old = Tune::decode("1|0.50|0.25|200|1|1|1|1");
        assert_eq!(old.melee_step, MELEE_PER_LEVEL);
        assert_eq!(old.armor_cut, ARMOR_CUT_PER_LEVEL);
        // A version-2 line has no cut field, so later fields stay aligned.
        let v2 = Tune::decode("2|0.50|0.25|200|1|1|1|1|0.08|1.50|3|4|5");
        assert_eq!(v2.ward_step, WARD_CUT_PER_RANK);
        assert_eq!(v2.armor_cut, ARMOR_CUT_PER_LEVEL);
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
        // A netherite fist covers diamond, at its 9.0 speed.
        s.levels[Tree::Fist.index()] = FIST_MAX;
        let diamond = MiningFacts {
            needs: 4,
            ..stone
        };
        assert!((s.mining_factor(tune, diamond) - 2.0 * 9.0 * (100.0 / 30.0)).abs() < 1e-4);
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

    /// A landed hit always hurts: the cuts are shares of the hit, capped, and a
    /// hit never falls below one point, so knockback and arrow hits still run.
    #[test]
    fn the_armour_tree_never_makes_the_player_untouchable() {
        let tune = Tune::default();
        let mut s = Skills::default();
        s.levels[Tree::Armor.index()] = 6;
        // A zombie's normal hit is 3 points: some of it still lands.
        let hit = s.incoming_damage(tune, 3.0);
        assert!(hit >= INCOMING_FLOOR, "level 6 left {hit}");
        assert!(hit < 3.0);
        // Ten levels with Bulwark at three ranks cut half the hit, no more.
        let mut all = Skills::default();
        all.levels[Tree::Armor.index()] = MAX_LEVEL;
        all.levels[Tree::Bulwark.index()] = BULWARK_MAX;
        assert_eq!(all.incoming_damage(tune, 10.0), 5.0);
        // Tuning pushed past the caps cannot beat the floor.
        let wild = Tune {
            armor_cut: 1.0,
            ward_step: 1.0,
            ..tune
        };
        assert_eq!(all.incoming_damage(wild, 3.0), 1.5);
        assert_eq!(all.incoming_damage(wild, 0.5), 0.5);
        assert_eq!(all.incoming_damage(wild, 0.0), 0.0);
    }

    #[test]
    fn an_older_tuning_line_takes_the_new_armour_share() {
        let old = Tune::decode("3|0.50|0.25|200|1|1|1|1|0.08|0.50|0.25|3|3|3");
        assert_eq!(old.armor_cut, ARMOR_CUT_PER_LEVEL);
        assert_eq!(old.ward_step, 0.25);
        assert_eq!(Tune::decode(&Tune::default().encode()), Tune::default());
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
            "tkatt-skills: Fist mining 3/5, 5 point(s) to spend."
        );
        s.run(Op::Reset);
        assert_eq!(s.levels, [0; ROWS]);
        assert_eq!(s.points, 5);
        // A status command is not a change and keeps the state as it is.
        s.run(Op::Status);
        assert_eq!(s.levels, [0; ROWS]);
    }
}
