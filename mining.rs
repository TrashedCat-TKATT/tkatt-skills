// Mining progress: the big mining skill speeds up every block by its rank, and
// the fist tree turns a bare hand into a pickaxe of its tier on the blocks
// vanilla tags for one. The arithmetic is `Skills::mining_factor` in
// runtime.rs, where it is unit-tested; this helper only gathers the game's
// facts and applies the factor.
fn skill_trees_mining(
    value: f32,
    game: &crate::game::Game<'_>,
    facts: &horizon_items::blocks::StateFacts,
) -> f32 {
    use crate::game::tkatt_skills as m;
    if facts.hardness == 0.0 {
        return value;
    }
    // Mining progress is computed on the game's server thread, where the
    // screen's load has not happened: read the state here (once).
    let skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    let tune = m::tuning();
    if skills.fist_tier() == 0 && skills.mining_multiplier(tune) == 1.0 {
        return value;
    }
    let held = game.held();
    let bare = held.is_none_or(|s| &*s.item == "minecraft:air" || s.count == 0);
    let items = game.items();
    let pickaxe = bare && items.block_in_tag("minecraft:mineable/pickaxe", &facts.name);
    let base_speed = if pickaxe {
        items.mining_speed(held, &facts.name)
    } else {
        0.0
    };
    let mut needs = 0;
    if facts.requires_tool {
        let tags = [
            "minecraft:needs_wood_tool",
            "minecraft:needs_stone_tool",
            "minecraft:needs_iron_tool",
            "minecraft:needs_diamond_tool",
        ];
        for (i, tag) in tags.iter().enumerate() {
            if items.block_in_tag(tag, &facts.name) {
                needs = i as u32 + 1;
            }
        }
    }
    let factor = skills.mining_factor(
        tune,
        m::MiningFacts {
            bare,
            pickaxe,
            base_speed,
            needs,
            covers: items.correct_for_drops(held, &facts.name),
        },
    );
    value * factor
}
