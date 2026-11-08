// Whether a block drops for this hand: the fist tier counts as a pickaxe of
// its tier for the blocks vanilla tags as needing one. The rule is
// `Skills::fist_drop` in runtime.rs, where it is unit-tested; this helper only
// gathers the game's facts and hands them over.
fn skill_trees_fist_drops(
    value: bool,
    game: &crate::game::Game<'_>,
    facts: &horizon_items::blocks::StateFacts,
) -> bool {
    use crate::game::tkatt_skills as m;
    if value {
        return true;
    }
    // The break runs on the game's server thread, where the screen's load has
    // not happened: read the state here (once) rather than trust `current`.
    let skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    if skills.fist_tier() == 0 {
        return false;
    }
    let bare = game
        .held()
        .is_none_or(|s| &*s.item == "minecraft:air" || s.count == 0);
    if !bare {
        return false;
    }
    // Wood, stone, iron and diamond blocks each carry the tag of the tool
    // they need; the highest one decides.
    let items = game.items();
    let tags = [
        "minecraft:needs_wood_tool",
        "minecraft:needs_stone_tool",
        "minecraft:needs_iron_tool",
        "minecraft:needs_diamond_tool",
    ];
    let mut needed = 0;
    for (i, tag) in tags.iter().enumerate() {
        if items.block_in_tag(tag, &facts.name) {
            needed = i as u32 + 1;
        }
    }
    skills.fist_drop(value, needed)
}
